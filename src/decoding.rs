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
    R16_AF,
    R16_SP,
    R16_HLD,
    R16_HLI,
    CC_NZ,
    CC_Z,
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
    JR { offset_oprd: Operand },
    JR_CC { cc: Operand, offset_oprd: Operand },
    CALL { proc: Operand },
    RET,
    PUSH { reg: Operand },
    POP { reg: Operand },
    DEC { x: Operand },
    INC { x: Operand },
    ADD_8 { y: Operand },
    SUB { y: Operand },
    XOR { y: Operand },
    BIT { bit: u8, src: Operand },
    RL { x: Operand },
    RR { x: Operand },
    RLC { x: Operand },
    RRC { x: Operand },
    RLA,
    RRA,
    RLCA,
    RRCA,
    CP { y: Operand },
    DI,
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
    use Operand::*;
    use Operation::*;

    let instr = console.memory().read_byte(address); // instruction byte
    let imm8 = console.memory().read_byte(address + 1); // immediate byte, if needed
    let imm16 = console.memory().read_word(address + 1); // immediate word, if needed

    match instr {
        0x00 => {
            return Instruction { op: NOP, size: 1 };
        }
        0x01 => {
            // ld bc, imm16
            return Instruction {
                op: LD {
                    dst: R16_BC,
                    src: IMM16(imm16),
                },
                size: 3,
            };
        }
        0x03 => {
            // inc bc
            return Instruction {
                op: INC { x: R16_BC },
                size: 1,
            };
        }
        0x04 => {
            // inc b
            return Instruction {
                op: INC { x: R8_B },
                size: 1,
            };
        }
        0x05 => {
            // dec b
            return Instruction {
                op: DEC { x: R8_B },
                size: 1,
            };
        }
        0x06 => {
            // ld b, imm8
            return Instruction {
                op: LD {
                    dst: R8_B,
                    src: IMM8(imm8),
                },
                size: 2,
            };
        }
        0x0C => {
            // inc c
            return Instruction {
                op: INC { x: R8_C },
                size: 1,
            };
        }
        0x0D => {
            // dec c
            return Instruction {
                op: DEC { x: R8_C },
                size: 1,
            };
        }
        0x0E => {
            // ld c, imm8
            return Instruction {
                op: LD {
                    dst: R8_C,
                    src: IMM8(imm8),
                },
                size: 2,
            };
        }
        0x11 => {
            // ld de, imm16
            return Instruction {
                op: LD {
                    dst: R16_DE,
                    src: IMM16(imm16),
                },
                size: 3,
            };
        }
        0x13 => {
            // inc de
            return Instruction {
                op: INC { x: R16_DE },
                size: 1,
            };
        }
        0x15 => {
            // dec d
            return Instruction {
                op: DEC { x: R8_D },
                size: 1,
            };
        }
        0x16 => {
            // ld d, imm8
            return Instruction {
                op: LD {
                    dst: R8_D,
                    src: IMM8(imm8),
                },
                size: 2,
            };
        }
        0x17 => {
            // rla
            return Instruction { op: RLA, size: 1 };
        }
        0x18 => {
            // jr imm8
            return Instruction {
                op: JR {
                    offset_oprd: IMM8_SIGNED(i8::from_le_bytes([console
                        .memory()
                        .read_byte(address + 1)])),
                },
                size: 2,
            };
        }
        0x1A => {
            // ld a, (de)
            return Instruction {
                op: LD {
                    dst: R8_A,
                    src: PTR(Box::new(R16_DE)),
                },
                size: 1,
            };
        }
        0x1D => {
            // dec e
            return Instruction {
                op: DEC { x: R8_E },
                size: 1,
            };
        }
        0x1E => {
            // ld e, imm8
            return Instruction {
                op: LD {
                    dst: R8_E,
                    src: IMM8(imm8),
                },
                size: 2,
            };
        }
        0x20 => {
            // jr nz, imm8
            return Instruction {
                op: JR_CC {
                    cc: CC_NZ,
                    // the relative jump offset is signed
                    offset_oprd: IMM8_SIGNED(i8::from_le_bytes([console
                        .memory()
                        .read_byte(address + 1)])),
                },
                size: 2,
            };
        }
        0x21 => {
            // ld hl, imm16
            return Instruction {
                op: LD {
                    dst: R16_HL,
                    src: IMM16(imm16),
                },
                size: 3,
            };
        }
        0x22 => {
            // ld (hl+), a
            return Instruction {
                op: LD {
                    dst: PTR(Box::new(R16_HLI)),
                    src: R8_A,
                },
                size: 1,
            };
        }
        0x23 => {
            // inc hl
            return Instruction {
                op: INC { x: R16_HL },
                size: 1,
            };
        }
        0x24 => {
            // inc h
            return Instruction {
                op: INC { x: R8_H },
                size: 1,
            };
        }
        0x28 => {
            // jr z, imm8
            return Instruction {
                op: JR_CC {
                    cc: CC_Z,
                    offset_oprd: IMM8_SIGNED(i8::from_le_bytes([console
                        .memory()
                        .read_byte(address + 1)])),
                },
                size: 2,
            };
        }
        0x2E => {
            // ld l, imm8
            return Instruction {
                op: LD {
                    dst: R8_L,
                    src: IMM8(imm8),
                },
                size: 2,
            };
        }
        0x31 => {
            // ld sp, imm16
            return Instruction {
                op: LD {
                    dst: R16_SP,
                    src: IMM16(imm16),
                },
                size: 3,
            };
        }
        0x32 => {
            // ld (hl-), a
            return Instruction {
                op: LD {
                    dst: PTR(Box::new(R16_HLD)),
                    src: R8_A,
                },
                size: 1,
            };
        }
        0x33 => {
            // inc sp
            return Instruction {
                op: INC { x: R16_SP },
                size: 1,
            };
        }
        0x3D => {
            // dec a
            return Instruction {
                op: DEC { x: R8_A },
                size: 1,
            };
        }
        0x3E => {
            // ld a, imm8
            return Instruction {
                op: LD {
                    dst: R8_A,
                    src: IMM8(imm8),
                },
                size: 2,
            };
        }
        0x4F => {
            // ld c, a
            return Instruction {
                op: LD {
                    dst: R8_C,
                    src: R8_A,
                },
                size: 1,
            };
        }
        0x57 => {
            // ld d, a
            return Instruction {
                op: LD {
                    dst: R8_D,
                    src: R8_A,
                },
                size: 1,
            };
        }
        0x67 => {
            // ld h, a
            return Instruction {
                op: LD {
                    dst: R8_H,
                    src: R8_A,
                },
                size: 1,
            };
        }
        0x77 => {
            // ld (hl), a
            return Instruction {
                op: LD {
                    dst: PTR(Box::new(R16_HL)),
                    src: R8_A,
                },
                size: 1,
            };
        }
        0x78 => {
            // ld a, b
            return Instruction {
                op: LD {
                    dst: R8_A,
                    src: R8_B,
                },
                size: 1,
            };
        }
        0x7B => {
            // ld a, e
            return Instruction {
                op: LD {
                    dst: R8_A,
                    src: R8_E,
                },
                size: 1,
            };
        }
        0x7C => {
            // ld a, h
            return Instruction {
                op: LD {
                    dst: R8_A,
                    src: R8_H,
                },
                size: 1,
            };
        }
        0x7D => {
            // ld a, l
            return Instruction {
                op: LD {
                    dst: R8_A,
                    src: R8_L,
                },
                size: 1,
            };
        }
        0x86 => {
            // add a, (hl)
            return Instruction {
                op: ADD_8 {
                    y: PTR(Box::new(R16_HL)),
                },
                size: 1,
            };
        }
        0x90 => {
            // sub a, b
            return Instruction {
                op: SUB { y: R8_B },
                size: 1,
            };
        }
        0xA8 => {
            // xor a, b
            return Instruction {
                op: XOR { y: R8_B },
                size: 1,
            };
        }
        0xA9 => {
            // xor a, c
            return Instruction {
                op: XOR { y: R8_C },
                size: 1,
            };
        }
        0xAA => {
            // xor a, d
            return Instruction {
                op: XOR { y: R8_D },
                size: 1,
            };
        }
        0xAB => {
            // xor a, e
            return Instruction {
                op: XOR { y: R8_E },
                size: 1,
            };
        }
        0xAC => {
            // xor a, h
            return Instruction {
                op: XOR { y: R8_H },
                size: 1,
            };
        }
        0xAD => {
            // xor a, l
            return Instruction {
                op: XOR { y: R8_L },
                size: 1,
            };
        }
        0xAE => {
            // xor a, (hl)
            return Instruction {
                op: XOR {
                    y: PTR(Box::new(R16_HL)),
                },
                size: 1,
            };
        }
        0xAF => {
            // xor a, a
            return Instruction {
                op: XOR { y: R8_A },
                size: 1,
            };
        }
        0xBE => {
            // cp a, (hl)
            return Instruction {
                op: CP {
                    y: PTR(Box::new(R16_HL)),
                },
                size: 1,
            };
        }
        0xC1 => {
            // pop bc
            return Instruction {
                op: POP { reg: R16_BC },
                size: 1,
            };
        }
        0xC3 => {
            // jp imm16
            return Instruction {
                op: JP { addr: IMM16(imm16) },
                size: 3,
            };
        }
        0xC5 => {
            // push bc
            return Instruction {
                op: PUSH { reg: R16_BC },
                size: 1,
            };
        }
        0xC9 => {
            // ret
            return Instruction { op: RET, size: 1 };
        }
        0xCB => {
            //prefixed bit manipulation instructions
            match imm8 {
                0x11 => {
                    // rl c
                    return Instruction {
                        op: RL { x: R8_C },
                        size: 2,
                    };
                }
                0x7C => {
                    // bit 7, h
                    return Instruction {
                        op: BIT { bit: 7, src: R8_H },
                        size: 2,
                    };
                }
                _ => panic!(
                    "DECODING : UNHANDLED INSTRUCTION (0xCB{:02X}) at PC {:#06X}",
                    imm8, address
                ),
            }
        }
        0xCD => {
            // call imm16
            return Instruction {
                op: CALL { proc: IMM16(imm16) },
                size: 3,
            };
        }
        0xE0 => {
            // ld (imm8), a
            return Instruction {
                op: LD {
                    dst: PTR(Box::new(IMM8(imm8))),
                    src: R8_A,
                },
                size: 2,
            };
        }
        0xE2 => {
            // ld (c), a
            return Instruction {
                op: LD {
                    dst: PTR(Box::new(R8_C)),
                    src: R8_A,
                },
                size: 1,
            };
        }
        0xEA => {
            // ld (imm16), a
            return Instruction {
                op: LD {
                    dst: PTR(Box::new(IMM16(imm16))),
                    src: R8_A,
                },
                size: 3,
            };
        }
        0xF0 => {
            // ld a, (imm8)
            return Instruction {
                op: LD {
                    dst: R8_A,
                    src: PTR(Box::new(IMM8(imm8))),
                },
                size: 2,
            };
        }
        0xF3 => {
            // di
            return Instruction { op: DI, size: 1 };
        }
        0xFE => {
            // cp imm8
            return Instruction {
                op: CP { y: IMM8(imm8) },
                size: 2,
            };
        }

        _ => panic!(
            "DECODING : UNHANDLED INSTRUCTION ({:#04X}) at PC {:#06X}",
            instr, address
        ),
    }
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
        Operand::R16_AF => String::from("af"),
        Operand::R16_SP => String::from("sp"),
        Operand::R16_HLD => String::from("hl-"),
        Operand::R16_HLI => String::from("hl+"),
        Operand::CC_NZ => String::from("nz"),
        Operand::CC_Z => String::from("z"),
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
        Operation::JR {
            offset_oprd: offset,
        } => format!("jr {offset}"),
        Operation::JR_CC {
            cc,
            offset_oprd: offset,
        } => format!("jr {cc}, {offset}"),
        Operation::CALL { proc } => format!("call {proc}"),
        Operation::RET => String::from("ret"),
        Operation::PUSH { reg: word } => format!("push {word}"),
        Operation::POP { reg: word } => format!("pop {word}"),
        Operation::DEC { x } => format!("dec {x}"),
        Operation::INC { x } => format!("inc {x}"),
        Operation::ADD_8 { y } => format!("add a, {y}"),
        Operation::SUB { y } => format!("sub a, {y}"),
        Operation::XOR { y: x } => format!("xor a, {x}"),
        Operation::BIT { bit, src: r8 } => format!("bit {bit}, {r8}"),
        Operation::RL { x } => format!("rl {x}"),
        Operation::RR { x } => format!("rr {x}"),
        Operation::RLC { x } => format!("rlc {x}"),
        Operation::RRC { x } => format!("rrc {x}"),
        Operation::RLA => String::from("rla"),
        Operation::RRA => String::from("rra"),
        Operation::RLCA => String::from("rlca"),
        Operation::RRCA => String::from("rrca"),
        Operation::CP { y } => format!("cp {y}"),
        Operation::DI => String::from("di"),
    }
}

impl fmt::Display for Instruction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", instruction_to_string(self))
    }
}
