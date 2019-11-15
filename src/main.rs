use crate::debug::debugger::debugger;
use std::env;
use std::error::Error;
use std::fs;

use vm::State;
use debug::Meta;

use crate::util::*;
use crate::error::*;
mod vm;
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
    // registers

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
        match state.program[state.ip as usize] {
            0 => halt(&state, &mut meta),
            1 => {
                if meta.debug {
                    println!("opcode 1: SET [A] TO B");
                    println!(" A: STATE.REGISTER");
                }
                state.ip = state.ip + 2;
                let a = write_argument(&state) as usize;
                if meta.debug {
                    println!(" B: INTEGER");
                }
                state.ip = state.ip + 2;
                let b: u16 = read_argument(&state);

                state.register[a] = b;

                state.ip = state.ip + 2;
                if meta.debug {
                    println!(" RESULT:  [A{}] = B{}", a, b);
                    println!("          [A{}] = {}", a, state.register[a]);
                    println!("");
                    println!(" [IP A{}]", a);
                }
            }
            2 => {
                if meta.debug {
                    println!("opcode 2: PUSH TO STACK FROM [A]");
                    println!(" A: STATE.REGISTER");
                }
                state.ip = state.ip + 2;
                let a: u16 = read_argument(&state);
                let b: u16 = write_argument(&state);

                state.stack.push(a);
                state.sp = state.sp + 1;

                state.ip = state.ip + 2;
                if meta.debug {
                    println!(" RESULT:  <{}> = [A{}]", state.sp, b);
                    println!("          <{}> = {}", state.sp, a);
                    println!("          <{}> = {}", state.sp, state.stack[state.sp - 1]);
                    println!(" [SP IP] <{}>", state.sp);
                }
            }
            3 => {
                if meta.debug {
                    println!("opcode 3: POP FROM STACK TO [A]");
                    println!(" A: STATE.REGISTER");
                }
                state.ip = state.ip + 2;
                let a = write_argument(&state) as usize;

                state.sp = state.sp - 1;
                if let Some(data) = state.stack.pop() {
                    state.register[a] = data;
                } else {
                    // halt
                }

                state.ip = state.ip + 2;
                if meta.debug {
                    println!(" RESULT:  [A{}] = <{}>", a, state.sp);
                    println!("          [A{}] = {}", a, state.register[a]);
                    println!(" [IP SP A{}] <{}>", a, state.sp);
                }
            }
            4 => {
                if meta.debug {
                    println!("opcode 4: IF B EQUALS C SET A TO 1 ELSE A TO 0");
                    println!(" A: STATE.REGISTER");
                }
                state.ip = state.ip + 2;
                let a = write_argument(&state) as usize;

                if meta.debug {
                    println!(" B: INTEGER");
                }
                state.ip = state.ip + 2;
                let b: u16 = read_argument(&state);

                if meta.debug {
                    println!(" C: INTEGER");
                }
                state.ip = state.ip + 2;
                let c: u16 = read_argument(&state);

                if b == c {
                    state.register[a] = 1;
                } else {
                    state.register[a] = 0;
                }

                state.ip = state.ip + 2;
                if meta.debug {
                    println!(" RESULT:  [A{}] = B{} == C{}", a, b, c);
                    println!("          [A{}] = {}", a, state.register[a]);
                    println!("");
                    println!(" [{} IP]", a);
                }
            }
            5 => {
                //3 args, should increment ip by 8
                if meta.debug {
                    println!("opcode 5: IF B LARGER THAN C SET A TO 1 ELSE A TO 0");
                    println!(" A: STATE.REGISTER");
                }
                state.ip = state.ip + 2;
                let a = write_argument(&state) as usize;

                if meta.debug {
                    println!(" B: INTEGER");
                }
                state.ip = state.ip + 2;
                let b: u16 = read_argument(&state);

                if meta.debug {
                    println!(" C: INTEGER");
                }
                state.ip = state.ip + 2;
                let c: u16 = read_argument(&state);

                if b > c {
                    state.register[a] = 1;
                } else {
                    state.register[a] = 0;
                }

                state.ip = state.ip + 2;
                if meta.debug {
                    println!(" RESULT:  [A{}] = B{} > C{}", a, b, c);
                    println!("          [A{}] = {}", a, state.register[a]);
                    println!("");
                    println!("[{} IP]", a);
                }
            }
            6 => {
                if meta.debug {
                    println!("opcode 6: JUMP");
                    println!(" A: JUMP ADDRESS");
                }
                state.ip = state.ip + 2;
                let a = read_argument(&state) as usize;

                state.ip = a * 2;

                if meta.debug {
                    println!(" RESULT:  [IP] = &{}", a * 2);
                    println!("          [IP] = &{}", state.ip);
                    println!("");
                    println!(" [IP]");
                }
            }
            7 => {
                if meta.debug {
                    println!("opcode 7: JUMP IF NONZERO");
                    println!(" A: CONDITIONAL");
                }
                state.ip = state.ip + 2;
                let a: u16 = read_argument(&state);

                if meta.debug {
                    println!(" B: JUMP ADDRESS");
                }
                state.ip = state.ip + 2;
                let b = read_argument(&state) as usize;

                if a != 0 {
                    state.ip = b * 2;
                } else {
                    state.ip = state.ip + 2;
                }

                if meta.debug {
                    println!(" RESULT:  A{} != 0", b * 2);
                    println!("          [IP] = B{}", b * 2);
                    println!("          [IP] == {}", state.ip);
                    println!(" [IP]");
                }
            }
            8 => {
                if meta.debug {
                    println!("opcode 8: JUMP IF ZERO");
                    println!(" A: CONDITIONAL");
                }
                state.ip = state.ip + 2;
                let a: u16 = read_argument(&state);

                if meta.debug {
                    println!(" B: JUMP ADDRESS");
                }
                state.ip = state.ip + 2;
                let b = read_argument(&state) as usize;

                if a == 0 {
                    state.ip = b * 2;
                } else {
                    state.ip = state.ip + 2;
                }

                if meta.debug {
                    println!(" RESULT:  A{} == 0", a);
                    println!("          [IP] = B{}", b * 2);
                    println!("          [IP] == {}", state.ip);
                    println!(" [IP]");
                }
            }
            9 => {
                if meta.debug {
                    println!("opcode 9: ADD SET [A] RESULT B + C");
                    println!(" A: STATE.REGISTER");
                }
                state.ip = state.ip + 2;
                let a = write_argument(&state) as usize;

                if meta.debug {
                    println!(" B: INTEGER");
                }
                state.ip = state.ip + 2;
                let b: u16 = read_argument(&state);

                if meta.debug {
                    println!(" C: INTEGER");
                }
                state.ip = state.ip + 2;
                let c: u16 = read_argument(&state);

                state.register[a] = (b + c) % 32768;

                state.ip = state.ip + 2;
                if meta.debug {
                    println!(" RESULT:  B{} + C{} = {}", b, c, (b + c) % 32768);
                    println!("          [A{}] = {}", a, state.register[a]);
                    println!("");
                    println!(" [IP {}]", a);
                }
            }
            10 => {
                if meta.debug {
                    println!("opcode 10: MUTIPLY SET [A] RESULT B * C");
                    println!(" A: STATE.REGISTER");
                }
                state.ip = state.ip + 2;
                let a = write_argument(&state) as usize;

                if meta.debug {
                    println!(" B: INTEGER");
                }
                state.ip = state.ip + 2;
                let b = read_argument(&state) as usize;

                if meta.debug {
                    println!(" C: INTEGER");
                }
                state.ip = state.ip + 2;
                let c = read_argument(&state) as usize;

                state.register[a] = ((b * c) % 32768) as u16;

                state.ip = state.ip + 2;
                if meta.debug {
                    println!(" RESULT:  B{} * C{} = {}", b, c, (b * c) % 32768);
                    println!("          [A{}] = {}", a, state.register[a]);
                    println!("");
                    println!(" [IP {}]", a);
                }
            }
            11 => {
                if meta.debug {
                    println!("opcode 11: MODULO SET [A] RESULT B % C");
                    println!(" A: STATE.REGISTER");
                }
                state.ip = state.ip + 2;
                let a = write_argument(&state) as usize;

                if meta.debug {
                    println!(" B: INTEGER");
                }
                state.ip = state.ip + 2;
                let b: u16 = read_argument(&state);

                if meta.debug {
                    println!(" C: INTEGER");
                }
                state.ip = state.ip + 2;
                let c: u16 = read_argument(&state);

                state.register[a] = (b % c) % 32768;

                state.ip = state.ip + 2;
                if meta.debug {
                    println!(" RESULT:  {} % {} = {}", b, c, (b % c) % 32768);
                    println!("          [A{}] = {}", a, state.register[a]);
                    println!("");
                    println!(" [IP {}]", a);
                }
            }
            12 => {
                if meta.debug {
                    println!("opcode 12: AND SET [A] RESULT B & C");
                    println!(" A: STATE.REGISTER");
                }
                state.ip = state.ip + 2;
                let a = write_argument(&state) as usize;

                if meta.debug {
                    println!(" B: INTEGER");
                }
                state.ip = state.ip + 2;
                let b: u16 = read_argument(&state);

                if meta.debug {
                    println!(" C: INTEGER");
                }
                state.ip = state.ip + 2;
                let c: u16 = read_argument(&state);

                state.register[a] = (b & c) % 32768;

                state.ip = state.ip + 2;
                if meta.debug {
                    println!(" RESULT:  {} & {} = {}", b, c, (b & c) % 32768);
                    println!("          [A{}] = {}", a, state.register[a]);
                    println!("");
                    println!(" [IP {}]", a);
                }
            }
            13 => {
                if meta.debug {
                    println!("opcode 13: OR SET [A] RESULT B | C");
                    println!(" A: REGISTER");
                }
                state.ip = state.ip + 2;
                let a = write_argument(&state) as usize;

                if meta.debug {
                    println!(" B: INTEGER");
                }
                state.ip = state.ip + 2;
                let b: u16 = read_argument(&state);

                if meta.debug {
                    println!(" C: INTEGER");
                }
                state.ip = state.ip + 2;
                let c: u16 = read_argument(&state);

                state.register[a] = (b | c) % 32768;

                state.ip = state.ip + 2;
                if meta.debug {
                    println!(" RESULT:  {} | {} = {}", b, c, (b | c) % 32768);
                    println!("          [A{}] = {}", a, state.register[a]);
                    println!("");
                    println!(" [IP {}]", a);
                }
            }
            14 => {
                if meta.debug {
                    println!("opcode 14: NOT SET [A] RESULT !B");
                    println!(" A: REGISTER");
                }
                state.ip = state.ip + 2;
                let a = write_argument(&state) as usize;

                if meta.debug {
                    println!(" B: INTEGER");
                }
                state.ip = state.ip + 2;
                let b: u16 = read_argument(&state);

                state.register[a] = (!b) % 32768;
                state.ip = state.ip + 2;
                if meta.debug {
                    println!(" RESULT:  !{} = {}", b, (!b) % 32768);
                    println!("          [A{}] = {}", a, state.register[a]);
                    println!("");
                    println!(" [IP {}]", a);
                }
            }
            15 => {
                if meta.debug {
                    println!("opcode 15: RMEM READ TO [A] FROM &B");
                    println!(" A: REGISTER");
                }
                state.ip = state.ip + 2;
                let a = write_argument(&state) as usize;

                if meta.debug {
                    println!(" B: ADDRESS");
                }
                state.ip = state.ip + 2;
                let mut b = read_argument(&state) as usize;

                b = b * 2;

                if meta.debug {
                    println!(" &B: MEMORY AT B");
                }
                let c = read_x(&state, b);

                state.register[a] = c;

                state.ip = state.ip + 2;
                if meta.debug {
                    println!(" RESULT:  [A{}] = &{}", a, b);
                    println!("          [A{}] = {}", a, c);
                    println!("          [A{}] = {}", a, state.register[a]);
                    println!(" [IP {}]", a);
                }
            }
            16 => {
                if meta.debug {
                    println!("opcode 16: WMEM WRITE B TO &A");
                    println!(" A: ADDRESS");
                }
                state.ip = state.ip + 2;
                let mut a = read_argument(&state) as usize;
                a = a * 2;

                if meta.debug {
                    println!(" B: INTEGER");
                }
                state.ip = state.ip + 2;
                let b: u16 = read_argument(&state);

                let higher = (b >> 8) as u8;
                let lower = b as u8;

                state.program[a + 1] = higher;
                state.program[a] = lower;

                state.ip = state.ip + 2;
                if meta.debug {
                    println!(" RESULT:  [PROGRAM{}] = B{}", a, b);
                    println!("          b{:b} b{:b}", higher, lower);
                    println!(
                        "          b{:b} b{:b} = b{:b}",
                        state.program[a + 1],
                        state.program[a],
                        b
                    );
                    println!(" [IP PROGRAM]");
                }
            }
            17 => {
                if meta.debug {
                    println!("opcode 17: CALL &A");
                    println!(" A: ADDRESS");
                }
                state.ip = state.ip + 2;
                let a = read_argument(&state) as usize;

                state.ip = state.ip + 2;
                state.stack.push(state.ip as u16 / 2);
                state.sp = state.sp + 1;

                state.ip = a * 2;
                if meta.debug {
                    println!(" RESULT:  [IP{}] = A{}", state.ip, a * 2);
                    println!("          <{}> = IP{}", state.sp - 1, state.stack[state.sp - 1]);
                    println!("");
                    println!(" [IP SP]");
                }
            }
            18 => {
                if state.sp == 0 {
                    if meta.debug {
                        println!("opcode 18: return: {}", state.stack[state.sp]);
                    }
                    println!("instructions completed {}", meta.op_count);
                    println!("IP at {}", state.ip);
                    break;
                }

                state.sp = state.sp - 1;

                if meta.debug {
                    println!("opcode 18: RETURN: {}", state.stack[state.sp] * 2);
                }
                state.ip = state.stack[state.sp] as usize * 2;
                state.stack[state.sp] = 0;
            }
            19 => {
                state.ip = state.ip + 2;
                if meta.debug {
                    let a = read_argument(&state);
                    println!("opcode 19: PRINT: {}", state.program[state.ip]);
                    println!("{}", a as u8 as char);
                } else {
                    let a = read_argument(&state);
                    // eprintln!("opcode 19: PRINT: {} {}", a as u8 as char, a);
                    print!("{}", a as u8 as char);
                    // eprint!("{}", program[ip] as char);
                }
                state.ip = state.ip + 2;
            }
            20 => {
                if meta.debug {
                    println!("opcode 20: READ TO [A]");
                    println!(" A: STATE.REGISTER");
                }
                state.ip = state.ip + 2;
                let a = write_argument(&state) as usize;

                let res = read();
                if res as char == '~' {
                    counter = 0;
                    continue;
                }
                state.register[a] = res as u16;
                read_counter = 1;

                state.ip = state.ip + 2;
            }
            21 => {
                if meta.debug {
                    println!("opcode 21: NOOP");
                }
                state.ip = state.ip + 2;
            }
            22 => {
                if state.ip > 0 && meta.op_count > 1 {
                    panic!("opcode 22 encountered outside of load state")
                }
                if meta.debug {
                    println!("opcode 22: LOAD");
                }
                println!("opcode 22: LOAD");
                state.ip = state.ip + 1;
                for i in 0..7 {
                    // load the state.registers
                    let n = i * 2;
                    let higher = state.program[state.ip + n + 1] as u16;
                    let lower = state.program[state.ip + n] as u16;
                    let value: u16 = higher << 8 | lower;
                    state.register[i] = value;
                }
                state.ip = state.ip + 16;
                for i in 0..99 {
                    // load the state.registers
                    let n = i * 2;
                    let higher = state.program[state.ip + n + 1] as u16;
                    let lower = state.program[state.ip + n] as u16;
                    let value: u16 = higher << 8 | lower;
                    state.stack[i] = value;
                }
                state.ip = state.ip + 100;
                let higher = state.program[state.ip] as u16;
                let lower = state.program[state.ip + 1] as u16;
                let value: u16 = higher << 8 | lower;
                state.ip = state.ip + 2;
                state.sp = value as usize;
                let higher = state.program[state.ip] as u16;
                let lower = state.program[state.ip + 1] as u16;
                let value: u16 = higher << 8 | lower;
                state.ip = value as usize;
                if meta.debug {
                    println!("SP at {} {:x}", state.sp, state.sp);
                    println!("IP at {} {:x}", state.ip, state.ip);
                }
            }
            c => {
                println!(
                    "opcode {}: err unkown opcode at {} follows: {:x} {:x}",
                    c,
                    state.ip,
                    state.program[(state.ip + 1)],
                    state.program[(state.ip + 2)]
                );
                println!("instructions completed {}", meta.op_count);
                println!("IP at {} {:x}", state.ip, state.ip);
                let mut i = 0;
                loop {
                    println!("state.register {}: {}", i, state.register[i]);
                    i = i + 1;
                    if i > 7 {
                        break;
                    }
                }
                let mut i = 0;
                loop {
                    println!("stack {}: {}", i, state.stack[i]);
                    i = i + 1;
                    if i > 10 {
                        break;
                    }
                }
                // println!("dumping program");
                // if let Ok(_) = fs::write("./out", program) {
                //     println!("dumped!");
                // }
                break;
            }
        }
        if meta.halt == true {
            break;
        }
    }

    Ok(())
}

/// opcode 0: HALT
fn halt(state: &State, meta: &mut Meta) {
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
