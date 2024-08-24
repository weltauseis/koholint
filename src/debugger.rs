use std::io::Write;

use crate::{gameboy::Gameboy, instructions::decode_instruction};

pub fn debug_console(mut console: Gameboy) {
    println!("Welcome to my GBC debugger !");

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
                    println!("  help : display this help message");
                    println!("  exit : quit the debugger");
                    println!("  list : print assembly at current program counter");
                    println!("  next : execute current instruction");
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
                },
                _ => {
                    println!("Error : Unknown command ({})", subcommands[0]);
                }
            },
        }
    }
}
