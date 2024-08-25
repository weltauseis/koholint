use std::io::Write;

use crate::{decoding::decode_instruction, gameboy::Gameboy};

pub fn debug_console(mut console: Gameboy) {
    println!("Welcome to my GBC debugger !");

    let mut breakpoints: Vec<u16> = Vec::new();

    loop {
        // prompt
        print!("{:#06X} (dbg)> ", console.cpu().get_program_counter());
        std::io::stdout().flush().unwrap();

        // get user input
        let mut input = String::new();
        std::io::stdin().read_line(&mut input).unwrap();

        if input.is_empty() {
            // quit in case of EOF
            break;
        }

        let subcommands: Vec<&str> = input.trim().split_whitespace().collect();
        match subcommands.get(0) {
            None => {
                // continue if empty command
                continue;
            }
            Some(cmd) => match *cmd {
                "help" | "h" => {
                    println!("Available commands :");
                    println!("  help     : display this help message");
                    println!("  exit     : quit the debugger");
                    println!("  list     : print assembly at current program counter");
                    println!("  next     : execute current instruction");
                    println!("  continue : resume execution until next beakpoint");
                    println!("  break    : place a breakpoint at a specific program counter");
                    println!("  remove   : remove a breakpoint at a specific program counter");
                }
                "exit" => {
                    println!("Exiting debugger...");
                    break;
                }
                "list" | "l" => {
                    // TODO: find a way to show previous instructions

                    let pc = console.cpu().get_program_counter();
                    let mut to_list = match subcommands.get(1) {
                        None => 5,
                        Some(nb_string) => match nb_string.parse() {
                            Ok(nb) => nb,
                            Err(e) => {
                                println!("Error : {e}");
                                continue;
                            }
                        },
                    };

                    let mut pos = pc;
                    while to_list > 0 {
                        let instr = decode_instruction(&console, pos);
                        println!(
                            "{:>12} {:#06X} | {}",
                            if pos == pc { "->" } else { "" },
                            pos,
                            instr
                        );

                        pos += instr.size;
                        to_list -= 1;
                    }
                }
                "next" | "n" => {
                    console.step();
                }
                "continue" | "c" => loop {
                    console.step();

                    let pc = console.cpu().get_program_counter();
                    if breakpoints.iter().any(|breakpoint| pc == *breakpoint) {
                        println!("Reached breakpoint ({:#06X})", pc);
                        break;
                    }
                },
                "break" | "b" => match subcommands.get(1) {
                    None => {
                        println!("Error : Missing breakpoint adress");
                        continue;
                    }
                    Some(address_string) => {
                        let address = match u16::from_str_radix(address_string, 16) {
                            Ok(parsed) => parsed,
                            Err(e) => {
                                println!("Error : {e}");
                                continue;
                            }
                        };

                        if breakpoints.contains(&address) {
                            println!("Error : Breakpoint is already placed");
                            continue;
                        }

                        breakpoints.push(address);
                    }
                },
                "remove" | "r" => match subcommands.get(1) {
                    None => {
                        println!("Error : Missing breakpoint adress");
                        continue;
                    }
                    Some(address_string) => {
                        let address = match u16::from_str_radix(address_string, 16) {
                            Ok(parsed) => parsed,
                            Err(e) => {
                                println!("Error : {e}");
                                continue;
                            }
                        };

                        if let Some(pos) = breakpoints.iter().position(|&x| x == address) {
                            breakpoints.remove(pos);
                        } else {
                            println!("Error : Breakpoint not found");
                        }
                    }
                },
                _ => {
                    println!("Error : Unknown command ({})", subcommands[0]);
                }
            },
        }
    }
}
