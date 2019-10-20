use std::env;
use std::error::Error;
use std::fmt;
use std::fs;

use vm::State;
use debug::Meta;

mod vm;
mod util;
mod debug;

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

struct InvalidArgError {
    details: String,
}

impl InvalidArgError {
    fn new(msg: String) -> Box<InvalidArgError> {
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

struct Config {
    quiet: bool,
    debug: bool,
    path: String,
}

type BoxResult<T> = Result<T, Box<dyn Error>>;

type Deref<T> = *mut T;

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

const DEBUG: bool = false;

fn run(program: Vec<u8>, config: &Config) -> BoxResult<()> {
    // registers

    let mut state = State::recover(program)?;
    let mut ip = &mut state.ip;
    let mut sp = &mut state.sp;
    let meta = Meta::new();
    let mut ops = meta.op_count;

    let mut bp_op = 99;
    let mut bp = 999999999999;

    let stack: &mut Vec<u16> = &mut state.stack;
    let register: &mut [u16;8] = &mut state.register;

    let mut counter = 100000000;
    let mut read_counter = 0;

    let mut debug = config.debug;

    loop {
        if sp == 1027 {
            println!("sp hit 1027");
            println!("instructions completed {}", ops);
            println!("[IP] at {}", ip);
            let mut i = 0;
            loop {
                println!("[{}] = {}", i, register[i]);
                i = i + 1;
                if i > 8 {
                    break;
                }
            }
            let mut i = 0;
            loop {
                println!("<{}> = {}", i, stack[i]);
                i = i + 1;
                if i > 1027 {
                    break;
                }
            }
            break;
        }
        if read_counter > 0 {
            read_counter = read_counter + 1;
        }
        if counter == 0 || read_counter > 2000000 {
            if read_counter > 2000000 {
                println!("read counter > 20000");
                read_counter = 0;
            }
            println!("[IP] at {} {} {}", ip, read_counter, counter);
            println!("DEBUG> ");
            use std::io;
            let mut input = String::new();
            match io::stdin().read_line(&mut input) {
                Ok(_) => {
                    if input == "run\n" {
                        counter = -1;
                        continue;
                    }
                    if input.starts_with("n") {
                        let mut argv = input.split_ascii_whitespace();
                        argv.next();
                        if let Some(i) = argv.next() {
                            let i = if let Ok(i) = i.parse::<isize>() {
                                i
                            } else {
                                continue;
                            };
                            counter = i;
                            println!("stepping {}", i);
                        } else {
                            println!("stepping 100");
                            counter = 100;
                        }
                    }
                    if input.starts_with("step") {
                        let mut argv = input.split_ascii_whitespace();
                        argv.next();
                        if let Some(i) = argv.next() {
                            let i = if let Ok(i) = i.parse::<isize>() {
                                i
                            } else {
                                continue;
                            };
                            println!("stepping {}", i);
                            counter = i;
                        } else {
                            println!("stepping 100");
                            counter = 100;
                        }
                        continue;
                    }
                    if input.starts_with("breakpoint") {
                        let mut argv = input.split_ascii_whitespace();
                        argv.next();
                        if let Some(i) = argv.next() {
                            if i.starts_with("op") {
                                argv.next();
                                if let Some(i) = argv.next() {
                                    if let Ok(i) = i.parse::<isize>() {
                                        bp_op = i;
                                    } else {
                                        continue;
                                    };
                                }
                            }
                            if let Ok(i) = i.parse::<usize>() {
                                bp = i;
                            } else {
                                continue;
                            };
                            println!("breakpoint set at memory address {}", i);
                        } else {
                            println!("Useage: breakpoint [memory address | opcode]");
                        }
                        continue;
                    }
                    if input == "debug\n" {
                        println!("debug set");
                        debug = !debug;
                        continue;
                    }
                    if input.starts_with("save") {
                        let mut argv = input.split_ascii_whitespace();
                        argv.next();
                        if let Some(file) = argv.next() {
                            let mut offset = 0;
                            state.program[0] = 0x16;
                            offset = offset + 1;
                            for i in 0..7 {
                                // save the registers
                                let n = i * 2;
                                state.program[offset + n + 1] = (register[i] >> 8) as u8;
                                state.program[offset + n] = register[i] as u8;
                            }
                            offset = offset + 16;
                            for i in 0..99 {
                                // save the stack
                                let n = i * 2;
                                state.program[offset + n + 1] = (stack[i] >> 8) as u8;
                                state.program[offset + n] = stack[i] as u8;
                            }
                            offset = offset + 100;
                            state.program[offset] = (sp >> 8) as u8;
                            state.program[offset + 1] = sp as u8;
                            state.program[offset + 2] = (ip >> 8) as u8;
                            state.program[offset + 3] = ip as u8;
                            println!("saving program to {}", file);
                            if let Ok(_) = fs::write(file, &state.program) {
                                println!("dumped!");
                            }
                        } else {
                            let mut offset = 0;
                            state.program[0] = 0x16;
                            offset = offset + 1;
                            for i in 0..7 {
                                // save the registers
                                let n = i * 2;
                                state.program[offset + n + 1] = (register[i] >> 8) as u8;
                                state.program[offset + n] = register[i] as u8;
                            }
                            offset = offset + 16;
                            for i in 0..99 {
                                // save the stack
                                let n = i * 2;
                                state.program[offset + n + 1] = (stack[i] >> 8) as u8;
                                state.program[offset + n] = stack[i] as u8;
                            }
                            offset = offset + 100;
                            state.program[offset] = (sp >> 8) as u8;
                            state.program[offset + 1] = sp as u8;
                            state.program[offset + 2] = (ip >> 8) as u8;
                            state.program[offset + 3] = ip as u8;
                            println!("saving program");
                            if let Ok(_) = fs::write("./out.sav", &state.program) {
                                println!("dumped!");
                            }
                        }
                        continue;
                    }
                    if input.starts_with("dump") {
                        let mut argv = input.split_ascii_whitespace();
                        argv.next();
                        if let Some(file) = argv.next() {
                            println!("dumping program to {}", file);
                            if let Ok(_) = fs::write(file, &state.program) {
                                println!("dumped!");
                            }
                        } else {
                            println!("dumping program");
                            if let Ok(_) = fs::write("./out", &state.program) {
                                println!("dumped!");
                            }
                        }
                        continue;
                    }
                    if input.starts_with("info") {
                        println!("instructions completed {}", ops);
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
                        continue;
                    }
                    if input.starts_with("s") {
                        let mut argv = input.split_ascii_whitespace();
                        argv.next();
                        if let Some(i) = argv.next() {
                            let mut i: usize = if let Ok(i) = i.parse::<usize>() {
                                i
                            } else {
                                continue;
                            };
                            println!("<{}> = {}", i, stack[i]);
                            if let Some(value) = argv.next() {
                                let value: u16 = if let Ok(i) = value.parse::<u16>() {
                                    i
                                } else {
                                    continue;
                                };
                                stack[i] = value;
                                println!("<{}> = {}", i, stack[i]);
                            }
                            loop {
                                println!("<{}> = {}", i, stack[i]);
                                i = i - 1;
                                if i == 0 {
                                    break;
                                }
                            }
                        } else {
                            let mut i = 0;
                            loop {
                                println!("<{}> = {}", i, stack[i]);
                                i = i + 1;
                                if i > 40 {
                                    break;
                                }
                            }
                        }
                        continue;
                    }
                    if input.starts_with("r") {
                        let mut argv = input.split_ascii_whitespace();
                        argv.next();
                        if let Some(i) = argv.next() {
                            let i: usize = if let Ok(i) = i.parse::<usize>() {
                                i
                            } else {
                                continue;
                            };
                            if i > 7 {
                                continue;
                            }
                            println!("[{}] = {}", i, register[i]);
                            if let Some(value) = argv.next() {
                                let value: u16 = if let Ok(i) = value.parse::<u16>() {
                                    i
                                } else {
                                    continue;
                                };
                                register[i] = value;
                                println!("[{}] = {}", i, register[i]);
                            }
                        } else {
                            let mut i = 0;
                            loop {
                                println!("[{}] = {}", i, register[i]);
                                i = i + 1;
                                if i > 8 {
                                    break;
                                }
                            }
                        }
                        continue;
                    }
                }
                Err(_) => {}
            }
        } else {
            if counter > 0 {
                counter = counter - 1;
            }
        }
        ops = ops + 1;

        if DEBUG || debug {
            println!("{} {} {}", ip, ops, read_counter);
        }
        if ip == bp || bp_op == state.program[ip as usize] as isize {
            counter = 0;
            continue;
        }
        match state.program[ip as usize] {
            0 => {
                state.ip = ip;
                state.sp = sp;
                halt(state, meta);
                break;
            }
            1 => {
                if DEBUG || debug {
                    println!("opcode 1: SET [A] TO B");
                    println!(" A: REGISTER");
                }
                ip = ip + 2;
                let a = write_argument(&state) as usize;
                if DEBUG || debug {
                    println!(" B: INTEGER");
                }
                ip = ip + 2;
                let b: u16 = read_argument(&state);

                register[a] = b;

                ip = ip + 2;
                if DEBUG || debug {
                    println!(" RESULT:  [A{}] = B{}", a, b);
                    println!("          [A{}] = {}", a, register[a]);
                    println!("");
                    println!(" [IP A{}]", a);
                }
            }
            2 => {
                if DEBUG || debug {
                    println!("opcode 2: PUSH TO STACK FROM [A]");
                    println!(" A: REGISTER");
                }
                ip = ip + 2;
                let a: u16 = read_argument(&state);
                let b: u16 = write_argument(&state);

                stack.push(a);
                sp = sp + 1;

                ip = ip + 2;
                if DEBUG || debug {
                    println!(" RESULT:  <{}> = [A{}]", sp, b);
                    println!("          <{}> = {}", sp, a);
                    println!("          <{}> = {}", sp, stack[sp - 1]);
                    println!(" [SP IP] <{}>", sp);
                }
            }
            3 => {
                if DEBUG || debug {
                    println!("opcode 3: POP FROM STACK TO [A]");
                    println!(" A: REGISTER");
                }
                ip = ip + 2;
                let a = write_argument(&state) as usize;

                sp = sp - 1;
                if let Some(data) = stack.pop() {
                    register[a] = data;
                } else {
                    // halt
                }

                ip = ip + 2;
                if DEBUG || debug {
                    println!(" RESULT:  [A{}] = <{}>", a, sp);
                    println!("          [A{}] = {}", a, register[a]);
                    // println!("          <{}> = {}", sp, stack[sp]);
                    println!(" [IP SP A{}] <{}>", a, sp);
                }
            }
            4 => {
                if DEBUG || debug {
                    println!("opcode 4: IF B EQUALS C SET A TO 1 ELSE A TO 0");
                    println!(" A: REGISTER");
                }
                ip = ip + 2;
                let a = write_argument(&state) as usize;

                if DEBUG || debug {
                    println!(" B: INTEGER");
                }
                ip = ip + 2;
                let b: u16 = read_argument(&state);

                if DEBUG || debug {
                    println!(" C: INTEGER");
                }
                ip = ip + 2;
                let c: u16 = read_argument(&state);

                if b == c {
                    register[a] = 1;
                } else {
                    register[a] = 0;
                }

                ip = ip + 2;
                if DEBUG || debug {
                    println!(" RESULT:  [A{}] = B{} == C{}", a, b, c);
                    println!("          [A{}] = {}", a, register[a]);
                    println!("");
                    println!(" [{} IP]", a);
                }
            }
            5 => {
                //3 args, should increment ip by 8
                if DEBUG || debug {
                    println!("opcode 5: IF B LARGER THAN C SET A TO 1 ELSE A TO 0");
                    println!(" A: REGISTER");
                }
                ip = ip + 2;
                let a = write_argument(&state) as usize;

                if DEBUG || debug {
                    println!(" B: INTEGER");
                }
                ip = ip + 2;
                let b: u16 = read_argument(&state);

                if DEBUG || debug {
                    println!(" C: INTEGER");
                }
                ip = ip + 2;
                let c: u16 = read_argument(&state);

                if b > c {
                    register[a] = 1;
                } else {
                    register[a] = 0;
                }

                ip = ip + 2;
                if DEBUG || debug {
                    println!(" RESULT:  [A{}] = B{} > C{}", a, b, c);
                    println!("          [A{}] = {}", a, register[a]);
                    println!("");
                    println!("[{} IP]", a);
                }
            }
            6 => {
                if DEBUG || debug {
                    println!("opcode 6: JUMP");
                    println!(" A: JUMP ADDRESS");
                }
                ip = ip + 2;
                let a = read_argument(&state) as usize;

                ip = a * 2;

                if DEBUG || debug {
                    println!(" RESULT:  [IP] = &{}", a * 2);
                    println!("          [IP] = &{}", ip);
                    println!("");
                    println!(" [IP]");
                }
            }
            7 => {
                if DEBUG || debug {
                    println!("opcode 7: JUMP IF NONZERO");
                    println!(" A: CONDITIONAL");
                }
                ip = ip + 2;
                let a: u16 = read_argument(&state);

                if DEBUG || debug {
                    println!(" B: JUMP ADDRESS");
                }
                ip = ip + 2;
                let b = read_argument(&state) as usize;

                if a != 0 {
                    ip = b * 2;
                } else {
                    ip = ip + 2;
                }

                if DEBUG || debug {
                    println!(" RESULT:  A{} != 0", b * 2);
                    println!("          [IP] = B{}", b * 2);
                    println!("          [IP] == {}", ip);
                    println!(" [IP]");
                }
            }
            8 => {
                if DEBUG || debug {
                    println!("opcode 8: JUMP IF ZERO");
                    println!(" A: CONDITIONAL");
                }
                ip = ip + 2;
                let a: u16 = read_argument(&state);

                if DEBUG || debug {
                    println!(" B: JUMP ADDRESS");
                }
                ip = ip + 2;
                let b = read_argument(&state) as usize;

                if a == 0 {
                    ip = b * 2;
                } else {
                    ip = ip + 2;
                }

                if DEBUG || debug {
                    println!(" RESULT:  A{} == 0", a);
                    println!("          [IP] = B{}", b * 2);
                    println!("          [IP] == {}", ip);
                    println!(" [IP]");
                }
            }
            9 => {
                if DEBUG || debug {
                    println!("opcode 9: ADD SET [A] RESULT B + C");
                    println!(" A: REGISTER");
                }
                ip = ip + 2;
                let a = write_argument(&state) as usize;

                if DEBUG || debug {
                    println!(" B: INTEGER");
                }
                ip = ip + 2;
                let b: u16 = read_argument(&state);

                if DEBUG || debug {
                    println!(" C: INTEGER");
                }
                ip = ip + 2;
                let c: u16 = read_argument(&state);

                register[a] = (b + c) % 32768;

                ip = ip + 2;
                if DEBUG || debug {
                    println!(" RESULT:  B{} + C{} = {}", b, c, (b + c) % 32768);
                    println!("          [A{}] = {}", a, register[a]);
                    println!("");
                    println!(" [IP {}]", a);
                }
            }
            10 => {
                if DEBUG || debug {
                    println!("opcode 10: MUTIPLY SET [A] RESULT B * C");
                    println!(" A: REGISTER");
                }
                ip = ip + 2;
                let a = write_argument(&state) as usize;

                if DEBUG || debug {
                    println!(" B: INTEGER");
                }
                ip = ip + 2;
                let b = read_argument(&state) as usize;

                if DEBUG || debug {
                    println!(" C: INTEGER");
                }
                ip = ip + 2;
                let c = read_argument(&state) as usize;

                register[a] = ((b * c) % 32768) as u16;

                ip = ip + 2;
                if DEBUG || debug {
                    println!(" RESULT:  B{} * C{} = {}", b, c, (b * c) % 32768);
                    println!("          [A{}] = {}", a, register[a]);
                    println!("");
                    println!(" [IP {}]", a);
                }
            }
            11 => {
                if DEBUG || debug {
                    println!("opcode 11: MODULO SET [A] RESULT B % C");
                    println!(" A: REGISTER");
                }
                ip = ip + 2;
                let a = write_argument(&state) as usize;

                if DEBUG || debug {
                    println!(" B: INTEGER");
                }
                ip = ip + 2;
                let b: u16 = read_argument(&state);

                if DEBUG || debug {
                    println!(" C: INTEGER");
                }
                ip = ip + 2;
                let c: u16 = read_argument(&state);

                register[a] = (b % c) % 32768;

                ip = ip + 2;
                if DEBUG || debug {
                    println!(" RESULT:  {} % {} = {}", b, c, (b % c) % 32768);
                    println!("          [A{}] = {}", a, register[a]);
                    println!("");
                    println!(" [IP {}]", a);
                }
            }
            12 => {
                if DEBUG || debug {
                    println!("opcode 12: AND SET [A] RESULT B & C");
                    println!(" A: REGISTER");
                }
                ip = ip + 2;
                let a = write_argument(&state) as usize;

                if DEBUG || debug {
                    println!(" B: INTEGER");
                }
                ip = ip + 2;
                let b: u16 = read_argument(&state);

                if DEBUG || debug {
                    println!(" C: INTEGER");
                }
                ip = ip + 2;
                let c: u16 = read_argument(&state);

                register[a] = (b & c) % 32768;

                ip = ip + 2;
                if DEBUG || debug {
                    println!(" RESULT:  {} & {} = {}", b, c, (b & c) % 32768);
                    println!("          [A{}] = {}", a, register[a]);
                    println!("");
                    println!(" [IP {}]", a);
                }
            }
            13 => {
                if DEBUG || debug {
                    println!("opcode 13: OR SET [A] RESULT B | C");
                    println!(" A: REGISTER");
                }
                ip = ip + 2;
                let a = write_argument(&state) as usize;

                if DEBUG || debug {
                    println!(" B: INTEGER");
                }
                ip = ip + 2;
                let b: u16 = read_argument(&state);

                if DEBUG || debug {
                    println!(" C: INTEGER");
                }
                ip = ip + 2;
                let c: u16 = read_argument(&state);

                register[a] = (b | c) % 32768;

                ip = ip + 2;
                if DEBUG || debug {
                    println!(" RESULT:  {} | {} = {}", b, c, (b | c) % 32768);
                    println!("          [A{}] = {}", a, register[a]);
                    println!("");
                    println!(" [IP {}]", a);
                }
            }
            14 => {
                if DEBUG || debug {
                    println!("opcode 14: NOT SET [A] RESULT !B");
                    println!(" A: REGISTER");
                }
                ip = ip + 2;
                let a = write_argument(&state) as usize;

                if DEBUG || debug {
                    println!(" B: INTEGER");
                }
                ip = ip + 2;
                let b: u16 = read_argument(&state);

                register[a] = (!b) % 32768;
                ip = ip + 2;
                if DEBUG || debug {
                    println!(" RESULT:  !{} = {}", b, (!b) % 32768);
                    println!("          [A{}] = {}", a, register[a]);
                    println!("");
                    println!(" [IP {}]", a);
                }
            }
            15 => {
                if DEBUG || debug {
                    println!("opcode 15: RMEM READ TO [A] FROM &B");
                    println!(" A: REGISTER");
                }
                ip = ip + 2;
                let a = write_argument(&state) as usize;

                if DEBUG || debug {
                    println!(" B: ADDRESS");
                }
                ip = ip + 2;
                let mut b = read_argument(&state) as usize;

                b = b * 2;

                if DEBUG || debug {
                    println!(" &B: MEMORY AT B");
                }
                let c = read_argument(&state);

                register[a] = c;

                ip = ip + 2;
                if DEBUG || debug {
                    println!(" RESULT:  [A{}] = &{}", a, b);
                    println!("          [A{}] = {}", a, c);
                    println!("          [A{}] = {}", a, register[a]);
                    println!(" [IP {}]", a);
                }
            }
            16 => {
                if DEBUG || debug {
                    println!("opcode 16: WMEM WRITE B TO &A");
                    println!(" A: ADDRESS");
                }
                ip = ip + 2;
                let mut a = read_argument(&state) as usize;
                a = a * 2;

                if DEBUG || debug {
                    println!(" B: INTEGER");
                }
                ip = ip + 2;
                let b: u16 = read_argument(&state);

                let higher = (b >> 8) as u8;
                let lower = b as u8;

                state.program[a + 1] = higher;
                state.program[a] = lower;

                ip = ip + 2;
                if DEBUG || debug {
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
                if DEBUG || debug {
                    println!("opcode 17: CALL &A");
                    println!(" A: ADDRESS");
                }
                ip = ip + 2;
                let a = read_argument(&state) as usize;

                ip = ip + 2;
                stack.push(ip as u16 / 2);
                sp = sp + 1;

                ip = a * 2;
                if DEBUG || debug {
                    println!(" RESULT:  [IP{}] = A{}", ip, a * 2);
                    println!("          <{}> = IP{}", sp - 1, stack[sp - 1]);
                    println!("");
                    println!(" [IP SP]");
                }
            }
            18 => {
                if sp == 0 {
                    if DEBUG || debug {
                        println!("opcode 18: return: {}", stack[sp]);
                    }
                    println!("instructions completed {}", ops);
                    println!("IP at {}", ip);
                    break;
                }

                sp = sp - 1;

                if DEBUG || debug {
                    println!("opcode 18: RETURN: {}", stack[sp] * 2);
                }
                ip = stack[sp] as usize * 2;
                stack[sp] = 0;
            }
            19 => {
                ip = ip + 2;
                if DEBUG || debug {
                    let a = read_argument(&state);
                    println!("opcode 19: PRINT: {}", state.program[ip]);
                    println!("{}", a as u8 as char);
                } else {
                    let a = read_argument(&state);
                    // eprintln!("opcode 19: PRINT: {} {}", a as u8 as char, a);
                    print!("{}", a as u8 as char);
                    // eprint!("{}", program[ip] as char);
                }
                ip = ip + 2;
            }
            20 => {
                if DEBUG || debug {
                    println!("opcode 20: READ TO [A]");
                    println!(" A: REGISTER");
                }
                ip = ip + 2;
                let a = write_argument(&state) as usize;

                let res = read();
                if res as char == '~' {
                    counter = 0;
                    continue;
                }
                register[a] = res as u16;
                read_counter = 1;

                ip = ip + 2;
            }
            21 => {
                if DEBUG || debug {
                    println!("opcode 21: NOOP");
                }
                ip = ip + 2;
            }
            22 => {
                if ip > 0 && ops > 1 {
                    panic!("opcode 22 encountered outside of load state")
                }
                debug = false;
                if DEBUG || debug {
                    println!("opcode 22: LOAD");
                }
                println!("opcode 22: LOAD");
                ip = ip + 1;
                for i in 0..7 {
                    // load the registers
                    let n = i * 2;
                    let higher = state.program[ip + n + 1] as u16;
                    let lower = state.program[ip + n] as u16;
                    let value: u16 = higher << 8 | lower;
                    register[i] = value;
                }
                ip = ip + 16;
                for i in 0..99 {
                    // load the registers
                    let n = i * 2;
                    let higher = state.program[ip + n + 1] as u16;
                    let lower = state.program[ip + n] as u16;
                    let value: u16 = higher << 8 | lower;
                    stack[i] = value;
                }
                ip = ip + 100;
                let higher = state.program[ip] as u16;
                let lower = state.program[ip + 1] as u16;
                let value: u16 = higher << 8 | lower;
                ip = ip + 2;
                sp = value as usize;
                let higher = state.program[ip] as u16;
                let lower = state.program[ip + 1] as u16;
                let value: u16 = higher << 8 | lower;
                ip = value as usize;
                if DEBUG || debug {
                    println!("SP at {} {:x}", sp, sp);
                    println!("IP at {} {:x}", ip, ip);
                }
            }
            c => {
                println!(
                    "opcode {}: err unkown opcode at {} follows: {:x} {:x}",
                    c,
                    ip,
                    state.program[(ip + 1)],
                    state.program[(ip + 2)]
                );
                println!("instructions completed {}", ops);
                println!("IP at {} {:x}", ip, ip);
                let mut i = 0;
                loop {
                    println!("register {}: {}", i, register[i]);
                    i = i + 1;
                    if i > 7 {
                        break;
                    }
                }
                let mut i = 0;
                loop {
                    println!("stack {}: {}", i, stack[i]);
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
    }

    Ok(())
}

fn read_argument(state: &State) -> u16 {
    let higher = state.program[state.ip + 1] as u16;
    let lower = state.program[state.ip] as u16;

    let mut argument: u16 = higher << 8 | lower;
    if DEBUG || state.debug {
        println!("read_argument found number {}", argument);
    }
    while argument > 32767 {
        let index = argument as usize;
        if DEBUG || state.debug {
            println!("               reguested contents of [{}]", index % 32768);
        }
        argument = state.register[index % 32768];
        if DEBUG || state.debug {
            println!("                    content {}", argument);
        }
    }
    return argument;
}

fn write_argument(state: &State) -> u16 {
    let higher = state.program[state.ip + 1] as u16;
    let lower = state.program[state.ip] as u16;

    let mut argument: u16 = higher << 8 | lower;
    if DEBUG || state.debug {
        println!("write_argument found number {}", argument);
    }
    if argument > 32767 {
        argument = argument % 32768;
        if DEBUG || state.debug {
            println!("                request is to register [{}]", argument);
        }
    }
    if argument > 7 {
        println!(" using special register [8], request was for {}", argument);
        argument = 8;
    }
    return argument;
}

fn read() -> u8 {
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

fn halt(state: State, meta: Meta) {
    let register = state.register;
    let stack = state.stack.clone();
    let ip = state.ip;
    if DEBUG || state.debug {
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
}
