use std::mem;
use std::slice;
use std::str::CharIndices;
use std::iter::Peekable;
use std::collections::HashMap;

const ATTRDEFHEAD : &'static str = "<!-- attrib ";
const ATTRDEFEND  : &'static str = " -->";

#[derive(Debug)]
enum Elem<'base> {
    Text(&'base str),
    Attr(&'base str),
    AttrDef(&'base str, Vec<Elem<'base>>),
    Cond(&'base str, &'base str, Vec<Elem<'base>>, bool),
}

#[derive(Clone, Debug)]
struct Parser<'source>(
    &'source str,
    Peekable<CharIndices<'source>>,
    Option<&'static str>,
);
impl<'a> Parser<'a> {
    fn new(source: &'a str) -> Self {
        Parser(source,
               source.char_indices().peekable(),
               None)
    }
    fn commit(&mut self, oth: Self) {
        *self = oth;
    }
    fn peek(&mut self) -> Option<(usize, char)> {self.1.peek().copied()}
    fn pull(&mut self) -> Option<(usize, char)> {self.1.next()}
    fn pos(&mut self) -> usize {
        self.1.peek().map(|x|x.0).unwrap_or_else(|| self.0.len())
    }
    fn push_nesting(&mut self, nest: &'static str) {
        self.2 = Some(nest);
    }
    fn pop_nesting(&mut self) { self.2 = None; }
    fn scan_str(&mut self, base: &str) -> Option<()> {
        let mut parser = self.clone();
        for c in base.chars() {
            if parser.pull()?.1 != c { return None }
        }
        self.commit(parser);
        Some(())
    }
    fn test_str(&mut self, base: &str) -> bool {
        let mut parser = self.clone();
        for c in base.chars() {
            match parser.pull() {
                None => {return false;}
                Some(x) if x.1 != c => { return false }
                _ => ()
            }
        }
        true
    }
    fn scan_delim(&mut self, delim: &str) -> Option<&'a str> {
        let start = self.pos();
        let mut parser = self.clone();
        while !parser.test_str(delim) {
            parser.pull()?;
        }
        self.commit(parser);
        let end = self.pos();
        Some(&self.0[start .. end])
    }
    fn scan_delim_char2(&mut self, delim_a: char, delim_b: char) -> Option<&'a str> {
        let start = self.pos();
        let mut parser = self.clone();
        while !(parser.peek()?.1 == delim_a || parser.peek()?.1 == delim_b) {
            parser.pull()?;
        }
        self.commit(parser);
        let end = self.pos();
        Some(&self.0[start .. end])
    }
    fn test_head(&mut self) -> bool {
        // tests if we're at the head of anything
        if self.peek().is_none() { return false }
        if self.peek().unwrap().1 == '[' { return true }
        if self.test_str(ATTRDEFHEAD) { return true }
        false
    }
    fn test_nest(&mut self) -> bool {
        // tests if we're on something significant to nesting
        match self.2 {
            None => false,
            Some(x) => self.test_str(x)
        }
    }
}

impl<'a> Parser<'a> {
    fn parse_text(&mut self) -> Elem<'a> {
        eprintln!("\t[text @ {}]", self.pos());
        let start = self.pos();
        if self.test_head() {
            // eprintln!("[text] skipping head");
            self.pull();
        }
        while !(self.test_head() || self.test_nest()) {
            // eprintln!("[text] pulling and testing");
            if self.pull().is_none() { break; }
        }
        let end = self.pos();
        Elem::Text(&self.0[start .. end])
    }
    fn parse_attr(&mut self) -> Option<Elem<'a>> {
        eprintln!("\t[attr @ {}]", self.pos());
        let mut parser = self.clone();
        parser.scan_str("[#")?;
        let cont = parser.scan_delim("#]")?;
        parser.scan_str("#]")?;
        self.commit(parser);
        Some(Elem::Attr(cont))
    }
    fn parse_attr_def(&mut self) -> Option<Elem<'a>> {
        eprintln!("\t[attr def @ {}]", self.pos());
        let mut parser = self.clone();
        // eprintln!("[attr def] handling head");
        parser.scan_str(ATTRDEFHEAD)?;
        parser.push_nesting(ATTRDEFEND);
        
        // eprintln!("[attr def] getting target");
        let targ = parser.scan_delim(" : ")?;
        parser.scan_str(" : ")?;
        
        // eprintln!("[attr def] recursing");
        let mut inner = Vec::new();
        while !parser.test_str(ATTRDEFEND) {
            inner.push(parser.parse_element()?);
        }

        // eprintln!("[attr def] cleaning up");
        parser.pop_nesting();
        parser.scan_str(ATTRDEFEND)?;
        self.commit(parser);
        Some(Elem::AttrDef(targ, inner))
    }

    fn parse_conditional(&mut self) -> Option<Elem<'a>> {
        // Just compare attr and string
        eprintln!("\t[cond] @ {}", self.pos());
        let mut parser = self.clone();
        //eprintln!("[cond] consuming head");
        parser.scan_str("[")?;

        let start = parser.pos();
        //eprintln!("[cond] consuming var");
        let capture = parser.scan_delim_char2('!', '=')?;
        //eprintln!("[cond] got var {capture:?}");
        //eprintln!("[cond] consuming method");
        let method = parser.scan_str("!=").map(|_|"!=").or_else(||parser.scan_str("=").map(|_|"="))?;
        //eprintln!("[cond] got method {method:?}");
        //eprintln!("[cond] consuming end");
        
        let end = parser.pos();
        let termin = &parser.0[start .. end];
        //eprintln!("[cond] expecting terminator {termin:?}");
        
        //eprintln!("[cond] parsing expected");
        let expected = parser.scan_delim("]")?;
        //eprintln!("[cond] got {expected:?}");
        parser.pull()?;

        parser.push_nesting("[/");
        //eprintln!("[cond] parsing inner");
        let mut inner = Vec::new();
        while !parser.test_str("[/") {
            inner.push(parser.parse_element()?);
        }
        //eprintln!("[cond] got {inner:?}");
        parser.scan_str("[/");
        parser.pop_nesting();
        
        //eprintln!("[cond] consuming terminator");
        let _terminator = parser.scan_str(termin)?;
        //eprintln!("[cond] success");
        parser.scan_str("]");

        self.commit(parser);

        Some(Elem::Cond(capture, expected, inner, method=="!="))
    }

    fn parse_element(&mut self) -> Option<Elem<'a>> {
        eprintln!("\t[elem @ {}]", self.pos());
        // eprintln!("[elem] testing existance");
        if self.peek().is_none() { return None }
        // eprintln!("[elem] trying attr");
        if let Some(res) = self.parse_attr() { return Some(res) }
        // eprintln!("[elem] trying attr def");
        if let Some(res) = self.parse_attr_def() { return Some(res) }
        // eprintln!("[elem] trying conditional");
        if let Some(res) = self.parse_conditional() { return Some(res) }
        // eprintln!("[elem] trying text");
        return Some(self.parse_text());
    }
}

struct Machine<'source, 'code: 'source> {
    vars: HashMap<String, &'code[Elem<'source>]>,
    // vars: Vec<HashMap<String, &'code[Elem<'source>]>>,
    code: Vec<(slice::Iter<'code, Elem<'source>>, bool)>,
    outp: String,
}
impl<'s, 'c: 's> Machine<'s, 'c> {
    fn new(code: &'c[Elem<'s>]) -> Self {
        Machine{
            // vars: vec![HashMap::new()],
            vars: HashMap::new(),
            code: vec![(code.iter(), false)],
            outp: String::new(),
        }
    }
    fn new_frame(&mut self, code: &'c[Elem<'s>]) {
        // self.vars.push(HashMap::new());
        self.code.push((code.iter(), false));
    }
    fn drop_frame(&mut self) {
        // self.vars.pop();
        self.code.pop();
    }
    fn get_instr(&mut self) -> Option<&'c Elem<'s>> {
        let (end, signif) = self.code.last_mut()?;
        let signif = *signif;
        match end.next() {
            None => {self.drop_frame(); if signif {None} else {self.get_instr()}},
            Some(instr) => Some(instr),
        }
    }
    fn invoke(&mut self, handle: &'s str) {
        /*
        for i in self.vars.iter().rev() {
            if !i.contains_key(handle) {continue}
            self.new_frame(i.get(handle).unwrap());
            return;
        }
        */
        if let Some(frame) = self.vars.get(handle)
        {self.new_frame(frame);}
        else { self.new_frame(&[]); }
    }
    fn make_signif(&mut self) {
        if let Some((_, sig)) = self.code.last_mut() {
            *sig = true;
        }
    }
    fn step(&mut self) -> bool {
        eprintln!("[eval] getting next instr");
        let oper = match self.get_instr() {
            None => {return false;},
            Some(x) => x
        };
        eprintln!("\tgot {oper:?}");
        match oper {
            Elem::Text(val) => {
                eprintln!("[eval] writing text");
                self.outp.push_str(val);
            },
            Elem::Attr(name) => {
                eprintln!("[eval] invoking {name:?}");
                self.invoke(name);
            },
            Elem::AttrDef(handle, ref value) =>{
                eprintln!("[eval] defining attr {handle:?}");
                self.vars.entry(handle.to_string())
                    .and_modify(|x| *x=value).or_insert(value);
            } 
            Elem::Cond(invoke,result, body, negate) => {
                eprintln!("[eval] cond on {invoke:?}");
                let mut local = String::new();
                mem::swap(&mut local, &mut self.outp);
                self.invoke(invoke);
                self.make_signif();
                self.run();
                mem::swap(&mut local, &mut self.outp);
                let mut res = local.as_str() == *result;
                if *negate { res = !res; } 
                if res {
                    self.new_frame(body);
                }
            }
            _ => {}
        }
        true
    }
    fn run(&mut self) {
        while self.step() {}
    }
}

fn main() {
    let test = r#"
<!-- attrib magic : cool -->
<!-- attrib testo : test -->
[#testo#][testo=]magic[/testo=]
    "#;

    let mut parser = Parser::new(test);
    println!("[PARSING] {test:?}");
    let mut code = Vec::new();
    while parser.peek().is_some() {
        let res = parser.parse_element();
        match res {
            Some(res) => code.push(res),
            None => { println!("[PARSE FAILED]"); break }
        }
    }
    println!("[PARSED] {code:?}");

    let mut _outer = Machine::new(&code);
    _outer.run();
    println!("{:?}", _outer.outp);
}

// TODO
//  -   Conditionals
//  -   most of everything else
