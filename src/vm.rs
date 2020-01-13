use std::error::Error;
use std::fmt;
use crate::util::to_u16;

pub type BoxResult<T> = Result<T,Box<dyn Error>>;

struct RecoveryError {
    details: String
}

impl RecoveryError {
    fn new(msg: String) -> Box<RecoveryError> {
        Box::new(RecoveryError{details: msg})
    }
}

impl fmt::Debug for RecoveryError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self.details)
    }
}

impl fmt::Display for RecoveryError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.details)
    }
}

impl Error for RecoveryError {
    fn description(&self) -> &str {
        &self.details
    }
}

pub struct State {
    pub program: Vec<u8>,
    pub register:[u16;8],
    pub ip: usize,
    pub stack: Vec<u16>,
    pub debug: bool,
}

impl State {
    pub fn new(program: Vec<u8>) -> State {
        return State {
            program,
            register: [0;8],
            ip: 0,
            stack: Vec::new(),
            debug: false,
        }
    }

    pub fn recover(mut save: Vec<u8>) -> BoxResult<State> {
        match save[0] {
            0x00..=0x15 => { // regular program?
                return Ok(State::new(save));
            },
            0x16 => {
                return recover_legacy(save);
            },
            0x17 => {
                // continue
            },
            op => {
                return Err(RecoveryError::new(format!("This does not seem to be a valid file, invalid starting code {}", op)));
            }
        }
        println!("recovering");
        let mut header: Vec<u8> = save.drain(0..21).collect();


        header.remove(0); // remove 0x17

        let sp = to_u16(header[0], header[1]) as usize;
        header.drain(0..2);
        let mut stack: Vec<u8> = save.drain(0..(sp * 2)).collect();
        let mut state = State::new(save);
        state.ip = to_u16(header[0], header[1]) as usize;
        header.drain(0..2);

        for i in 0..8 { // load the registers
            let n = i * 2;
            state.register[i] = to_u16(header[n], header[n + 1]);
        }

        header.drain(0..16);

        for i in 0..sp { // load the stack
            let n = i * 2;
            state.stack.push(to_u16(stack[n], stack[n + 1]))
        }

        Ok(state)
    }

    pub fn save(state: &State) -> Vec<u8> {
        let mut save:Vec<u8> = Vec::new();

        save.push(0x17); // if 23 is encountered, we know its a save file, 22 is legacy
        save.push((state.stack.len() >> 8)  as u8);
        save.push(state.stack.len() as u8);
        save.push((state.ip >> 8)  as u8);
        save.push(state.ip as u8);
        for i in 0..8 { // save the registers
            save.push((state.register[i] >> 8) as u8);
            save.push(state.register[i] as u8);
        }
        for i in 0..state.stack.len() { // save the stack
            save.push((state.stack[i] >> 8) as u8);
            save.push(state.stack[i] as u8);
        }
        save.append(&mut state.program.clone());
        return save;
    }
}

fn recover_legacy(program: Vec<u8>) -> BoxResult<State> {
    println!("legacy recovery");
    let mut ip = 1;
    let mut state = State::new(program.clone());
    for i in 0..7 { // load the registers
        let n = i * 2;
        let higher = program[ip + n + 1] as u16;
        let lower = program[ip + n] as u16;
        let value: u16 = higher << 8 | lower;
        state.register[i] = value;
    }
    ip = ip + 16;
    for i in 0..99 { // load the stack
        let n = i * 2;
        let higher = program[ip + n + 1] as u16;
        let lower = program[ip + n] as u16;
        let value: u16 = higher << 8 | lower;
        state.stack.push(value);
    }
    ip = ip + 100;
    let higher = program[ip] as u16;
    let lower = program[ip + 1] as u16;
    let value: u16 = higher << 8 | lower;
    ip = ip + 2;
    let sp = value as usize;
    for _i in sp..99 {
        state.stack.pop();
    }
    let higher = program[ip] as u16;
    let lower = program[ip + 1] as u16;
    let value: u16 = higher << 8 | lower;
    state.ip = value as usize;

    Ok(state)
}
