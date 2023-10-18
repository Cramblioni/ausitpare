type BufInd = usize;
type InstrRel = u16;
type InstrInd = usize;
type StrInd = u32;
type CodeInd = u32;

#[derive(Debug, Clone, Copy)]
enum Instr {
    PutStr(StrInd),
    Invoke(CodeInd),
    Proceed,
    
    Mode(Mode),
    PrepScan(InstrRel),
    DropScan,
    Skip(InstrRel),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Mode{ Read, Write }

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
    /* <!-- attrib _0 : [_1=testo] abba [/_1=][#_1#][#_2#]-->
     * <!-- attrib _1 : [_2!=]testo[/_2!=] -->
     * <!-- attrib _2 : testo -->
     */
    //
    let strs = [
        "testo",
        " abba ",
        "testo",
    ];
    let attrs = [vec![
        Instr::PrepScan(5),
        Instr::Invoke(1),
        Instr::Mode(Mode::Read),
        Instr::PutStr(0),
        Instr::DropScan,
        Instr::PutStr(1),
        Instr::Invoke(1),
        Instr::Invoke(2),
        Instr::Proceed
    ], vec![
        Instr::PrepScan(4),
        Instr::Invoke(2),
        Instr::Mode(Mode::Read),
        Instr::DropScan,
        Instr::Skip(1),
        Instr::PutStr(2),
        Instr::Proceed,
    ], vec![
        Instr::PutStr(0),
        Instr::Proceed
    ]];
    let code = attrs.iter().map(|x|&x[..]).collect::<Vec<_>>();
    println!("Hello, world!");
    let mut machine = Machine::new(&code[..], &strs);
    machine.run();
    println!("{:?}", machine.buf);
}
