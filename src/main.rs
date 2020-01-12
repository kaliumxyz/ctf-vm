use crate::debug::debugger::debugger;
use std::env;
use std::error::Error;
use std::fs;

use vm::State;
use debug::Meta;

use crate::util::*;
use crate::error::*;
mod vm;
mod opcode;
mod util;
mod debug;
mod error;

/***
 * TODO:
 *     - pausing the VM from anywhere in the code
 *     - seperating the parsing from the execution
 *     - implementing an ABI or some other API to allow for seperate debugger
 *       processes.
 *     - taking snapshots at any state in the code.
 *     - snapshots which don't mutate program memory.
 *     - proper breakpoints for both points in the program, specific operations
 *       and register access.
 *     - seperate the debug printing from the operation execution.
 *     - add stepping.
 *     - add GUI or TUI to allow for rendering the stack and registers while
 *       stepping trough the code.
 *
 ***/

struct Config {
    quiet: bool,
    debug: bool,
    path: String,
}

type BoxResult<T> = Result<T, Box<dyn Error>>;

fn main() -> BoxResult<()> {
    let args: Vec<String> = env::args().collect();

    if args.len() == 1 {
        println!("USAGE: {} [OPTIONS] [FILE]", args[0]);
        println!("-d: start with debug mode on");
        return Ok(());
    }

    let mut config = Config {
        quiet: false,
        debug: false,
        path: String::new(),
    };

    if args.len() == 2 {
        config.path = args[1].clone();
    } else {
        for i in 1..args.len() {
            match args[i].as_ref() {
                "-d" | "--debug" => {
                    config.debug = true;
                }
                "-q" | "--quiet" => {
                    config.quiet = true;
                }
                file => {
                    if config.path == "" {
                        config.path = file.to_owned();
                    } else {
                        return Err(InvalidArgError::new(format!("unknown argument {}", file)));
                    }
                }
            }
        }
    }

    let program = load(&config)?;
    run(program, &config)?;

    Ok(())
}

fn load(config: &Config) -> BoxResult<Vec<u8>> {
    if config.quiet {
        println!("reading: {}", config.path);
    }
    let program = fs::read(&config.path)?;
    if config.quiet {
        println!("running: {}", config.path);
    }
    return Ok(program);
}

fn run(program: Vec<u8>, config: &Config) -> BoxResult<()> {
    let mut state = State::recover(program)?;

    let mut meta = Meta::new();

    let mut counter = 100000000;
    let mut read_counter = 0;

    meta.debug = config.debug;

    loop {
        if state.sp == 1027 {
            println!("sp hit 1027");
            println!("instructions completed {}", meta.op_count);
            println!("[IP] at {}", state.ip);
            let mut i = 0;
            loop {
                println!("[{}] = {}", i, state.register[i]);
                i = i + 1;
                if i > 8 {
                    break;
                }
            }
            let mut i = 0;
            loop {
                println!("<{}> = {}", i, state.stack[i]);
                i = i + 1;
                if i > 1027 {
                    break;
                }
            }
            debugger(&mut state, &mut meta)?;
        }
        if read_counter > 0 {
            read_counter = read_counter + 1;
        }
        if counter == 0 || read_counter > 2000000 {
            debugger(&mut state, &mut meta)?;
        } else {
            if counter > 0 {
                counter = counter - 1;
            }
        }
        meta.op_count = meta.op_count + 1;

        if meta.debug {
            println!("{} {} {}", state.ip, meta.op_count, read_counter);
        }
        if meta.break_op == state.program[state.ip as usize] {
            counter = 0;
            continue;
        }

        if meta.debug {
            println!("{:?}", opcode::parse(&state.program, &state.ip));
        }

        // if we want, run the opcode;
        opcode::execute(&mut state, &mut meta);

        if meta.halt == true {
            break;
        }
    }

    Ok(())
}

/// opcode 0: HALT
pub fn halt(state: &State, meta: &mut Meta) {
    let register = state.register;
    let stack = state.stack.clone();
    let ip = state.ip;
    if meta.debug {
        println!("opcode 0: HALT");
    }
    println!("instructions completed {}", meta.op_count);
    println!("IP at {} {:x}", ip, ip);
    let mut i = 0;
    loop {
        println!("register {}: {} {:x}", i, register[i], register[i]);
        i = i + 1;
        if i > 7 {
            break;
        }
    }
    let mut i = 0;
    loop {
        println!("stack {}: {} {:x}", i, stack[i], stack[i]);
        i = i + 1;
        if i > 10 {
            break;
        }
    }
    meta.halt = true;
}
