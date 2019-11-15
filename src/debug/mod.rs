use std::fmt;

pub mod debugger;

pub struct Meta {
    pub op_count: usize,
    pub breakpoint: bool,
    pub debug: bool,
    pub breakpoints: Vec<usize>,
    pub break_op: u8,
    pub halt: bool,
    pub counters: Vec<usize>,
}

impl Meta {
    pub fn new() -> Meta {
        return Meta {
            op_count: 0,
            breakpoint: true,
            breakpoints: Vec::new(),
            counters: Vec::new(),
            break_op: 0,
            halt: false,
            debug: false,
        }
    }
    // pub fn recover(op_count: usize) -> Meta {
    //     return Meta {
    //         op_count,
    //         breakpoint: true,
    //         breakpoints: Vec::new(),
    //         counters: Vec::new(),
    //         break_op: 0,
    //         halt: false,
    //         debug: false,
    //     }
    // }
}


impl fmt::Debug for Meta {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self.op_count)
    }
}

impl fmt::Display for Meta {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.op_count)
    }
}
