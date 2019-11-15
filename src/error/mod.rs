use std::fmt;
use std::error::Error;

pub struct InvalidArgError {
    details: String,
}

impl InvalidArgError {
    pub fn new(msg: String) -> Box<InvalidArgError> {
        Box::new(InvalidArgError { details: msg })
    }
}

impl fmt::Debug for InvalidArgError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self.details)
    }
}

impl fmt::Display for InvalidArgError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.details)
    }
}

impl Error for InvalidArgError {
    fn description(&self) -> &str {
        &self.details
    }
}
