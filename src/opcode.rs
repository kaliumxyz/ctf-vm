use crate::util::read;
use crate::util::read_x;
use crate::util::read_argument;
use crate::halt;
use crate::debug::Meta;
use crate::vm::State;
use crate::util::write_argument;

#[derive(Debug)]
pub enum Code {
    /// halt: 0
    ///   stop execution and terminate the program
    Halt,
    /// set: 1 a b
    ///   set register <a> to the value of <b>
    Set(u8, u8),
    /// push: 2 a
    ///   push <a> onto the stack
    Push(u8),
    /// pop: 3 a
    ///   remove the top element from the stack and write it into <a>; empty stack = error
    Pop(u8),
    /// eq: 4 a b c
    ///   set <a> to 1 if <b> is equal to <c>; set it to 0 otherwise
    Equals(u8, u8, u8),
    /// gt: 5 a b c
    ///   set <a> to 1 if <b> is greater than <c>; set it to 0 otherwise
    GreaterThan(u8, u8, u8),
    /// jmp: 6 a
    ///   jump to <a>
    Jump(u8),
    /// jt: 7 a b
    ///   if <a> is nonzero, jump to <b>
    JumpIfTrue(u8, u8),
    /// jf: 8 a b
    ///   if <a> is zero, jump to <b>
    JumpIfFalse(u8, u8),
    /// add: 9 a b c
    ///   assign into <a> the sum of <b> and <c> (modulo 32768)
    Add(u8, u8, u8),
    /// mult: 10 a b c
    ///   store into <a> the product of <b> and <c> (modulo 32768)
    Multiply(u8, u8, u8),
    /// mod: 11 a b c
    ///   store into <a> the remainder of <b> divided by <c>
    Modulo(u8, u8, u8),
    /// and: 12 a b c
    ///   stores into <a> the bitwise and of <b> and <c>
    And(u8, u8, u8),
    /// or: 13 a b c
    ///   stores into <a> the bitwise or of <b> and <c>
    Or(u8, u8, u8),
    /// not: 14 a b
    ///   stores 15-bit bitwise inverse of <b> in <a>
    Not(u8, u8),
    /// rmem: 15 a b
    ///   read memory at address <b> and write it to <a>
    ReadMemory(u8, u8),
    /// wmem: 16 a b
    ///   write the value from <b> into memory at address <a>
    WriteMemory(u8, u8),
    /// call: 17 a
    ///   write the address of the next instruction to the stack and jump to <a>
    Call(u8),
    /// ret: 18
    ///   remove the top element from the stack and jump to it; empty stack = halt
    Return,
    /// out: 19 a
    ///   write the character represented by ascii code <a> to the terminal
    Out(u8),
    /// in: 20 a
    ///   read a character from the terminal and write its ascii code to <a>; it can be assumed that once input starts, it will continue until a newline is encountered; this means that you can safely read whole lines from the keyboard and trust that they will be fully read
    In(u8),
    /// noop: 21
    ///   no operation
    Noop,
    Unknown,
}

impl Code {
    pub fn len(&self) -> usize {
        match self {
            Code::Halt => 0,
            Code::Set(..) => 2,
            Code::Push(..) => 1,
            Code::Pop(..) => 1,
            Code::Equals(..) => 3,
            Code::GreaterThan(..) => 3,
            Code::Jump(..) => 1,
            Code::JumpIfTrue(..) => 2,
            Code::JumpIfFalse(..) => 2,
            Code::Add(..) => 3,
            Code::Multiply(..) => 3,
            Code::Modulo(..) => 3,
            Code::And(..) => 3,
            Code::Or(..) => 3,
            Code::Not(..) => 2,
            Code::ReadMemory(..) => 2,
            Code::WriteMemory(..) => 2,
            Code::Call(..) => 1,
            Code::Return => 0,
            Code::Out(..) => 1,
            Code::In(..) => 1,
            _ => 0,
        }
    }
}

/// get the opcode and arguments
pub fn parse(program: &Vec<u8>, ip: &usize)  -> Code {
    match program[*ip] {
        0 => Code::Halt,
        1 => Code::Set(program[ip+1], program[ip+2]),
        2 => Code::Push(program[ip+1]),
        3 => Code::Pop(program[ip+1]),
        4 => Code::Equals(program[ip+1], program[ip+2], program[ip+3]),
        5 => Code::GreaterThan(program[ip+1], program[ip+2], program[ip+3]),
        6 => Code::Jump(program[ip+1]),
        7 => Code::JumpIfTrue(program[ip+1], program[ip+2]),
        8 => Code::JumpIfFalse(program[ip+1], program[ip+2]),
        9 => Code::Add(program[ip+1], program[ip+2], program[ip+3]),
        10 => Code::Multiply(program[ip+1], program[ip+2], program[ip+3]),
        11 => Code::Modulo(program[ip+1], program[ip+2], program[ip+3]),
        12 => Code::And(program[ip+1], program[ip+2], program[ip+3]),
        13 => Code::Or(program[ip+1], program[ip+2], program[ip+3]),
        14 => Code::Not(program[ip+1], program[ip+2]),
        15 => Code::ReadMemory(program[ip+1], program[ip+2]),
        16 => Code::WriteMemory(program[ip+1], program[ip+2]),
        17 => Code::Call(program[ip+1]),
        18 => Code::Return,
        19 => Code::Out(program[ip+1]),
        20 => Code::In(program[ip+1]),
        21 => Code::Noop,
        _ => Code::Unknown,
    }
}

/// get debug information about op
pub fn debug_op(state: &mut State, meta: &mut Meta, code: Code) {
}

// /// print debug information about op
// pub fn inspect_op(code: Code) {
//     match code {
//         Code::Halt => {
//             println!("opcode 0: Halt");
//             println!("length: 2");
//         },
//         Code::Set(arg0, arg1) => {
//             println!("opcode 1: SET [A] TO B");
//             println!("length: 4");
//             println!(" A: REGISTER");
//             println!(" B: INTEGER");
//         }
//         Code::Unknown => {
//             println!("opcode unknown");
//         }
//     }
// }

// /// execute the opcode with side effects
// pub fn execute_op(state: &mut State, meta: &mut Meta, code: Code) {
//     match code {
//         Code::Halt => {
//             meta.halt = true;
//         },
//         Code::Set(arg0, arg1) => {
//             if meta.debug {
//                 println!("opcode 1: SET [A] TO B");
//                 println!(" A: STATE.REGISTER");
//             }
//             state.ip = state.ip + 2;
//             let a = write_argument(&state) as usize;
//             if meta.debug {
//                 println!(" B: INTEGER");
//             }
//             state.ip = state.ip + 2;
//             let b: u16 = read_argument(&state);

//             state.register[a] = b;

//             state.ip = state.ip + 2;
//             if meta.debug {
//                 println!(" RESULT:  [A{}] = B{}", a, b);
//                 println!("          [A{}] = {}", a, state.register[a]);
//                 println!("");
//                 println!(" [IP A{}]", a);
//             }
//         }
//         // Code::Push(program[ip+1]),
//         // Code::Pop(program[ip+1]),
//         // Code::Equals(program[ip+1], program[ip+2], program[ip+3]),
//         // Code::GreaterThan(program[ip+1], program[ip+2], program[ip+3]),
//         // Code::Jump(program[ip+1]),
//         // Code::JumpIfTrue(program[ip+1], program[ip+2]),
//         // Code::JumpIfFalse(program[ip+1], program[ip+2]),
//         // Code::Add(program[ip+1], program[ip+2], program[ip+3]),
//         // Code::Multiply(program[ip+1], program[ip+2], program[ip+3]),
//         // Code::Modulo(program[ip+1], program[ip+2], program[ip+3]),
//         // Code::And(program[ip+1], program[ip+2], program[ip+3]),
//         // Code::Or(program[ip+1], program[ip+2], program[ip+3]),
//         // Code::Not(program[ip+1], program[ip+2]),
//         // Code::ReadMemory(program[ip+1], program[ip+2]),
//         // Code::WriteMemory(program[ip+1], program[ip+2]),
//         // Code::Call(program[ip+1]),
//         // Code::Return,
//         // Code::Out(program[ip+1]),
//         // Code::In(program[ip+1]),
//         // Code::Noop,
//         // Code::Unknown,
//     }
// }

/// run the OP code with side effects
pub fn execute(state: &mut State, meta: &mut Meta) {
        match state.program[state.ip as usize] {
            0 => {
                meta.halt = true;
            },
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

                state.ip = state.ip + 2;
                if meta.debug {
                    println!(" RESULT:  <{}> = [A{}]", state.stack.len(), b);
                    println!("          <{}> = {}", state.stack.len(), a);
                    println!("          <{}> = {}", state.stack.len(), state.stack[state.stack.len() - 1]);
                    println!(" [SP IP] <{}>", state.stack.len());
                }
            }
            3 => {
                if meta.debug {
                    println!("opcode 3: POP FROM STACK TO [A]");
                    println!(" A: STATE.REGISTER");
                }
                state.ip = state.ip + 2;
                let a = write_argument(&state) as usize;

                if let Some(data) = state.stack.pop() {
                    state.register[a] = data;
                } else {
                    // halt
                }

                state.ip = state.ip + 2;
                if meta.debug {
                    println!(" RESULT:  [A{}] = <{}>", a, state.stack.len());
                    println!("          [A{}] = {}", a, state.register[a]);
                    println!(" [IP SP A{}] <{}>", a, state.stack.len());
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

                state.ip = a * 2;
                if meta.debug {
                    println!(" RESULT:  [IP{}] = A{}", state.ip, a * 2);
                    println!("          <{}> = IP{}", state.stack.len() - 1, state.stack[state.stack.len() - 1]);
                    println!("");
                    println!(" [IP SP]");
                }
            }
            18 => {
                if state.stack.len() == 0 {
                    if meta.debug {
                        println!("opcode 18: return: {}", state.stack[state.stack.len() - 1]);
                    }
                    println!("instructions completed {}", meta.op_count);
                    println!("IP at {}", state.ip);
                    meta.halt = true;
                }


                if meta.debug {
                    println!("opcode 18: RETURN: {}", state.stack[state.stack.len() - 1] * 2);
                }

                if let Some(n) = state.stack.pop() {
                    state.ip = n as usize * 2;
                } else {
                    // bad state?
                }
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
                    meta.debugging = true;
                    state.ip = state.ip - 2;
                } else {
                    state.register[a] = res as u16;
                    state.ip = state.ip + 2;
                }
            }
            21 => {
                if meta.debug {
                    println!("opcode 21: NOOP");
                }
                state.ip = state.ip + 2;
            }
            c => {
                println!(
                    "opcode {}: err unknown opcode at {} follows: {:x} {:x}",
                    c,
                    state.ip,
                    state.program[(state.ip + 1)],
                    state.program[(state.ip + 2)]
                );
                // println!("dumping program");
                // if let Ok(_) = fs::write("./out", program) {
                //     println!("dumped!");
                // }
                meta.debugging = true;
            }
        }
}
