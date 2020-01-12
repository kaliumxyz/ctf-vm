use crate::vm::State;
use crate::vm::BoxResult;
use crate::debug::Meta;

use std::fs;
use std::io;
use std::io::Write;

#[derive(PartialEq, Clone)]
pub enum Command {
    Save(String),
    // Load(String),
    Help,
    Run,
    Step(usize),
    DebugSet(bool),
    DebugGet,
    RegisterSet(usize, u16),
    RegisterGet,
    RegisterGetN(usize),
    StackSet(usize, u16),
    StackGet,
    BreakPointOpSet(u8),
    BreakPointOpGet,
    StackGetN(usize),
    Null,
    Noop,
    PrintMemory,
    PrintMemoryRange(usize,usize),
    PrintMemoryX(usize),
    Halt,
}

pub fn debugger(state: &mut State, meta: &mut Meta) -> BoxResult<()>  {
    println!("[IP] at {}", state.ip);
    for counter in meta.counters.clone() {
        println!(" {}", counter);
    }
    loop {
        print!("DEBUG> ");
        io::stdout().flush()?; // flushing to ensure that DEBUG> gets printed before the read_line
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        let mut command = match lex(input) {
            Ok(command) => command,
            Err(error) => {
                eprintln!("{}", error);
                Command::Noop
            }
        };

        if command == Command::Null {
            command = meta.last.clone();
        } else {
            meta.last = command.clone();
        }

        match command {
            Command::Run => {
                break;
            }
            Command::Noop => {
            }
            Command::PrintMemory => {
                for n in state.program.clone() {
                    println!("{}", n);
                }
            }
            Command::PrintMemoryRange(mut n, m) => {
                loop {
                    if n > m {
                        break;
                    }
                    println!("{}", state.program[n]);
                    n = n + 1;
                }
            }
            Command::PrintMemoryX(m) => {
                let mut n = state.ip;
                loop {
                    if n > m {
                        break;
                    }
                    println!("{}", state.program[n]);
                    n = n + 1;
                }
            }
            Command::BreakPointOpSet(op) => {
                meta.break_op = op;
                println!("DEBUG: {}", meta.break_op);
            }
            Command::BreakPointOpGet => {
                println!("DEBUG: {}", meta.break_op);
            }
            Command::DebugSet(value) => {
                meta.debug = value;
                println!("DEBUG: {}", meta.debug);
            }
            Command::DebugGet => {
                println!("DEBUG: {}", meta.debug);
            }
            Command::RegisterSet(register, value) => {
                state.register[register] = value;
                println!("DEBUG: [{}] = {}", register, value);
            }
            Command::RegisterGet => {
                println!("DEBUG: {:?}", state.register);
            }
            Command::RegisterGetN(register) => {
                println!("DEBUG: [{}]: {}", register, state.register[register]);
            }
            Command::Help => {
                println!("DEBUG: What are you asking me for? Read the source code!");
            }
            Command::Step(n) => {
                if n > 0 {
                    println!("stepping {}", n);
                    meta.debugging = true;
                } else {
                    println!("step");
                    meta.debugging = true;
                }
                //         meta.counters.push(100);
                return Ok(())
            }
            Command::Save(path) => {
                println!("saving program to {}", path);
                fs::write(path, State::save(state))?;
                println!("dumped!");
            }
            Command::StackGet => {
                println!("DEBUG: {:?}", state.stack);
            }
            Command::StackSet(index, value) => {
                state.stack[index] = value;
                println!("DEBUG: {:?}", state.stack);
            }
            Command::StackGetN(index) => {
                println!("DEBUG: {:?}", state.stack[index]);
            }
            Command::Null => {
            }
            Command::Halt => {
                meta.halt = !meta.halt;
                println!("DEBUG: halt set to: {}", meta.halt);
            }
        }
    }
    Ok(())
}

fn lex(line: String) -> BoxResult<Command> {
    use std::io::{Error, ErrorKind};

    let mut argv = line.split_whitespace();

    if let Some(command) = argv.next() {
        match command {
            "n" | "step" => {
                if let Some(arg) = argv.next() {
                    let n = arg.parse::<usize>()?;
                    Ok(Command::Step(n))
                } else {
                    Ok(Command::Step(0))
                }
            }
            "m" | "memory" => {
                if let Some(arg) = argv.next() {
                    let n = arg.parse::<usize>()?;
                    if let Some(arg2) = argv.next() {
                        let m = arg2.parse::<usize>()?;
                        Ok(Command::PrintMemoryRange(n, m))
                    } else {
                        Ok(Command::PrintMemoryX(n))
                    }
                } else {
                    Ok(Command::PrintMemory)
                }
            }
            "s" | "stack" => {
                if let Some(arg) = argv.next() {
                    let n = arg.parse::<usize>()?;
                    Ok(Command::StackGetN(n))
                } else {
                    Ok(Command::StackGet)
                }
            }
            "run" => {
                Ok(Command::Run)
            }
            "fuck" => {
                println!("command fuck not given");
                Ok(Command::Noop)
            }
            "op" | "bp" | "bp op" => {
                if let Some(arg) = argv.next() {
                    let n = arg.parse::<u8>()?;
                    Ok(Command::BreakPointOpSet(n))
                } else {
                    Ok(Command::BreakPointOpGet)
                }
            }
            "r" | "register" => {
                if let Some(register) = argv.next() {
                    let register = register.parse::<usize>()?;

                    if register > 7 {
                        return Err(Box::new(Error::new(ErrorKind::InvalidInput, format!("We only have 8 registers, thats 0 to 7"))))
                    }
                    if let Some(value) = argv.next() {
                        let value = value.parse::<u16>()?;
                        Ok(Command::RegisterSet(register, value))
                    } else {
                        Ok(Command::RegisterGetN(register))
                    }
                } else {
                    Ok(Command::RegisterGet)
                }
            }
            "debug" => {
                if let Some(status) = argv.next() {
                    match status {
                        "true" | "1" | "t" | "True" | "on" | "On" => {
                            Ok(Command::DebugSet(true))
                        }
                        "false" | "0" | "f" | "False" | "off" | "Off" => {
                            Ok(Command::DebugSet(false))
                        }
                        _ => Err(Box::new(Error::new(ErrorKind::NotFound, format!("value {} is invalid, must be boolean", status))))
                    }
                } else {
                    Ok(Command::DebugGet)
                }
            }
            "help" | "man" | "?" => {
                Ok(Command::Help)
            }
            "halt" => {
                Ok(Command::Halt)
            }
            "save" => {
                if let Some(path) = argv.next() {
                    Ok(Command::Save(String::from(path)))
                } else {
                    Ok(Command::Save(String::from("./out")))
                }
            }
            _ => Err(Box::new(Error::new(ErrorKind::NotFound, format!("command {} not found", command))))
        }
    } else {
        Ok(Command::Null)
    }
}
