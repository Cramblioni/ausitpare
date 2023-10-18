use std::collections::HashMap;
type BufInd = usize;
type InstrRel = u16;
type StrInd = u32;
type CodeInd = u32;
type InstrInd = usize;

#[derive(Debug, Clone, Copy)]
enum Instr {
    PutStr(StrInd),
    Invoke(CodeInd),
    Proceed,
    
    Mode(Mode),
    PrepScan(InstrRel),
    DropScan,
    Skip(InstrRel),
    
    Trap, // A runtime error for a compile time temporary
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Mode{ Read, Write }

#[derive(Debug, Clone)]
enum Elem<'base> {
    Text(&'base str),
    Attr(&'base str),
    AttrDef(&'base str, Vec<Self>),
    Cond(&'base str, Vec<Self>, Vec<Self>, bool),
}
type Code = Vec<Instr>;
struct Compiler<'base> {
    strs: Vec<&'base str>,
    str_ind: HashMap<&'base str, StrInd>,
    attrs: Vec<Option<Code>>,
    attr_ind: HashMap<&'base str, CodeInd>,
    queue: Vec<(&'base str, Vec<Elem<'base>>)>,
}
#[allow(unused, dead_code)]
mod fuckyou {
    fn detach<'a, 'b: 'a, T>(x: &'a T) -> &'b T {
        unsafe { std::ptr::read(std::ptr::addr_of!(x).cast()) }
    }
    fn detach_mut<'a, 'b: 'a, T>(x: &'a mut T) -> &'b mut T {
        unsafe { std::ptr::read(std::ptr::addr_of!(x).cast()) }
    }
}
impl<'b> Compiler<'b> {
    fn new() -> Self {
        Compiler {
            strs: Vec::new(),
            str_ind: HashMap::new(),
            attrs: Vec::new(),
            attr_ind: HashMap::new(),
            queue: Vec::new(),
        }
    }
    fn to_exec(self) -> (Vec<Code>, Vec<&'b str>, HashMap<&'b str, CodeInd>) {
        (self.attrs.into_iter()
          .map(Option::unwrap).collect(),
        self.strs, self.attr_ind)
    }
    // returns `None` if the attribute already exists and is bound
    fn new_attr(&mut self, name: &'b str) -> Option<CodeInd> {
        if self.attr_ind.contains_key(name) {
            let ind = self.attr_ind[name];
            if self.attrs[ind as usize].is_some() {
                return None;
            }
            return Some(ind);
        }
        let ret = self.attrs.len() as CodeInd;
        self.attr_ind.insert(name, ret);
        self.attrs.push(None);
        Some(ret)
    }
    fn bind_attr(&mut self, name: &'b str) -> CodeInd {
        self.attr_ind.entry(name)
         .or_insert_with(|| {
            let ret = self.attrs.len() as CodeInd;
            self.attrs.push(None);
            ret
         }).clone()
    }
    fn bind_str(&mut self, string: &'b str ) -> StrInd {
        self.str_ind.entry(string)
         .or_insert_with(|| {
            let ret = self.strs.len() as StrInd;
            self.strs.push(string);
            ret
         }).clone()
    }
    fn enqueue(&mut self, name: &'b str, body: Vec<Elem<'b>>) {
        self.queue.push((name, body));
    }
    fn new_unit<'d>(&'d mut self, binding: CodeInd) -> CompileUnit<'b, 'd> {
        CompileUnit{
            compiler: self, // detach_mut(self),
            code: Vec::new(),
            binding
        }
    }
    fn prep(&mut self, init: Vec<Elem<'b>>) {
        self.attrs.push(None);
        let mut fst = self.new_unit(0);
        fst.push_many(init);
        fst.commit();
    }
    fn run(&mut self) -> Result<(), &'b str> {
        while let Some((name, body)) = self.queue.pop() {
            let ind = if let Some(x) = self.new_attr(name) {x}
                else {return Err(name)};
            let mut fst = self.new_unit(ind);
            fst.push_many(body);
            fst.commit();
        }
        Ok(())
    }
    fn done(&self) -> bool {self.queue.len() == 0}
}

struct CompileUnit<'base: 'dur, 'dur> {
    compiler: &'dur mut Compiler<'base>,
    code: Code,
    binding: CodeInd
}

impl<'b: 'd, 'd> CompileUnit<'b, 'd> {
    fn commit(mut self) {
        self.code.push(Instr::Proceed);
        let _ = self.compiler.attrs
         .get_mut(self.binding as usize)
         .unwrap()
         .insert(self.code);
    }
    fn push(&mut self, elem: Elem<'b>) {
        match elem {
            Elem::Text(string) => {
                eprintln!("compiling text");
                let ind = self.compiler.bind_str(string);
                self.code.push(Instr::PutStr(ind));
            }
            Elem::Attr(name) => {
                eprintln!("compiling attr");
                let ind = self.compiler.bind_attr(name);
                self.code.push(Instr::Invoke(ind));
            }
            Elem::AttrDef(name, body) => {
                eprintln!("compiling attr def");
                self.compiler.enqueue(name, body);
            }
            Elem::Cond(targ, cond, cond_body, negate) => {
                eprintln!("compiling cond");
                let head = self.code.len();
                self.code.push(Instr::Trap);
                self.push(Elem::Attr(targ));
                self.code.push(Instr::Mode(Mode::Read));
                self.push_many(cond);
                self.code.push(Instr::DropScan);
                    let body = self.code.len();
                    let jmp = (body - head) as InstrRel;
                    self.code[head] = Instr::PrepScan(jmp);
                if negate {
                    self.code.push(Instr::Trap);
                    let skip = self.code.len();
                    self.push_many(cond_body);
                    let body_end = self.code.len();
                    let dist = (body_end-skip) as InstrRel;
                    self.code[skip - 1] = Instr::Skip(dist);
                } else {
                    self.push_many(cond_body);
                }
            }
        }
    }
    fn push_many(&mut self, elems: Vec<Elem<'b>>) {
        for i in elems.into_iter() { self.push(i) }
    }
}

struct ScanFrame(InstrRel, BufInd, Mode, BufInd, InstrInd);

struct Machine<'base> {
    scans: Vec<ScanFrame>,
    buf: String,
    strs: &'base[&'base str],
    code: &'base [&'base [Instr]],
    frames: Vec<(usize, &'base [Instr])>,
}
impl<'b> Machine<'b> {
    fn new(code: &'b [&'b [Instr]], strs: &'b [&'b str]) -> Self {
        Machine{
            scans: Vec::new(),
            buf: String::new(),
            strs,
            code,
            frames: vec![(0, code[0])]
        }
    }
    fn get_mode(&self) -> Mode {
        self.scans.last().map(|x|x.2)
            .unwrap_or(Mode::Write)
    }
    fn new_scan(&mut self, fail: InstrRel) {
        self.scans.push(
            ScanFrame(fail, self.buf.len(),
            Mode::Write, self.buf.len(),
            self.frames.last().unwrap().0));
    }
    fn pop_scan(&mut self) {
        let x = if let Some(x) = self.scans.pop() {x} else {return};
        self.buf.truncate(x.3)
    }
    fn fail(&mut self) {
        // Like pop_scan, but we jump aswell
        
        let frame = self.scans.pop().unwrap();
        let last = self.frames.last_mut().unwrap();
        self.buf.truncate(frame.3);
        last.0 = frame.4 as usize + frame.0 as usize;
        println!("scan failed, jumping to {}({}+{})", last.0, frame.4, frame.0);
    }
    fn new_frame(&mut self, code: &'b [Instr]) {
        self.frames.push((0, code));
    }
    fn pop_frame(&mut self) {
        self.frames.pop();
    }
    fn s(&self) -> usize {
        self.scans.last().map(|x|x.1).unwrap_or(self.buf.len())
    }
    fn cs(&mut self, amt: usize) {
        let ScanFrame(_, ref mut s, _, _, _)
            = self.scans.last_mut().unwrap();
        *s += amt;
    }
    fn finished(&self) -> bool { self.frames.len() == 0 }
    fn get_instr(&mut self) -> Instr {
        let (ref mut pc, ref code) = self.frames.last_mut().unwrap();
        let opc = *pc;
        *pc += 1;
        code.get(opc).copied().unwrap()
    }
    fn step(&mut self) {
        match self.get_instr() {
            Instr::PutStr(sind) => match self.get_mode() {
                Mode::Write => 
                    {self.buf.push_str(self.strs[sind as usize])}
                Mode::Read => {
                    let targ = self.strs[sind as usize];
                    let s = self.s();
                    if self.buf[s..].len() < targ.len()
                        { self.fail(); return; }
                    if &self.buf[s..s+targ.len()] != targ
                        { self.fail(); return; }
                    self.cs(targ.len());
                }
            }
            Instr::Proceed => {
                self.pop_frame()
            }
            Instr::Invoke(ind) => {
                self.new_frame(self.code[ind as usize]);
            }
            Instr::PrepScan(ind) => {
                self.new_scan(ind);
            }
            Instr::DropScan => {
                if self.s() != self.buf.len()
                    {self.fail();}
                else
                    {self.pop_scan();}
            }
            Instr::Mode(mode) => {
                self.scans.last_mut().unwrap().2 = mode;
            },
            Instr::Skip(x) => {
                let last = self.frames.last_mut().unwrap();
                last.0 += x as usize;
            }
            Instr::Trap => {
                panic!("It's A Trap !");
            }
            _=>()
        }
    }
    fn run(&mut self) {
        while !self.finished() {
            self.step()
        }
    }
}

fn main() {
    let source = vec![
        Elem::Text("testo bedano "),
        Elem::AttrDef("dave", vec![
            Elem::Text("dave was here"),
        ]),
        Elem::AttrDef("test", vec![
            Elem::Text("testo"),
        ]),
        Elem::Cond("test", vec![
            Elem::Text("testoa"),
        ], vec![
            Elem::Attr("dave"),
            Elem::Text(" party"),
        ], true),
    ];
    println!("prepping to compile");
    let mut compiler = Compiler::new();
    compiler.prep(source);
    while !compiler.done() {
        match compiler.run() {
            Ok(()) => {break;},
            Err(name) => {
                println!("\x1b[91m[ERROR] ignoring redef of `{name}`\x1b[0m")
            }
        }
    }
    let (code, strs, attrs) = compiler.to_exec();
    println!("done compiling");
    println!("===== disasm");
    println!("_start:");
    for instr in code[0].iter() {
        println!("\t{instr:?}");
    }
    for (attr, &ind) in attrs.iter() {
        println!("attr@{ind}({attr}):");
        for instr in code[ind as usize].iter() {
            println!("\t{instr:?}");
        }
    }
    println!("===== exec");
    let code_map = code.iter().map(|x| &x[..]).collect::<Vec<_>>();
    let mut machine = Machine::new(&code_map[..], &strs[..]);
    machine.run();
    println!("got:\n{:?}", machine.buf);
}
