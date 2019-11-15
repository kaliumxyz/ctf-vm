use crate::vm::State;

pub fn to_u16 (higher: u8, lower: u8) -> u16 {
    return (higher as u16) << 8 | lower as u16;
}

pub fn read_argument(state: &State) -> u16 {
    let higher = state.program[state.ip + 1] as u16;
    let lower = state.program[state.ip] as u16;

    let mut argument: u16 = higher << 8 | lower;
    if state.debug {
        println!("read_argument found number {}", argument);
    }
    while argument > 32767 {
        let index = argument as usize;
        if state.debug {
            println!("               reguested contents of [{}]", index % 32768);
        }
        if index % 32768 < 8 {
            argument = state.register[index % 32768];
        }
        if state.debug {
            println!("                    content {}", argument);
        }
    }
    return argument;
}

pub fn read_x(state: &State, x: usize) -> u16 {
    let higher = state.program[x + 1] as u16;
    let lower = state.program[x] as u16;

    let mut argument: u16 = higher << 8 | lower;
    if state.debug {
        println!("read_argument found number {}", argument);
    }
    while argument > 32767 {
        let index = argument as usize;
        if state.debug {
            println!("               reguested contents of [{}]", index % 32768);
        }
        if index % 32768 < 8 {
            argument = state.register[index % 32768];
        }
        if state.debug {
            println!("                    content {}", argument);
        }
    }
    return argument;
}

pub fn write_argument(state: &State) -> u16 {
    let higher = state.program[state.ip + 1] as u16;
    let lower = state.program[state.ip] as u16;

    let mut argument: u16 = higher << 8 | lower;
    if state.debug {
        println!("write_argument found number {}", argument);
    }
    if argument > 32767 {
        argument = argument % 32768;
        if state.debug {
            println!("                request is to register [{}]", argument);
        }
    }
    if argument > 7 {
        println!(" using special register [8], request was for {}", argument);
        argument = 8;
    }
    return argument;
}

pub fn read() -> u8 {
    use std::io::{stdin, Read};

    let stdin = stdin();

    let mut input = stdin.lock();
    let mut reader: [u8; 1] = [0; 1];
    if let Ok(_) = input.read_exact(&mut reader) {
        return reader[0];
    } else {
        return b'~';
    }
}
