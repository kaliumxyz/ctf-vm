use std::error::Error;
use std::fmt;

type BoxResult<T> = Result<T,Box<dyn Error>>;

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
    pub sp: usize,
    pub ip: usize,
    pub stack: Vec<u16>,
}

impl State {
    pub fn new(program: Vec<u8>) -> State {
        return State {
            program,
            register: [0;8],
            sp: 0,
            ip: 0,
            stack: Vec::new(),
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
        let mut header: Vec<u8> = save.drain(0..10000).collect();

        let mut state = State::new(save);

        header.remove(0); // remove 0x17

        for i in 0..7 { // load the registers
            let n = i * 2;
            state.register[i] = to_u16(header[n], header[n + 1]);
        }
        header.drain(0..15);
        for i in 0..1027 { // load the stack
            let n = i * 2;
            state.stack[i] = to_u16(header[n], header[n + 1]);
        }
        header.drain(0..2055);
        state.sp = to_u16(header[0], header[1]) as usize;
        header.drain(0..1);
        state.ip = to_u16(header[0], header[1]) as usize;

        Ok(state)
    }

    pub fn save(state: State) -> Vec<u8> {
        let mut program = state.program;
        let mut save:Vec<u8> = Vec::new();

        save.push(0x17); // if 23 is encountered, we know its a save file, 22 is legacy
        save.push((state.sp >> 8)  as u8);
        save.push(state.sp as u8);
        save.push((state.ip >> 8)  as u8);
        save.push(state.ip as u8);
        for i in 0..7 { // save the registers
            save.push((state.register[i] >> 8) as u8);
            save.push(state.register[i] as u8);
        }
        for i in 0..1027 { // save the stack
            save.push((state.stack[i] >> 8) as u8);
            save.push(state.stack[i] as u8);
        }
        if save.len() < 10000 {
            save.append(&mut Vec::<u8>::with_capacity(10000 - save.len()));
        }
        save.append(&mut program);
        return save;
    }
}

fn recover_legacy(program: Vec<u8>) -> BoxResult<State> {
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
    for i in 0..99 { // load the registers
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
    state.sp = value as usize;
    let higher = program[ip] as u16;
    let lower = program[ip + 1] as u16;
    let value: u16 = higher << 8 | lower;
    state.ip = value as usize;

    Ok(state)
}

fn to_u16 (higher: u8, lower: u8) -> u16 {
    return (higher as u16) << 8 | lower as u16;
}
