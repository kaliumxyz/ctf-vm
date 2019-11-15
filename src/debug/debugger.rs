use crate::vm::State;
use crate::vm::BoxResult;
use crate::debug::Meta;

use std::fs;
use std::io;
use std::io::Write;

enum Command {
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
    StackGetN(usize),
    Null,
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
        let command = match lex(input) {
            Ok(command) => command,
            Err(error) => {
                eprintln!("{}", error);
                Command::Null
            }
        };
        match command {
            Command::Run => {
                break;
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
                } else {
                    println!("step");
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
            Command::Null => {}
        }
    }
    Ok(())
    // if input.starts_with("n") || input.starts_with("step") {
    //     let mut argv = input.split_ascii_whitespace();
    //     argv.next();
    //     if let Some(i) = argv.next() {
    //         let i = if let Ok(i) = i.parse::<isize>() {
    //             i
    //         } else {
    //             return Ok(());
    //         };
    //     }
    // }
    // if input.starts_with("breakpoint") {
    //     let mut argv = input.split_ascii_whitespace();
    //     argv.next();
    //     if let Some(i) = argv.next() {
    //         if i.starts_with("op") {
    //             argv.next();
    //             if let Some(i) = argv.next() {
    //                 if let Ok(i) = i.parse::<isize>() {
    //                     meta.break_op = i as u8;
    //                 } else {
    //                     return Ok(());
    //                 };
    //             }
    //         }
    //         if let Ok(i) = i.parse::<usize>() {
    //             meta.breakpoints.push(i);
    //         } else {
    //             return Ok(());
    //         };
    //         println!("breakpoint set at memory address {}", i);
    //     } else {
    //         println!("Useage: breakpoint [memory address | opcode]");
    //     }
    //     return Ok(());
    // }
    // if input == "debug\n" {
    //     println!("debug set");
    //     meta.debug = !meta.debug;
    //     return Ok(());
    // }
    // if input.starts_with("dump") {
    //     let mut argv = input.split_ascii_whitespace();
    //     argv.next();
    //     if let Some(file) = argv.next() {
    //         println!("dumping program to {}", file);
    //         if let Ok(_) = fs::write(file, &state.program) {
    //             println!("dumped!");
    //         }
    //     } else {
    //         println!("dumping program");
    //         if let Ok(_) = fs::write("./out", &state.program) {
    //             println!("dumped!");
    //         }
    //     }
    //     return Ok(());
    // }
    // if input.starts_with("info") {
    //     println!("instructions completed {}", meta.op_count);
    //     println!("IP at {} {:x}", state.ip, state.ip);
    //     let mut i = 0;
    //     loop {
    //         println!("state.register {}: {} {:x}", i, state.register[i], state.register[i]);
    //         i = i + 1;
    //         if i > 7 {
    //             break;
    //         }
    //     }
    //     let mut i = 0;
    //     loop {
    //         println!("stack {}: {} {:x}", i, state.stack[i], state.stack[i]);
    //         i = i + 1;
    //         if i > 10 {
    //             break;
    //         }
    //     }
    //     return Ok(());
    // }
    // if input.starts_with("s") {
    //     let mut argv = input.split_ascii_whitespace();
    //     argv.next();
    //     if let Some(i) = argv.next() {
    //         let mut i: usize = if let Ok(i) = i.parse::<usize>() {
    //             i
    //         } else {
    //             return Ok(());
    //         };
    //         println!("<{}> = {}", i, state.stack[i]);
    //         if let Some(value) = argv.next() {
    //             let value: u16 = if let Ok(i) = value.parse::<u16>() {
    //                 i
    //             } else {
    //                 return Ok(());
    //             };
    //             state.stack[i] = value;
    //             println!("<{}> = {}", i, state.stack[i]);
    //         }
    //         loop {
    //             println!("<{}> = {}", i, state.stack[i]);
    //             i = i - 1;
    //             if i == 0 {
    //                 break;
    //             }
    //         }
    //     } else {
    //         let mut i = 0;
    //         loop {
    //             println!("<{}> = {}", i, state.stack[i]);
    //             i = i + 1;
    //             if i > 40 {
    //                 break;
    //             }
    //         }
    //     }
    //     return Ok(());
    // }
    // if input.starts_with("r") {
    //     let mut argv = input.split_ascii_whitespace();
    //     argv.next();
    //     if let Some(i) = argv.next() {
    //         let i: usize = if let Ok(i) = i.parse::<usize>() {
    //             i
    //         } else {
    //             return Ok(());
    //         };
    //         if i > 7 {
    //             return Ok(());
    //         }
    //         println!("[{}] = {}", i, state.register[i]);
    //         if let Some(value) = argv.next() {
    //             let value: u16 = if let Ok(i) = value.parse::<u16>() {
    //                 i
    //             } else {
    //                 return Ok(());
    //             };
    //             state.register[i] = value;
    //             println!("[{}] = {}", i, state.register[i]);
    //         }
    //     } else {
    //         let mut i = 0;
    //         loop {
    //             println!("[{}] = {}", i, state.register[i]);
    //             i = i + 1;
    //             if i > 8 {
    //                 break;
    //             }
    //         }
    //     }
    //     return Ok(());
    // }
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
                        "true" | "1" | "t" | "True" => {
                            Ok(Command::DebugSet(true))
                        }
                        "false" | "0" | "f" | "False" => {
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
