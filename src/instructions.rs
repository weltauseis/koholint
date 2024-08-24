use core::fmt;
use std::path::Display;

use crate::{gameboy::Gameboy, memory::Memory};

#[derive(Debug)]
pub enum Operand {
    R8_A,
    R8_B,
    R8_C,
    R8_D,
    R8_E,
    R8_H,
    R8_L,
    R16_BC,
    R16_DE,
    R16_HL,
    R16_SP,
}
pub enum Operation {
    NOP,
    LD_R16_IMM16 { r16: Operand, imm16: u16 },
    LD_R8_IMM8 { r8: Operand, imm8: u8 },
    DEC_R8 { r8: Operand },
    JP_IMM16 { imm16: u16 },
}

pub struct Instruction {
    pub op: Operation,
    pub size: u16,
}

impl fmt::Display for Instruction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", get_instruction_assembly(self))
    }
}

pub fn decode_next_instruction(console: &Gameboy) -> Instruction {
    let pc = console.cpu().get_program_counter();
    return decode_instruction(console, pc);
}

pub fn decode_instruction(console: &Gameboy, address: u16) -> Instruction {
    let instr = console.memory().read_byte(address);

    match instr {
        0x00 => {
            return Instruction {
                op: Operation::NOP,
                size: 1,
            };
        }
        0x01 => {
            // ld bc, imm16
            return Instruction {
                op: Operation::LD_R16_IMM16 {
                    r16: Operand::R16_BC,
                    imm16: console.memory().read_word(address + 1),
                },
                size: 3,
            };
        }
        0x05 => {
            // dec b
            return Instruction {
                op: Operation::DEC_R8 { r8: Operand::R8_B },
                size: 1,
            };
        }
        0x06 => {
            // ld b, imm8
            return Instruction {
                op: Operation::LD_R8_IMM8 {
                    r8: Operand::R8_B,
                    imm8: console.memory().read_byte(address + 1),
                },
                size: 2,
            };
        }
        0x0D => {
            // dec c
            return Instruction {
                op: Operation::DEC_R8 { r8: Operand::R8_C },
                size: 1,
            };
        }
        0x0E => {
            // ld c, imm8
            return Instruction {
                op: Operation::LD_R8_IMM8 {
                    r8: Operand::R8_C,
                    imm8: console.memory().read_byte(address + 1),
                },
                size: 2,
            };
        }
        0x11 => {
            // ld de, imm16
            return Instruction {
                op: Operation::LD_R16_IMM16 {
                    r16: Operand::R16_DE,
                    imm16: console.memory().read_word(address + 1),
                },
                size: 3,
            };
        }
        0x20 => {
            // jr nz, imm8
        }
        0x21 => {
            // ld hl, imm16
            return Instruction {
                op: Operation::LD_R16_IMM16 {
                    r16: Operand::R16_HL,
                    imm16: console.memory().read_word(address + 1),
                },
                size: 3,
            };
        }
        0x31 => {
            // ld sp, imm16");
            return Instruction {
                op: Operation::LD_R16_IMM16 {
                    r16: Operand::R16_SP,
                    imm16: console.memory().read_word(address + 1),
                },
                size: 3,
            };
        }
        0x32 => {
            // ld (hl-), a");
        }
        // xor a, r8
        0xA8 => {
            // xor a, b");
        }
        0xA9 => {
            // xor a, c");
        }
        0xAA => {
            // xor a, d");
        }
        0xAB => {
            // xor a, e");
        }
        0xAC => {
            // xor a, h");
        }
        0xAD => {
            // xor a, l");
        }
        0xAE => {
            // xor a, (hl)");
        }
        0xAF => {
            // xor a, a");
        }
        0xC3 => {
            // jp imm16
            return Instruction {
                op: Operation::JP_IMM16 {
                    imm16: console.memory().read_word(address + 1),
                },
                size: 3,
            };
        }

        _ => panic!(
            "DECODING : UNHANDLED INSTRUCTION ({:#04X}) at PC {:#06X}",
            instr, address
        ),
    }

    todo!(
        "DECODING : UNHANDLED INSTRUCTION ({:#04X}) at PC {:#06X}",
        instr,
        address
    )
}

pub fn get_instruction_assembly(instr: &Instruction) -> String {
    match &instr.op {
        Operation::NOP => String::from("nop"),
        Operation::LD_R16_IMM16 { r16, imm16 } => {
            format!("ld {}, {:#06X}", operand_to_str(r16), imm16)
        }
        Operation::LD_R8_IMM8 { r8, imm8 } => {
            format!("ld {}, {:#04X}", operand_to_str(r8), imm8)
        }
        Operation::DEC_R8 { r8 } => format!("dec {}", operand_to_str(r8)),
        Operation::JP_IMM16 { imm16 } => format!("jp {:#06X}", imm16),
        _ => todo!(),
    }
}

fn operand_to_str(operand: &Operand) -> &'static str {
    match operand {
        Operand::R8_A => "a",
        Operand::R8_B => "b",
        Operand::R8_C => "c",
        Operand::R8_D => "d",
        Operand::R8_E => "e",
        Operand::R8_H => "h",
        Operand::R8_L => "l",
        Operand::R16_BC => "bc",
        Operand::R16_DE => "de",
        Operand::R16_HL => "hl",
        Operand::R16_SP => "sp",
    }
}
