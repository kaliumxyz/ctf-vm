use crate::debug::debugger::Command;
use std::fmt;
use crate::opcode::Code;

pub mod debugger;


pub struct Meta {
    pub op_count: usize,
    pub breakpoint: bool,
    pub debug: bool,
    pub pause: bool,
    pub debugging: bool,
    pub breakpoints: Vec<usize>,
    pub break_op: Code,
    pub halt: bool,
    pub last: Command,
    pub counters: Vec<usize>,
}

impl Meta {
    pub fn new() -> Meta {
        return Meta {
            op_count: 0,
            breakpoint: true,
            debugging: false,
            pause: true,
            breakpoints: Vec::new(),
            counters: Vec::new(),
            break_op: Code::Halt, // by default break on Halt
            halt: false,
            last: Command::Null,
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
