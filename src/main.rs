use std::env;
use std::fs;

fn main() {
    let args: Vec<String> = env::args().collect();

    println!("reading: {}", &args[1]);

    if let Ok(program) = fs::read(&args[1]) {
        run(program);
    } else {
        println!("err");
    }


}

fn run(mut program: Vec<u8>) {
    // registers
    let mut ip: usize = 0;
    let mut sp: usize = 0;
    let mut register:[u16;8]= [0; 8];
    let mut op_counter = 0;

    let mut stack:[u16;1028] = [0; 1028];

    let debug = true;
    let mut breakpoint = 0;

    loop {
        op_counter = op_counter + 1;
        if debug == true {
            println!("{} {}", ip, op_counter);
        }
        if op_counter > 1000 {
            println!("instructions completed {}", op_counter);
            println!("IP at {}", ip);
            break;
        }
        match program[ip as usize]{
            0 => {
                if debug == true {
                    println!("opcode 0: halt");
                }
                println!("instructions completed {}", op_counter);
                println!("IP at {}", ip);
                break;
            },
            1 => {
                if debug == true {
                    println!("opcode 1: set a b");
                }

                ip = ip + 2;
                let a = program[ip + 1] as usize;
                let b = program[ip] as usize;

                let mut target = a << 8 | b;

                target = target % 32768;
                ip = ip + 2;

                if target < 8 {
                    let c = program[ip + 1] as u16;
                    let d = program[ip] as u16;

                    let value = c << 8 | d;

                    register[target] = value;
                    if debug == true {
                        println!("set {} to {}", target, value);
                        if value == 16724 {
                            if breakpoint == 9 {
                                println!("instructions completed {}", op_counter);
                                println!("IP at {}", ip);
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
                                break;
                            } else {
                                breakpoint = breakpoint + 1;
                            }
                        }
                    }
                }

                ip = ip + 2;

            },
            2 => {
                ip = ip + 2;
                let higher = program[ip + 1] as usize;
                let lower = program[ip] as usize;

                let mut a = higher << 8 | lower;

                if debug == true {
                    println!("opcode 2 (raw): {} reg {}", a, a % 32768);
                }

                if a > 32767 {
                    a = register[a % 32768] as usize;
                }

                if debug == true {
                    println!("opcode 2: push a: {}", a);
                }
                stack[sp] = a as u16;
                sp = sp + 1;
                ip = ip + 2;
            },
            3 => {
                ip = ip + 2;
                let higher = program[ip + 1] as usize;
                let lower = program[ip] as usize;

                let a = (higher << 8 | lower) % 32768;

                if debug == true {
                    println!("opcode 3: pop into a: {} {}", a, stack[sp]);
                }

                sp = sp - 1;
                register[a] = stack[sp];
                if debug == true {
                    println!("opcode 3: pop into a: {} {}", a, stack[sp]);
                }
                stack[sp] = 0;
                ip = ip + 2;
            },
            4 => {
                ip = ip + 2;
                let mut higher = program[ip + 1] as usize;
                let mut lower = program[ip] as usize;

                let a = (higher << 8 | lower) % 32768;

                ip = ip + 2;
                higher = program[ip + 1] as usize;
                lower = program[ip] as usize;

                let mut b = higher << 8 | lower;

                ip = ip + 2;
                higher = program[ip + 1] as usize;
                lower = program[ip] as usize;

                let mut c = higher << 8 | lower;
                if debug == true {
                    println!("opcode 4 (raw): set a to b == c: {} {} {}", a, b, c);
                }

                if b > 32767 {
                    b = register[b % 32768] as usize;
                }

                if c > 32767 {
                    c = register[c % 32768] as usize;
                }

                if debug == true {
                    println!("opcode 4: set a to b == c: {} {} {}", a, b, c);
                }
                if b == c {
                    register[a] = 1;
                } else {
                    register[a] = 0;
                }

                if debug == true {
                    println!("opcode 4: {}", register[a]);
                }
                ip = ip + 2;
            },
            5 => {
                ip = ip + 2;
                let mut higher = program[ip + 1] as usize;
                let mut lower = program[ip] as usize;

                let a = (higher << 8 | lower) % 32768;

                ip = ip + 2;
                higher = program[ip + 1] as usize;
                lower = program[ip] as usize;

                let b = (higher << 8 | lower) % 32768;

                ip = ip + 2;
                higher = program[ip + 1] as usize;
                lower = program[ip] as usize;

                let c = (higher << 8 | lower) % 32768;

                if debug == true {
                    println!("opcode 5: set a to b > c: {} {} {}", a, b, c);
                }
                if b > c {
                    register[a] = 1;
                } else {
                    register[a] = 0;
                }

                if debug == true {
                    println!("opcode 5: {}", register[a]);
                }
                ip = ip + 2;
            },
            6 => {
                ip = ip + 2;
                let a = program[ip + 1] as usize;
                let b = program[ip] as usize;

                if debug == true {
                    println!("opcode 6: jump to: {}", a << 8 | b);
                }

                ip = (a << 8 | b) * 2;
            },
            7 => {
                ip = ip + 2;
                let mut higher = program[ip + 1] as usize;
                let mut lower = program[ip] as usize;

                let mut a = higher << 8 | lower;

                ip = ip + 2;
                higher = program[ip + 1] as usize;
                lower = program[ip] as usize;

                let b = higher << 8 | lower;

                if debug == true {
                    println!("opcode 7 (raw): jump to b if a is nonzero: {} {}", a, b);
                }

                if a > 32767 && a < 32776 {
                    a = register[a % 32768] as usize;
                }

                if debug == true {
                    println!("opcode 7: jump to b if a is nonzero: {} {}", a, b);
                }

                if a > 0 {
                    ip = b * 2;
                } else {
                    ip = ip + 2;
                }
            },
            8 => {
                ip = ip + 2;
                let mut higher = program[ip + 1] as usize;
                let mut lower = program[ip] as usize;

                let mut a = higher << 8 | lower;

                ip = ip + 2;
                higher = program[ip + 1] as usize;
                lower = program[ip] as usize;

                let b = higher << 8 | lower;

                if debug == true {
                    println!("opcode 8 (raw): jump to b if a is zero: {} {}", a, b);
                }

                if a > 32767 && a < 32776 {
                    a = register[a % 32768] as usize;
                }

                if debug == true {
                    println!("opcode 8: jump to b if a is zero: {} {}", a, b * 2);
                }

                if a > 0 {
                    ip = ip + 2;
                } else {
                    ip = b * 2;
                }
            },
            9 => {
                ip = ip + 2;
                let mut higher = program[ip + 1] as usize;
                let mut lower = program[ip] as usize;

                let a = (higher << 8 | lower) % 32768;

                ip = ip + 2;
                higher = program[ip + 1] as usize;
                lower = program[ip] as usize;

                let b = higher << 8 | lower;

                ip = ip + 2;
                higher = program[ip + 1] as usize;
                lower = program[ip] as usize;

                let c = higher << 8 | lower;

                let b = b as u16;
                let c = c as u16;

                register[a] = ((b + c) % 32768) as u16;

                if debug == true {
                    println!("opcode 9: add into a, b + c: {} {} {}", a, b, c);
                    println!("opcode 9: {}", register[a]);
                }

                ip = ip + 2;
            },
            10 => {
                ip = ip + 2;
                let mut higher = program[ip + 1] as usize;
                let mut lower = program[ip] as usize;

                let a = (higher << 8 | lower) % 32768;

                ip = ip + 2;
                higher = program[ip + 1] as usize;
                lower = program[ip] as usize;

                let mut b = higher << 8 | lower;

                ip = ip + 2;
                higher = program[ip + 1] as usize;
                lower = program[ip] as usize;

                let mut c = higher << 8 | lower;
                if debug == true {
                    println!("opcode 10 (raw): {} {} {}", a, b, c);
                }

                if b > 32767 {
                    b = register[b % 32768] as usize;
                }

                if c > 32767 {
                    c = register[c % 32768] as usize;
                }

                if debug == true {
                    println!("opcode 10: set a to b * c: {} {} {}", a, b, c);
                }

                register[a] = ((b * c) % 32768) as u16;

                if debug == true {
                    println!("opcode 10: {}", register[a]);
                }
                ip = ip + 2;
            },
            11 => {
                ip = ip + 2;
                let mut higher = program[ip + 1] as usize;
                let mut lower = program[ip] as usize;

                let a = (higher << 8 | lower) % 32768;

                ip = ip + 2;
                higher = program[ip + 1] as usize;
                lower = program[ip] as usize;

                let mut b = higher << 8 | lower;

                ip = ip + 2;
                higher = program[ip + 1] as usize;
                lower = program[ip] as usize;

                let mut c = higher << 8 | lower;
                if debug == true {
                    println!("opcode 11 (raw): {} {} {}", a, b, c);
                }

                if b > 32767 {
                    b = register[b % 32768] as usize;
                }

                if c > 32767 {
                    c = register[c % 32768] as usize;
                }

                let b = b as u16;
                let c = c as u16;

                if debug == true {
                    println!("opcode 11: set a to b % c: {} {} {}", a, b, c);
                }

                register[a] = ((b % c) % 32768) as u16;

                if debug == true {
                    println!("opcode 11: {}", register[a]);
                }
                ip = ip + 2;
            },
            12 => {
                ip = ip + 2;
                let mut higher = program[ip + 1] as usize;
                let mut lower = program[ip] as usize;

                let a = (higher << 8 | lower) % 32768;

                ip = ip + 2;
                higher = program[ip + 1] as usize;
                lower = program[ip] as usize;

                let mut b = higher << 8 | lower;

                ip = ip + 2;
                higher = program[ip + 1] as usize;
                lower = program[ip] as usize;

                let mut c = higher << 8 | lower;

                if b > 32767 {
                    b = register[b % 32768] as usize;
                }

                if c > 32767 {
                    c = register[c % 32768] as usize;
                }

                let b = b as u16;
                let c = c as u16;

                if debug == true {
                    println!("opcode 12: and, store into a bitwise b and c: {} {} {}", a, b, c);
                }
                register[a] = b & c;

                if debug == true {
                    println!("opcode 12: {}", register[a]);
                }
                ip = ip + 2;
            },
            13 => {
                ip = ip + 2;
                let mut higher = program[ip + 1] as usize;
                let mut lower = program[ip] as usize;

                let a = (higher << 8 | lower) % 32768;

                ip = ip + 2;
                higher = program[ip + 1] as usize;
                lower = program[ip] as usize;

                let mut b = higher << 8 | lower;

                ip = ip + 2;
                higher = program[ip + 1] as usize;
                lower = program[ip] as usize;

                let mut c = higher << 8 | lower;

                if b > 32767 {
                    b = register[b % 32768] as usize;
                }

                if c > 32767 {
                    c = register[c % 32768] as usize;
                }

                let b = b as u16;
                let c = c as u16;

                if debug == true {
                    println!("opcode 13: or, store into a bitwise or b c: {} {} {}", a, b, c);
                }
                register[a] = b | c;

                if debug == true {
                    println!("opcode 13: {}", register[a]);
                }
                ip = ip + 2;
            },
            14 => {
                ip = ip + 2;
                let mut higher = program[ip + 1] as usize;
                let mut lower = program[ip] as usize;

                let a = (higher << 8 | lower) % 32768;

                ip = ip + 2;
                higher = program[ip + 1] as usize;
                lower = program[ip] as usize;

                let mut b = higher << 8 | lower;

                if b > 32767 {
                    b = register[b % 32768] as usize;
                }

                let b = b as u16;

                if debug == true {
                    println!("opcode 14: not, store into a not b: {} {}", a, b);
                }
                register[a] = !b % 32768;

                if debug == true {
                    println!("opcode 14: {}", register[a]);
                }
                ip = ip + 2;
            },
            15 => {
                ip = ip + 2;
                let mut higher = program[ip + 1] as usize;
                let mut lower = program[ip] as usize;

                let a = (higher << 8 | lower) % 32768;

                ip = ip + 2;
                higher = program[ip + 1] as usize;
                lower = program[ip] as usize;

                let mut b = higher << 8 | lower;
                if debug == true {
                    println!("opcode 15 (raw): {} {}", a, b);
                }

                if b > 32767 {
                    b = register[b % 32768] as usize;
                }

                b = b * 2;

                higher = program[b + 1] as usize;
                lower = program[b] as usize;

                let c = higher << 8 | lower;

                if debug == true {
                    println!("opcode 15: read from mem b to reg a: {} {} {}", a, b, c);
                }

                register[a] = c as u16;

                if debug == true {
                    println!("opcode 15: {}", register[a]);
                }
                ip = ip + 2;
            },
            16 => {
                ip = ip + 2;
                let mut higher = program[ip + 1] as usize;
                let mut lower = program[ip] as usize;

                let mut a = higher << 8 | lower;

                ip = ip + 2;
                higher = program[ip + 1] as usize;
                lower = program[ip] as usize;

                let mut b = higher << 8 | lower;

                if debug == true {
                    println!("opcode 16 (raw): {} {}", a, b);
                }

                if a > 32767 {
                    a = register[a % 32768] as usize;
                }

                if b > 32767 {
                    b = register[b % 32768] as usize;
                }

                a = a * 2;

                if debug == true {
                    println!("opcode 16: write to a, b: {} {}", a, b);
                }

                program[a+1] = higher as u8;
                program[a] = lower as u8;

                if debug == true {
                    println!("opcode 16: {}", b);
                }
                ip = ip + 2;
            },
            17 => {
                ip = ip + 2;
                let higher = program[ip + 1] as usize;
                let lower = program[ip] as usize;

                let mut a = higher << 8 | lower;

                if debug == true {
                    println!("opcode 17 (raw): call a: {}", a);
                }

                if a > 32767 {
                    a = register[a % 32768] as usize;
                }

                ip = ip + 2;
                if debug == true {
                    println!("opcode 17: call a: {} {}", a * 2, ip / 2);
                }
                stack[sp] = ip as u16 / 2;
                sp = sp + 1;

                ip = a * 2;
            },
            18 => {
                if sp == 0 {
                    if debug == true {
                        println!("opcode 18: return: {}", stack[sp]);
                    }
                    println!("instructions completed {}", op_counter);
                    println!("IP at {}", ip);
                    break;
                }

                sp = sp - 1;

                if debug == true {
                    println!("opcode 18: return: {}", stack[sp] * 2);
                }
                ip = stack[sp] as usize * 2;
                stack[sp] = 0;
            },
            19 => {
                ip = ip + 2;
                if debug == true {
                    println!("opcode 19: print: {}", program[ip] as char);
                } else {
                    print!("{}", program[ip] as char);
                }
                ip = ip + 2;
            },
            21 => {
                if debug == true {
                    println!("opcode 21: noop");
                }
                ip = ip + 2;
            },
            c => {
                println!("opcode {}: err unkown opcode at {} follows: {:x} {:x}", c, ip, program[(ip + 1)], program[(ip + 2)]);
                println!("instructions completed {}", op_counter);
                println!("IP at {}", ip);
                break;
            }
        }
    }
}
