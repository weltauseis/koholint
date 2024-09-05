use std::{
    io::Write,
    sync::{Arc, Mutex},
};

use log::error;

use crate::{decoding::decode_instruction, gameboy::Gameboy};

pub fn debug_console(mut console: Gameboy) {
    println!("Welcome to my GBC debugger !");

    let mut breakpoints: Vec<u16> = Vec::new();

    loop {
        // prompt
        print!("{:#06X} (dbg)> ", console.cpu().read_program_counter());
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
                // step program if empty command
                console.step();
            }
            Some(cmd) => {
                match *cmd {
                    "help" | "h" => {
                        println!("Available commands :");
                        println!("  help     : display this help message");
                        println!("  exit     : quit the debugger");
                        println!("  list     : print assembly at current program counter");
                        println!("  print    : print the value of a register");
                        println!("  flags    : print the value of the flags register");
                        println!("  next     : execute current instruction");
                        println!("  continue : resume execution until next beakpoint");
                        println!("  break    : place a breakpoint at a specific program counter");
                        println!("  remove   : remove a breakpoint at a specific program counter");
                        println!("  dump     : dump a VRAM tile into a ppm image file");
                    }
                    "exit" => {
                        println!("Exiting debugger...");
                        break;
                    }
                    "list" | "l" => {
                        // TODO: find a way to show previous instructions
                        let pc = console.cpu().read_program_counter();
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
                    "print" | "p" => match subcommands.get(1) {
                        None => {
                            println!("Error : Missing register or flag name");
                            continue;
                        }
                        Some(reg_name) => match *reg_name {
                            "a" => println!("a : {:#04X}", console.cpu().read_a_register()),
                            "b" => println!("b : {:#04X}", console.cpu().read_b_register()),
                            "c" => println!("c : {:#04X}", console.cpu().read_c_register()),
                            "d" => println!("d : {:#04X}", console.cpu().read_d_register()),
                            "e" => println!("e : {:#04X}", console.cpu().read_e_register()),
                            "h" => println!("h : {:#04X}", console.cpu().read_h_register()),
                            "l" => println!("l : {:#04X}", console.cpu().read_l_register()),
                            "bc" => println!("bc : {:#06X}", console.cpu().read_bc_register()),
                            "de" => println!("de : {:#06X}", console.cpu().read_de_register()),
                            "hl" => println!("hl : {:#06X}", console.cpu().read_hl_register()),
                            "sp" => {
                                println!("sp : {:#06X}", console.cpu().read_stack_pointer())
                            }
                            _ => {
                                println!("Error : Unknown register or flag ({})", reg_name);
                            }
                        },
                    },
                    "flags" | "f" => {
                        println!(
                            "{} {} {} {}",
                            console.cpu().read_z_flag() as u8,
                            console.cpu().read_n_flag() as u8,
                            console.cpu().read_h_flag() as u8,
                            console.cpu().read_c_flag() as u8
                        );
                    }
                    "next" | "n" => {
                        console.step();
                    }
                    "continue" | "c" => loop {
                        console.step();

                        let pc = console.cpu().read_program_counter();
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
                    "dump" => {
                        const WIDTH: u16 = 16; // in tiles
                        const HEIGHT: u16 = 24; // in tiles, should be equal to 384/WIDTH

                        let mut ppm_string = format!("P3\n{} {}\n255\n", WIDTH * 8, HEIGHT * 8);
                        // https://gbdev.io/pandocs/Tile_Data.html#vram-tile-data

                        // this whole thing is pretty convoluted but i don't think there's a better way,
                        // as the memory layout of the gameboy tiles is very different from that of "normal" images
                        for i in 0..HEIGHT {
                            for y in 0..8 {
                                for j in 0..WIDTH {
                                    let line_address =
                                    //  vram   | start of 16-bytes tile      | line offset
                                        0x8000 + (j * 16) + (i * 16 * WIDTH) + (2 * y);
                                    let byte_1: u8 = console.memory().read_byte(line_address);
                                    let byte_2: u8 = console.memory().read_byte(line_address + 1);
                                    for x in 0..8 {
                                        let mut pixel: u8 = 0;

                                        let bit_0 = byte_1 >> (7 - x) & 1;
                                        let bit_1 = byte_2 >> (7 - x) & 1;

                                        pixel |= bit_0;
                                        pixel |= bit_1 << 1;

                                        ppm_string.push_str(match pixel {
                                            0 => "15 15 27\n",
                                            1 => "86 90 117\n",
                                            2 => "198 183 190\n",
                                            3 => "250 251 246\n",
                                            _ => {
                                                error!("VERY weird value in VRAM dump");
                                                "255 0 0\n"
                                            }
                                        });
                                    }
                                }
                            }
                        }

                        std::fs::create_dir_all("dumps").unwrap();
                        let mut file = std::fs::File::create(format!("dumps/vram.ppm")).unwrap();
                        file.write_all(ppm_string.as_bytes()).unwrap();
                    }
                    _ => {
                        println!("Error : Unknown command ({})", subcommands[0]);
                    }
                }
            }
        }
    }
}

/* pub fn disassemble_rom(console: &Gameboy) -> Vec<(u16, String)> {
    let mut disassembly: Vec<(u16, String)> = Vec::new();

    let mut pc = 0;
    // FIXME : this currently only disassembles the boot rom
    for _ in 0..64 {
        let instr = decode_instruction(console, pc);
        disassembly.push((pc, instr.to_string()));
        pc += instr.size;
    }

    return disassembly;
} */
