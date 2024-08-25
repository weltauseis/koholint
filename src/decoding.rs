use core::fmt;

use crate::gameboy::Gameboy;

#[allow(non_camel_case_types)]
#[derive(Debug, Clone)]
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
    R16_HLI,
    CC_NZ,
    IMM8(u8),
    IMM8_SIGNED(i8),
    IMM16(u16),
    PTR(Box<Operand>),
}

#[allow(non_camel_case_types)]
pub enum Operation {
    NOP,
    LD { dst: Operand, src: Operand },
    JP { addr: Operand },
    JR_CC { cc: Operand, offset_oprd: Operand },
    CALL { proc: Operand },
    DEC { x: Operand },
    INC { x: Operand },
    XOR { y: Operand },
    BIT { bit: u8, src: Operand },
}

pub struct Instruction {
    pub op: Operation,
    pub size: u16,
}

pub fn decode_next_instruction(console: &Gameboy) -> Instruction {
    let pc = console.cpu().read_program_counter();
    return decode_instruction(console, pc);
}

// https://meganesu.github.io/generate-gb-opcodes/
// https://gbdev.io/gb-opcodes/optables/ (seems to correct a few mistakes)
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
                op: Operation::LD {
                    dst: Operand::R16_BC,
                    src: Operand::IMM16(console.memory().read_word(address + 1)),
                },
                size: 3,
            };
        }
        0x05 => {
            // dec b
            return Instruction {
                op: Operation::DEC { x: Operand::R8_B },
                size: 1,
            };
        }
        0x06 => {
            // ld b, imm8
            return Instruction {
                op: Operation::LD {
                    dst: Operand::R8_B,
                    src: Operand::IMM8(console.memory().read_byte(address + 1)),
                },
                size: 2,
            };
        }
        0x0C => {
            // inc c
            return Instruction {
                op: Operation::INC { x: Operand::R8_C },
                size: 1,
            };
        }
        0x0D => {
            // dec c
            return Instruction {
                op: Operation::DEC { x: Operand::R8_C },
                size: 1,
            };
        }
        0x0E => {
            // ld c, imm8
            return Instruction {
                op: Operation::LD {
                    dst: Operand::R8_C,
                    src: Operand::IMM8(console.memory().read_byte(address + 1)),
                },
                size: 2,
            };
        }
        0x11 => {
            // ld de, imm16
            return Instruction {
                op: Operation::LD {
                    dst: Operand::R16_DE,
                    src: Operand::IMM16(console.memory().read_word(address + 1)),
                },
                size: 3,
            };
        }
        0x1A => {
            // ld a, (de)
            return Instruction {
                op: Operation::LD {
                    dst: Operand::R8_A,
                    src: Operand::PTR(Box::new(Operand::R16_DE)),
                },
                size: 1,
            };
        }
        0x20 => {
            // jr nz, imm8
            return Instruction {
                op: Operation::JR_CC {
                    cc: Operand::CC_NZ,
                    // the relative jump offset is signed
                    offset_oprd: Operand::IMM8_SIGNED(i8::from_le_bytes([console
                        .memory()
                        .read_byte(address + 1)])),
                },
                size: 2,
            };
        }
        0x21 => {
            // ld hl, imm16
            return Instruction {
                op: Operation::LD {
                    dst: Operand::R16_HL,
                    src: Operand::IMM16(console.memory().read_word(address + 1)),
                },
                size: 3,
            };
        }
        0x31 => {
            // ld sp, imm16
            return Instruction {
                op: Operation::LD {
                    dst: Operand::R16_SP,
                    src: Operand::IMM16(console.memory().read_word(address + 1)),
                },
                size: 3,
            };
        }
        0x32 => {
            // ld (hl-), a
            return Instruction {
                op: Operation::LD {
                    dst: Operand::PTR(Box::new(Operand::R16_HLD)),
                    src: Operand::R8_A,
                },
                size: 1,
            };
        }
        0x3E => {
            // ld a, imm8
            return Instruction {
                op: Operation::LD {
                    dst: Operand::R8_A,
                    src: Operand::IMM8(console.memory().read_byte(address + 1)),
                },
                size: 2,
            };
        }
        0x77 => {
            // ld (hl), a
            return Instruction {
                op: Operation::LD {
                    dst: Operand::PTR(Box::new(Operand::R16_HL)),
                    src: Operand::R8_A,
                },
                size: 1,
            };
        }
        0xA8 => {
            // xor a, b
        }
        0xA9 => {
            // xor a, c
        }
        0xAA => {
            // xor a, d
        }
        0xAB => {
            // xor a, e
        }
        0xAC => {
            // xor a, h
        }
        0xAD => {
            // xor a, l
        }
        0xAE => {
            // xor a, (hl)
        }
        0xAF => {
            // xor a, a
            return Instruction {
                op: Operation::XOR { y: Operand::R8_A },
                size: 1,
            };
        }
        0xC3 => {
            // jp imm16
            return Instruction {
                op: Operation::JP {
                    addr: Operand::IMM16(console.memory().read_word(address + 1)),
                },
                size: 3,
            };
        }
        0xCB => {
            //prefixed bit manipulation instructions
            let other_byte = console.memory().read_byte(address + 1);
            match other_byte {
                0x7C => {
                    // bit 7, h
                    return Instruction {
                        op: Operation::BIT {
                            bit: 7,
                            src: Operand::R8_H,
                        },
                        size: 2,
                    };
                }
                _ => panic!(
                    "DECODING : UNHANDLED INSTRUCTION (0xCB{:02X}) at PC {:#06X}",
                    other_byte, address
                ),
            }
        }
        0xCD => {
            // call imm16
            return Instruction {
                op: Operation::CALL {
                    proc: Operand::IMM16(console.memory().read_word(address + 1)),
                },
                size: 3,
            };
        }
        0xE0 => {
            // ld (imm8), a
            return Instruction {
                op: Operation::LD {
                    dst: Operand::PTR(Box::new(Operand::IMM8(
                        console.memory().read_byte(address + 1),
                    ))),
                    src: Operand::R8_A,
                },
                size: 2,
            };
        }
        0xE2 => {
            // ld (c), a
            return Instruction {
                op: Operation::LD {
                    dst: Operand::PTR(Box::new(Operand::R8_C)),
                    src: Operand::R8_A,
                },
                size: 1,
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

fn operand_to_string(operand: &Operand) -> String {
    match operand {
        Operand::R8_A => String::from("a"),
        Operand::R8_B => String::from("b"),
        Operand::R8_C => String::from("c"),
        Operand::R8_D => String::from("d"),
        Operand::R8_E => String::from("e"),
        Operand::R8_H => String::from("h"),
        Operand::R8_L => String::from("l"),
        Operand::R16_BC => String::from("bc"),
        Operand::R16_DE => String::from("de"),
        Operand::R16_HL => String::from("hl"),
        Operand::R16_SP => String::from("sp"),
        Operand::R16_HLD => String::from("hl-"),
        Operand::R16_HLI => String::from("hl+"),
        Operand::CC_NZ => String::from("nz"),
        Operand::IMM8(imm8) => format!("{:#04X}", imm8),
        Operand::IMM8_SIGNED(imm8) => format!("{}", imm8),
        Operand::IMM16(imm16) => format!("{:#06X}", imm16),
        Operand::PTR(ptr) => format!("({})", operand_to_string(ptr)),
    }
}

impl fmt::Display for Operand {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", operand_to_string(self))
    }
}

pub fn instruction_to_string(instr: &Instruction) -> String {
    match &instr.op {
        Operation::NOP => String::from("nop"),
        Operation::LD { dst, src } => format!("ld {dst}, {src}"),
        Operation::JP { addr } => format! {"jp {addr}"},
        Operation::JR_CC {
            cc,
            offset_oprd: offset,
        } => format!("jr {cc}, {offset}"),
        Operation::CALL { proc } => format!("call {proc}"),
        Operation::DEC { x } => format!("dec {x}"),
        Operation::INC { x } => format!("inc {x}"),
        Operation::XOR { y: x } => format!("xor a, {x}"),
        Operation::BIT { bit, src: r8 } => format!("bit {bit}, {r8}"),
    }
}

impl fmt::Display for Instruction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", instruction_to_string(self))
    }
}
