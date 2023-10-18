/*
Machine:
	scanning: [ScanFrame]
	buffer: [u8]

ScanFrame:
	f: ind		// Fail
	s: ind		// Scan
	m: Mode		// Mode


[john=john]magic [/john=]shit
mode write
prepscan *fail
invoke john
mode read
putstr	"john"
dropscan
putstr "magic "
*fail
putstr "shit"

[john!=john]magic [/john!=]shit
mode write
prepscan *fail
invoke john
mode read
putstr	"john"
dropscan
goto *after
*fail
putstr "magic "
*after
putstr "shit"
*/

type Ind = usize;
#[derive(Clone, Copy, Debug)]
enum Mode { Read, Write }

struct ScanFrame {
    s: Ind,     // Scan
    f: Ind,     // Fail
    m: Mode,    // Mode
    r: Ind,     // Return
}

struct Machine<'mig> {
    scans: Vec<ScanFrame>,
    buff: String,
    strs: &'mig [&'mig str]
}
impl<'m> Machine<'m> {
    fn get_mode(&self) -> Mode {
        self.scans.last()
            .map(|x|x.m)
            .unwrap_or(Mode::Write)
    }
    fn prep_scan(&mut self, fail: Ind) {
        self.scans.push(ScanFrame{
            f: fail,
            s: self.buff.len(),
            r: self.buff.len(),
            m: Mode::Write
        })
    }
    fn drop_scan(&mut self) {
        let res = self.scans.pop();
        self.buff.truncate(res.r);
    }
    fn put_str(&mut self, sind: Ind) {
        match self.get_mode() {
            Mode::Write => 
                { self.buff.push_str(self.strs[sind as usize]) }
            Mode::Read => {
                
            }
        }
    }
}

fn main() {
    let strs = [
        "john",
        "john",
        "party ",
        "boi"
    ];
    let mut machine = Machine {
        scans: Vec::new(),
        buff: String::with_capacity(1024),
        strs: &strs
    };
}
