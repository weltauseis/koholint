use core::fmt;
use std::path::Display;

use crate::{gameboy::Gameboy, memory::Memory};

#[derive(Debug, Clone, Copy)]
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
    R16_HLD,
    CC_NZ,
}
pub enum Operation {
    NOP,
    LD_R16_IMM16 { r16: Operand, imm16: u16 },
    LD_R8_IMM8 { r8: Operand, imm8: u8 },
    LD_PTR_R8 { ptr: Operand, r8: Operand },
    DEC_R8 { r8: Operand },
    JP_IMM16 { imm16: u16 },
    JR_CC_R8 { cc: Operand, imm8: i8 },
    XOR_A_R8 { r8: Operand },
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

impl fmt::Display for Operand {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", operand_to_str(self))
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
            return Instruction {
                op: Operation::JR_CC_R8 {
                    cc: Operand::CC_NZ,
                    // the relative jump offset is signed
                    imm8: i8::from_le_bytes([console.memory().read_byte(address + 1)]),
                },
                size: 2,
            };
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
            // ld sp, imm16
            return Instruction {
                op: Operation::LD_R16_IMM16 {
                    r16: Operand::R16_SP,
                    imm16: console.memory().read_word(address + 1),
                },
                size: 3,
            };
        }
        0x32 => {
            // ld (hl-), a
            return Instruction {
                op: Operation::LD_PTR_R8 {
                    ptr: Operand::R16_HLD,
                    r8: Operand::R8_A,
                },
                size: 1,
            };
        }
        0x3E => {
            // ld a, imm8
            return Instruction {
                op: Operation::LD_R8_IMM8 {
                    r8: Operand::R8_A,
                    imm8: console.memory().read_byte(address + 1),
                },
                size: 2,
            };
        }
        0xA8 => {
            // xor a, b
        }
        0xA9 => {
            // xor a, c
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
            // xor a, a
            return Instruction {
                op: Operation::XOR_A_R8 { r8: Operand::R8_A },
                size: 1,
            };
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
            format!("ld {r16}, {:#06X}", imm16)
        }
        Operation::LD_R8_IMM8 { r8, imm8 } => {
            format!("ld {r8}, {:#04X}", imm8)
        }
        Operation::LD_PTR_R8 { ptr, r8 } => format!("ld ({ptr}), {r8}"),
        Operation::DEC_R8 { r8 } => format!("dec {r8}"),
        Operation::JP_IMM16 { imm16 } => format!("jp {:#06X}", imm16),
        Operation::JR_CC_R8 { cc, imm8 } => format!("jr {cc}, {}", imm8),
        Operation::XOR_A_R8 { r8 } => format!("xor a, {r8}"),
        _ => todo!("get_instruction_assembly : unhandled operation type"),
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
        Operand::R16_HLD => "hl-",
        Operand::CC_NZ => "nz",
    }
}
