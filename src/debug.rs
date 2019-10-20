use std::fmt;

pub struct Meta {
    pub op_count: usize,
}

impl Meta {
    pub fn new() -> Meta {
        return Meta{
            op_count: 0
        }
    }
    pub fn recover(op_count: usize) -> Meta {
        return Meta {
            op_count
        }
    }
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
