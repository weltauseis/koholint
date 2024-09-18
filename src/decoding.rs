use core::fmt;

use crate::{
    error::{EmulationError, EmulationErrorType},
    gameboy::Gameboy,
};

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
    CC_NC,
    CC_C,
    IMM8(u8),
    IMM8_SIGNED(i8),
    IMM16(u16),
    PTR(Box<Operand>),
}

#[allow(non_camel_case_types)]
#[derive(Debug)]
pub enum Operation {
    NOP,
    LD { dst: Operand, src: Operand },
    JP { addr: Operand },
    JR { offset_oprd: Operand },
    JR_CC { cc: Operand, offset_oprd: Operand },
    CALL { proc: Operand },
    CALL_CC { cc: Operand, proc: Operand },
    RST { addr: Operand },
    RET,
    RET_CC { cc: Operand },
    PUSH { reg: Operand },
    POP { reg: Operand },
    DEC { x: Operand },
    INC { x: Operand },
    ADD { x: Operand, y: Operand },
    SUB { y: Operand },
    OR { y: Operand },
    XOR { y: Operand },
    AND { y: Operand },
    BIT { bit: u8, src: Operand },
    SWAP { x: Operand },
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
    EI,
    CPL,
}

#[derive(Debug)]
pub struct Instruction {
    pub op: Operation,
    pub size: u16,
    pub cycles: u64,
    pub branch_cycles: Option<u64>, // some instructions (jumps) have different cycles whether they branch or not
}

pub fn decode_next_instruction(console: &Gameboy) -> Result<Instruction, EmulationError> {
    let pc = console.cpu().read_program_counter();
    return decode_instruction(console, pc);
}

// https://meganesu.github.io/generate-gb-opcodes/
// https://gbdev.io/gb-opcodes/optables/ (seems to correct a few mistakes)
pub fn decode_instruction(console: &Gameboy, address: u16) -> Result<Instruction, EmulationError> {
    use Operand::*;
    use Operation::*;

    let instr = console.memory().read_byte(address); // instruction byte
    let imm8 = console.memory().read_byte(address + 1); // immediate byte, if needed
    let imm16 = console.memory().read_word(address + 1); // immediate word, if needed

    match instr {
        0x00 => {
            return Ok(Instruction {
                op: NOP,
                size: 1,
                cycles: 4,
                branch_cycles: None,
            });
        }
        0x01 => {
            // ld bc, imm16
            return Ok(Instruction {
                op: LD {
                    dst: R16_BC,
                    src: IMM16(imm16),
                },
                size: 3,
                cycles: 12,
                branch_cycles: None,
            });
        }
        0x02 => {
            // ld (bc), a
            return Ok(Instruction {
                op: LD {
                    dst: PTR(Box::new(R16_BC)),
                    src: R8_A,
                },
                size: 1,
                cycles: 8,
                branch_cycles: None,
            });
        }
        0x03 => {
            // inc bc
            return Ok(Instruction {
                op: INC { x: R16_BC },
                size: 1,
                cycles: 8,
                branch_cycles: None,
            });
        }
        0x04 => {
            // inc b
            return Ok(Instruction {
                op: INC { x: R8_B },
                size: 1,
                cycles: 4,
                branch_cycles: None,
            });
        }
        0x05 => {
            // dec b
            return Ok(Instruction {
                op: DEC { x: R8_B },
                size: 1,
                cycles: 4,
                branch_cycles: None,
            });
        }
        0x06 => {
            // ld b, imm8
            return Ok(Instruction {
                op: LD {
                    dst: R8_B,
                    src: IMM8(imm8),
                },
                size: 2,
                cycles: 8,
                branch_cycles: None,
            });
        }
        0x09 => {
            // add hl, bc
            return Ok(Instruction {
                op: ADD {
                    x: R16_HL,
                    y: R16_BC,
                },
                size: 1,
                cycles: 8,
                branch_cycles: None,
            });
        }
        0x0A => {
            // ld a, (bc)
            return Ok(Instruction {
                op: LD {
                    dst: R8_A,
                    src: PTR(Box::new(R16_BC)),
                },
                size: 1,
                cycles: 8,
                branch_cycles: None,
            });
        }
        0x0B => {
            // dec bc
            return Ok(Instruction {
                op: DEC { x: R16_BC },
                size: 1,
                cycles: 8,
                branch_cycles: None,
            });
        }
        0x0C => {
            // inc c
            return Ok(Instruction {
                op: INC { x: R8_C },
                size: 1,
                cycles: 4,
                branch_cycles: None,
            });
        }
        0x0D => {
            // dec c
            return Ok(Instruction {
                op: DEC { x: R8_C },
                size: 1,
                cycles: 4,
                branch_cycles: None,
            });
        }
        0x0E => {
            // ld c, imm8
            return Ok(Instruction {
                op: LD {
                    dst: R8_C,
                    src: IMM8(imm8),
                },
                size: 2,
                cycles: 8,
                branch_cycles: None,
            });
        }
        0x11 => {
            // ld de, imm16
            return Ok(Instruction {
                op: LD {
                    dst: R16_DE,
                    src: IMM16(imm16),
                },
                size: 3,
                cycles: 12,
                branch_cycles: None,
            });
        }
        0x12 => {
            // ld (de), a
            return Ok(Instruction {
                op: LD {
                    dst: PTR(Box::new(R16_DE)),
                    src: R8_A,
                },
                size: 1,
                cycles: 8,
                branch_cycles: None,
            });
        }
        0x13 => {
            // inc de
            return Ok(Instruction {
                op: INC { x: R16_DE },
                size: 1,
                cycles: 8,
                branch_cycles: None,
            });
        }
        0x14 => {
            // inc d
            return Ok(Instruction {
                op: INC { x: R8_D },
                size: 1,
                cycles: 4,
                branch_cycles: None,
            });
        }
        0x15 => {
            // dec d
            return Ok(Instruction {
                op: DEC { x: R8_D },
                size: 1,
                cycles: 4,
                branch_cycles: None,
            });
        }
        0x16 => {
            // ld d, imm8
            return Ok(Instruction {
                op: LD {
                    dst: R8_D,
                    src: IMM8(imm8),
                },
                size: 2,
                cycles: 8,
                branch_cycles: None,
            });
        }
        0x17 => {
            // rla
            return Ok(Instruction {
                op: RLA,
                size: 1,
                cycles: 8,
                branch_cycles: None,
            });
        }
        0x18 => {
            // jr imm8
            return Ok(Instruction {
                op: JR {
                    offset_oprd: IMM8_SIGNED(i8::from_le_bytes([console
                        .memory()
                        .read_byte(address + 1)])),
                },
                size: 2,
                cycles: 12,
                branch_cycles: None,
            });
        }
        0x19 => {
            // add hl, de
            return Ok(Instruction {
                op: ADD {
                    x: R16_HL,
                    y: R16_DE,
                },
                size: 1,
                cycles: 8,
                branch_cycles: None,
            });
        }
        0x1A => {
            // ld a, (de)
            return Ok(Instruction {
                op: LD {
                    dst: R8_A,
                    src: PTR(Box::new(R16_DE)),
                },
                size: 1,
                cycles: 8,
                branch_cycles: None,
            });
        }
        0x1B => {
            // dec de
            return Ok(Instruction {
                op: DEC { x: R16_DE },
                size: 1,
                cycles: 8,
                branch_cycles: None,
            });
        }
        0x1C => {
            // inc e
            return Ok(Instruction {
                op: INC { x: R8_E },
                size: 1,
                cycles: 4,
                branch_cycles: None,
            });
        }
        0x1D => {
            // dec e
            return Ok(Instruction {
                op: DEC { x: R8_E },
                size: 1,
                cycles: 8,
                branch_cycles: None,
            });
        }
        0x1E => {
            // ld e, imm8
            return Ok(Instruction {
                op: LD {
                    dst: R8_E,
                    src: IMM8(imm8),
                },
                size: 2,
                cycles: 8,
                branch_cycles: None,
            });
        }
        0x20 => {
            // jr nz, imm8
            return Ok(Instruction {
                op: JR_CC {
                    cc: CC_NZ,
                    // the relative jump offset is signed
                    offset_oprd: IMM8_SIGNED(i8::from_le_bytes([console
                        .memory()
                        .read_byte(address + 1)])),
                },
                size: 2,
                cycles: 8,
                branch_cycles: Some(12),
            });
        }
        0x21 => {
            // ld hl, imm16
            return Ok(Instruction {
                op: LD {
                    dst: R16_HL,
                    src: IMM16(imm16),
                },
                size: 3,
                cycles: 12,
                branch_cycles: None,
            });
        }
        0x22 => {
            // ld (hl+), a
            return Ok(Instruction {
                op: LD {
                    dst: PTR(Box::new(R16_HLI)),
                    src: R8_A,
                },
                size: 1,
                cycles: 8,
                branch_cycles: None,
            });
        }
        0x23 => {
            // inc hl
            return Ok(Instruction {
                op: INC { x: R16_HL },
                size: 1,
                cycles: 8,
                branch_cycles: None,
            });
        }
        0x24 => {
            // inc h
            return Ok(Instruction {
                op: INC { x: R8_H },
                size: 1,
                cycles: 4,
                branch_cycles: None,
            });
        }
        0x25 => {
            // dec h
            return Ok(Instruction {
                op: DEC { x: R8_H },
                size: 1,
                cycles: 4,
                branch_cycles: None,
            });
        }
        0x26 => {
            // ld h, imm8
            return Ok(Instruction {
                op: LD {
                    dst: R8_H,
                    src: IMM8(imm8),
                },
                size: 2,
                cycles: 8,
                branch_cycles: None,
            });
        }
        0x28 => {
            // jr z, imm8
            return Ok(Instruction {
                op: JR_CC {
                    cc: CC_Z,
                    offset_oprd: IMM8_SIGNED(i8::from_le_bytes([console
                        .memory()
                        .read_byte(address + 1)])),
                },
                size: 2,
                cycles: 8,
                branch_cycles: Some(12),
            });
        }
        0x29 => {
            // add hl, hl
            return Ok(Instruction {
                op: ADD {
                    x: R16_HL,
                    y: R16_HL,
                },
                size: 1,
                cycles: 8,
                branch_cycles: None,
            });
        }
        0x2A => {
            // ld a, (hl+)
            return Ok(Instruction {
                op: LD {
                    dst: R8_A,
                    src: PTR(Box::new(R16_HLI)),
                },
                size: 1,
                cycles: 8,
                branch_cycles: None,
            });
        }
        0x2B => {
            // dec hl
            return Ok(Instruction {
                op: DEC { x: R16_HL },
                size: 1,
                cycles: 8,
                branch_cycles: None,
            });
        }
        0x2C => {
            // inc l
            return Ok(Instruction {
                op: INC { x: R8_L },
                size: 1,
                cycles: 4,
                branch_cycles: None,
            });
        }
        0x2D => {
            // dec l
            return Ok(Instruction {
                op: DEC { x: R8_L },
                size: 1,
                cycles: 4,
                branch_cycles: None,
            });
        }
        0x2E => {
            // ld l, imm8
            return Ok(Instruction {
                op: LD {
                    dst: R8_L,
                    src: IMM8(imm8),
                },
                size: 2,
                cycles: 8,
                branch_cycles: None,
            });
        }
        0x2F => {
            // cpl
            return Ok(Instruction {
                op: CPL,
                size: 1,
                cycles: 4,
                branch_cycles: None,
            });
        }
        0x31 => {
            // ld sp, imm16
            return Ok(Instruction {
                op: LD {
                    dst: R16_SP,
                    src: IMM16(imm16),
                },
                size: 3,
                cycles: 12,
                branch_cycles: None,
            });
        }
        0x32 => {
            // ld (hl-), a
            return Ok(Instruction {
                op: LD {
                    dst: PTR(Box::new(R16_HLD)),
                    src: R8_A,
                },
                size: 1,
                cycles: 12,
                branch_cycles: None,
            });
        }
        0x33 => {
            // inc sp
            return Ok(Instruction {
                op: INC { x: R16_SP },
                size: 1,
                cycles: 8,
                branch_cycles: None,
            });
        }
        0x34 => {
            // inc (hl)
            return Ok(Instruction {
                op: INC {
                    x: PTR(Box::new(R16_HL)),
                },
                size: 1,
                cycles: 12,
                branch_cycles: None,
            });
        }
        0x35 => {
            // dec (hl)
            return Ok(Instruction {
                op: DEC {
                    x: PTR(Box::new(R16_HL)),
                },
                size: 1,
                cycles: 12,
                branch_cycles: None,
            });
        }
        0x36 => {
            // ld (hl), imm8
            return Ok(Instruction {
                op: LD {
                    dst: PTR(Box::new(R16_HL)),
                    src: IMM8(imm8),
                },
                size: 2,
                cycles: 12,
                branch_cycles: None,
            });
        }
        0x39 => {
            // add hl, sp
            return Ok(Instruction {
                op: ADD {
                    x: R16_HL,
                    y: R16_SP,
                },
                size: 1,
                cycles: 8,
                branch_cycles: None,
            });
        }
        0x3A => {
            // ld a, (hl-)
            return Ok(Instruction {
                op: LD {
                    dst: R8_A,
                    src: PTR(Box::new(R16_HLD)),
                },
                size: 1,
                cycles: 8,
                branch_cycles: None,
            });
        }
        0x3B => {
            // dec sp
            return Ok(Instruction {
                op: DEC { x: R16_SP },
                size: 1,
                cycles: 8,
                branch_cycles: None,
            });
        }
        0x3C => {
            // inc a
            return Ok(Instruction {
                op: INC { x: R8_A },
                size: 1,
                cycles: 4,
                branch_cycles: None,
            });
        }
        0x3D => {
            // dec a
            return Ok(Instruction {
                op: DEC { x: R8_A },
                size: 1,
                cycles: 4,
                branch_cycles: None,
            });
        }
        0x3E => {
            // ld a, imm8
            return Ok(Instruction {
                op: LD {
                    dst: R8_A,
                    src: IMM8(imm8),
                },
                size: 2,
                cycles: 8,
                branch_cycles: None,
            });
        }
        0x40 => {
            // ld b, b
            return Ok(Instruction {
                op: LD {
                    dst: R8_B,
                    src: R8_B,
                },
                size: 1,
                cycles: 4,
                branch_cycles: None,
            });
        }
        0x41 => {
            // ld b, c
            return Ok(Instruction {
                op: LD {
                    dst: R8_B,
                    src: R8_C,
                },
                size: 1,
                cycles: 4,
                branch_cycles: None,
            });
        }
        0x42 => {
            // ld b, d
            return Ok(Instruction {
                op: LD {
                    dst: R8_B,
                    src: R8_D,
                },
                size: 1,
                cycles: 4,
                branch_cycles: None,
            });
        }
        0x43 => {
            // ld b, e
            return Ok(Instruction {
                op: LD {
                    dst: R8_B,
                    src: R8_E,
                },
                size: 1,
                cycles: 4,
                branch_cycles: None,
            });
        }
        0x44 => {
            // ld b, h
            return Ok(Instruction {
                op: LD {
                    dst: R8_B,
                    src: R8_H,
                },
                size: 1,
                cycles: 4,
                branch_cycles: None,
            });
        }
        0x45 => {
            // ld b, l
            return Ok(Instruction {
                op: LD {
                    dst: R8_B,
                    src: R8_L,
                },
                size: 1,
                cycles: 4,
                branch_cycles: None,
            });
        }
        0x46 => {
            // ld b, (hl)
            return Ok(Instruction {
                op: LD {
                    dst: R8_B,
                    src: PTR(Box::new(R16_HL)),
                },
                size: 1,
                cycles: 8,
                branch_cycles: None,
            });
        }
        0x47 => {
            // ld b, a
            return Ok(Instruction {
                op: LD {
                    dst: R8_B,
                    src: R8_A,
                },
                size: 1,
                cycles: 4,
                branch_cycles: None,
            });
        }
        0x48 => {
            // ld c, b
            return Ok(Instruction {
                op: LD {
                    dst: R8_C,
                    src: R8_B,
                },
                size: 1,
                cycles: 4,
                branch_cycles: None,
            });
        }
        0x49 => {
            // ld c, c
            return Ok(Instruction {
                op: LD {
                    dst: R8_C,
                    src: R8_C,
                },
                size: 1,
                cycles: 4,
                branch_cycles: None,
            });
        }
        0x4A => {
            // ld c, d
            return Ok(Instruction {
                op: LD {
                    dst: R8_C,
                    src: R8_D,
                },
                size: 1,
                cycles: 4,
                branch_cycles: None,
            });
        }
        0x4B => {
            // ld c, e
            return Ok(Instruction {
                op: LD {
                    dst: R8_C,
                    src: R8_E,
                },
                size: 1,
                cycles: 4,
                branch_cycles: None,
            });
        }
        0x4C => {
            // ld c, h
            return Ok(Instruction {
                op: LD {
                    dst: R8_C,
                    src: R8_H,
                },
                size: 1,
                cycles: 4,
                branch_cycles: None,
            });
        }
        0x4D => {
            // ld c, l
            return Ok(Instruction {
                op: LD {
                    dst: R8_C,
                    src: R8_L,
                },
                size: 1,
                cycles: 4,
                branch_cycles: None,
            });
        }
        0x4E => {
            // ld c, (hl)
            return Ok(Instruction {
                op: LD {
                    dst: R8_C,
                    src: PTR(Box::new(R16_HL)),
                },
                size: 1,
                cycles: 8,
                branch_cycles: None,
            });
        }
        0x4F => {
            // ld c, a
            return Ok(Instruction {
                op: LD {
                    dst: R8_C,
                    src: R8_A,
                },
                size: 1,
                cycles: 4,
                branch_cycles: None,
            });
        }
        0x50 => {
            // ld d, b
            return Ok(Instruction {
                op: LD {
                    dst: R8_D,
                    src: R8_B,
                },
                size: 1,
                cycles: 4,
                branch_cycles: None,
            });
        }
        0x51 => {
            // ld d, c
            return Ok(Instruction {
                op: LD {
                    dst: R8_D,
                    src: R8_C,
                },
                size: 1,
                cycles: 4,
                branch_cycles: None,
            });
        }
        0x52 => {
            // ld d, d
            return Ok(Instruction {
                op: LD {
                    dst: R8_D,
                    src: R8_D,
                },
                size: 1,
                cycles: 4,
                branch_cycles: None,
            });
        }
        0x53 => {
            // ld d, e
            return Ok(Instruction {
                op: LD {
                    dst: R8_D,
                    src: R8_E,
                },
                size: 1,
                cycles: 4,
                branch_cycles: None,
            });
        }
        0x54 => {
            // ld d, h
            return Ok(Instruction {
                op: LD {
                    dst: R8_D,
                    src: R8_H,
                },
                size: 1,
                cycles: 4,
                branch_cycles: None,
            });
        }
        0x55 => {
            // ld d, l
            return Ok(Instruction {
                op: LD {
                    dst: R8_D,
                    src: R8_L,
                },
                size: 1,
                cycles: 4,
                branch_cycles: None,
            });
        }
        0x56 => {
            // ld d, (hl)
            return Ok(Instruction {
                op: LD {
                    dst: R8_D,
                    src: PTR(Box::new(R16_HL)),
                },
                size: 1,
                cycles: 8,
                branch_cycles: None,
            });
        }
        0x57 => {
            // ld d, a
            return Ok(Instruction {
                op: LD {
                    dst: R8_D,
                    src: R8_A,
                },
                size: 1,
                cycles: 4,
                branch_cycles: None,
            });
        }
        0x58 => {
            // ld e, b
            return Ok(Instruction {
                op: LD {
                    dst: R8_E,
                    src: R8_B,
                },
                size: 1,
                cycles: 4,
                branch_cycles: None,
            });
        }
        0x59 => {
            // ld e, c
            return Ok(Instruction {
                op: LD {
                    dst: R8_E,
                    src: R8_C,
                },
                size: 1,
                cycles: 4,
                branch_cycles: None,
            });
        }
        0x5A => {
            // ld e, d
            return Ok(Instruction {
                op: LD {
                    dst: R8_E,
                    src: R8_D,
                },
                size: 1,
                cycles: 4,
                branch_cycles: None,
            });
        }
        0x5B => {
            // ld e, e
            return Ok(Instruction {
                op: LD {
                    dst: R8_E,
                    src: R8_E,
                },
                size: 1,
                cycles: 4,
                branch_cycles: None,
            });
        }
        0x5C => {
            // ld e, h
            return Ok(Instruction {
                op: LD {
                    dst: R8_E,
                    src: R8_H,
                },
                size: 1,
                cycles: 4,
                branch_cycles: None,
            });
        }
        0x5D => {
            // ld e, l
            return Ok(Instruction {
                op: LD {
                    dst: R8_E,
                    src: R8_L,
                },
                size: 1,
                cycles: 4,
                branch_cycles: None,
            });
        }
        0x5E => {
            // ld e, (hl)
            return Ok(Instruction {
                op: LD {
                    dst: R8_E,
                    src: PTR(Box::new(R16_HL)),
                },
                size: 1,
                cycles: 8,
                branch_cycles: None,
            });
        }
        0x5F => {
            // ld e, a
            return Ok(Instruction {
                op: LD {
                    dst: R8_E,
                    src: R8_A,
                },
                size: 1,
                cycles: 4,
                branch_cycles: None,
            });
        }
        0x60 => {
            // ld h, b
            return Ok(Instruction {
                op: LD {
                    dst: R8_H,
                    src: R8_B,
                },
                size: 1,
                cycles: 4,
                branch_cycles: None,
            });
        }
        0x61 => {
            // ld h, c
            return Ok(Instruction {
                op: LD {
                    dst: R8_H,
                    src: R8_C,
                },
                size: 1,
                cycles: 4,
                branch_cycles: None,
            });
        }
        0x62 => {
            // ld h, d
            return Ok(Instruction {
                op: LD {
                    dst: R8_H,
                    src: R8_D,
                },
                size: 1,
                cycles: 4,
                branch_cycles: None,
            });
        }
        0x63 => {
            // ld h, e
            return Ok(Instruction {
                op: LD {
                    dst: R8_H,
                    src: R8_E,
                },
                size: 1,
                cycles: 4,
                branch_cycles: None,
            });
        }
        0x64 => {
            // ld h, h
            return Ok(Instruction {
                op: LD {
                    dst: R8_H,
                    src: R8_H,
                },
                size: 1,
                cycles: 4,
                branch_cycles: None,
            });
        }
        0x65 => {
            // ld h, l
            return Ok(Instruction {
                op: LD {
                    dst: R8_H,
                    src: R8_L,
                },
                size: 1,
                cycles: 4,
                branch_cycles: None,
            });
        }
        0x66 => {
            // ld h, (hl)
            return Ok(Instruction {
                op: LD {
                    dst: R8_H,
                    src: PTR(Box::new(R16_HL)),
                },
                size: 1,
                cycles: 8,
                branch_cycles: None,
            });
        }
        0x67 => {
            // ld h, a
            return Ok(Instruction {
                op: LD {
                    dst: R8_H,
                    src: R8_A,
                },
                size: 1,
                cycles: 4,
                branch_cycles: None,
            });
        }
        0x68 => {
            // ld l, b
            return Ok(Instruction {
                op: LD {
                    dst: R8_L,
                    src: R8_B,
                },
                size: 1,
                cycles: 4,
                branch_cycles: None,
            });
        }
        0x69 => {
            // ld l, c
            return Ok(Instruction {
                op: LD {
                    dst: R8_L,
                    src: R8_C,
                },
                size: 1,
                cycles: 4,
                branch_cycles: None,
            });
        }
        0x6A => {
            // ld l, d
            return Ok(Instruction {
                op: LD {
                    dst: R8_L,
                    src: R8_D,
                },
                size: 1,
                cycles: 4,
                branch_cycles: None,
            });
        }
        0x6B => {
            // ld l, e
            return Ok(Instruction {
                op: LD {
                    dst: R8_L,
                    src: R8_E,
                },
                size: 1,
                cycles: 4,
                branch_cycles: None,
            });
        }
        0x6C => {
            // ld l, h
            return Ok(Instruction {
                op: LD {
                    dst: R8_L,
                    src: R8_H,
                },
                size: 1,
                cycles: 4,
                branch_cycles: None,
            });
        }
        0x6D => {
            // ld l, l
            return Ok(Instruction {
                op: LD {
                    dst: R8_L,
                    src: R8_L,
                },
                size: 1,
                cycles: 4,
                branch_cycles: None,
            });
        }
        0x6E => {
            // ld l, (hl)
            return Ok(Instruction {
                op: LD {
                    dst: R8_L,
                    src: PTR(Box::new(R16_HL)),
                },
                size: 1,
                cycles: 8,
                branch_cycles: None,
            });
        }
        0x6F => {
            // ld l, a
            return Ok(Instruction {
                op: LD {
                    dst: R8_L,
                    src: R8_A,
                },
                size: 1,
                cycles: 4,
                branch_cycles: None,
            });
        }
        0x70 => {
            // ld (hl), b
            return Ok(Instruction {
                op: LD {
                    dst: PTR(Box::new(R16_HL)),
                    src: R8_B,
                },
                size: 1,
                cycles: 8,
                branch_cycles: None,
            });
        }
        0x71 => {
            // ld (hl), c
            return Ok(Instruction {
                op: LD {
                    dst: PTR(Box::new(R16_HL)),
                    src: R8_C,
                },
                size: 1,
                cycles: 8,
                branch_cycles: None,
            });
        }
        0x72 => {
            // ld (hl), d
            return Ok(Instruction {
                op: LD {
                    dst: PTR(Box::new(R16_HL)),
                    src: R8_D,
                },
                size: 1,
                cycles: 8,
                branch_cycles: None,
            });
        }
        0x73 => {
            // ld (hl), e
            return Ok(Instruction {
                op: LD {
                    dst: PTR(Box::new(R16_HL)),
                    src: R8_E,
                },
                size: 1,
                cycles: 8,
                branch_cycles: None,
            });
        }
        0x74 => {
            // ld (hl), h
            return Ok(Instruction {
                op: LD {
                    dst: PTR(Box::new(R16_HL)),
                    src: R8_H,
                },
                size: 1,
                cycles: 8,
                branch_cycles: None,
            });
        }
        0x75 => {
            // ld (hl), l
            return Ok(Instruction {
                op: LD {
                    dst: PTR(Box::new(R16_HL)),
                    src: R8_L,
                },
                size: 1,
                cycles: 8,
                branch_cycles: None,
            });
        }
        0x77 => {
            // ld (hl), a
            return Ok(Instruction {
                op: LD {
                    dst: PTR(Box::new(R16_HL)),
                    src: R8_A,
                },
                size: 1,
                cycles: 8,
                branch_cycles: None,
            });
        }
        0x78 => {
            // ld a, b
            return Ok(Instruction {
                op: LD {
                    dst: R8_A,
                    src: R8_B,
                },
                size: 1,
                cycles: 4,
                branch_cycles: None,
            });
        }
        0x79 => {
            // ld a, c
            return Ok(Instruction {
                op: LD {
                    dst: R8_A,
                    src: R8_C,
                },
                size: 1,
                cycles: 4,
                branch_cycles: None,
            });
        }
        0x7A => {
            // ld a, d
            return Ok(Instruction {
                op: LD {
                    dst: R8_A,
                    src: R8_D,
                },
                size: 1,
                cycles: 4,
                branch_cycles: None,
            });
        }
        0x7B => {
            // ld a, e
            return Ok(Instruction {
                op: LD {
                    dst: R8_A,
                    src: R8_E,
                },
                size: 1,
                cycles: 4,
                branch_cycles: None,
            });
        }
        0x7C => {
            // ld a, h
            return Ok(Instruction {
                op: LD {
                    dst: R8_A,
                    src: R8_H,
                },
                size: 1,
                cycles: 4,
                branch_cycles: None,
            });
        }
        0x7D => {
            // ld a, l
            return Ok(Instruction {
                op: LD {
                    dst: R8_A,
                    src: R8_L,
                },
                size: 1,
                cycles: 4,
                branch_cycles: None,
            });
        }
        0x7E => {
            // ld a, (hl)
            return Ok(Instruction {
                op: LD {
                    dst: R8_A,
                    src: PTR(Box::new(R16_HL)),
                },
                size: 1,
                cycles: 8,
                branch_cycles: None,
            });
        }
        0x7F => {
            // ld a, a
            return Ok(Instruction {
                op: LD {
                    dst: R8_A,
                    src: R8_A,
                },
                size: 1,
                cycles: 4,
                branch_cycles: None,
            });
        }
        0x80 => {
            // add a, b
            return Ok(Instruction {
                op: ADD { x: R8_A, y: R8_B },
                size: 1,
                cycles: 4,
                branch_cycles: None,
            });
        }
        0x81 => {
            // add a, c
            return Ok(Instruction {
                op: ADD { x: R8_A, y: R8_C },
                size: 1,
                cycles: 4,
                branch_cycles: None,
            });
        }
        0x82 => {
            // add a, d
            return Ok(Instruction {
                op: ADD { x: R8_A, y: R8_D },
                size: 1,
                cycles: 4,
                branch_cycles: None,
            });
        }
        0x83 => {
            // add a, e
            return Ok(Instruction {
                op: ADD { x: R8_A, y: R8_E },
                size: 1,
                cycles: 4,
                branch_cycles: None,
            });
        }
        0x84 => {
            // add a, h
            return Ok(Instruction {
                op: ADD { x: R8_A, y: R8_H },
                size: 1,
                cycles: 4,
                branch_cycles: None,
            });
        }
        0x85 => {
            // add a, l
            return Ok(Instruction {
                op: ADD { x: R8_A, y: R8_L },
                size: 1,
                cycles: 4,
                branch_cycles: None,
            });
        }
        0x86 => {
            // add a, (hl)
            return Ok(Instruction {
                op: ADD {
                    x: R8_A,
                    y: PTR(Box::new(R16_HL)),
                },
                size: 1,
                cycles: 8,
                branch_cycles: None,
            });
        }
        0x87 => {
            // add a, a
            return Ok(Instruction {
                op: ADD { x: R8_A, y: R8_A },
                size: 1,
                cycles: 4,
                branch_cycles: None,
            });
        }
        0x90 => {
            // sub a, b
            return Ok(Instruction {
                op: SUB { y: R8_B },
                size: 1,
                cycles: 4,
                branch_cycles: None,
            });
        }
        0x91 => {
            // sub a, c
            return Ok(Instruction {
                op: SUB { y: R8_C },
                size: 1,
                cycles: 4,
                branch_cycles: None,
            });
        }
        0x92 => {
            // sub a, d
            return Ok(Instruction {
                op: SUB { y: R8_D },
                size: 1,
                cycles: 4,
                branch_cycles: None,
            });
        }
        0x93 => {
            // sub a, e
            return Ok(Instruction {
                op: SUB { y: R8_E },
                size: 1,
                cycles: 4,
                branch_cycles: None,
            });
        }
        0x94 => {
            // sub a, h
            return Ok(Instruction {
                op: SUB { y: R8_H },
                size: 1,
                cycles: 4,
                branch_cycles: None,
            });
        }
        0x95 => {
            // sub a, l
            return Ok(Instruction {
                op: SUB { y: R8_L },
                size: 1,
                cycles: 4,
                branch_cycles: None,
            });
        }
        0x96 => {
            // sub a, (hl)
            return Ok(Instruction {
                op: SUB {
                    y: PTR(Box::new(R16_HL)),
                },
                size: 1,
                cycles: 8,
                branch_cycles: None,
            });
        }
        0x97 => {
            // sub a, a
            return Ok(Instruction {
                op: SUB { y: R8_A },
                size: 1,
                cycles: 4,
                branch_cycles: None,
            });
        }
        0xA0 => {
            // and a, b
            return Ok(Instruction {
                op: AND { y: R8_B },
                size: 1,
                cycles: 4,
                branch_cycles: None,
            });
        }
        0xA1 => {
            // and a, c
            return Ok(Instruction {
                op: AND { y: R8_C },
                size: 1,
                cycles: 4,
                branch_cycles: None,
            });
        }
        0xA2 => {
            // and a, d
            return Ok(Instruction {
                op: AND { y: R8_D },
                size: 1,
                cycles: 4,
                branch_cycles: None,
            });
        }
        0xA3 => {
            // and a, e
            return Ok(Instruction {
                op: AND { y: R8_E },
                size: 1,
                cycles: 4,
                branch_cycles: None,
            });
        }
        0xA4 => {
            // and a, h
            return Ok(Instruction {
                op: AND { y: R8_H },
                size: 1,
                cycles: 4,
                branch_cycles: None,
            });
        }
        0xA5 => {
            // and a, l
            return Ok(Instruction {
                op: AND { y: R8_L },
                size: 1,
                cycles: 4,
                branch_cycles: None,
            });
        }
        0xA6 => {
            // and a, (hl)
            return Ok(Instruction {
                op: AND {
                    y: PTR(Box::new(R16_HL)),
                },
                size: 1,
                cycles: 8,
                branch_cycles: None,
            });
        }
        0xA7 => {
            // and a, a
            return Ok(Instruction {
                op: AND { y: R8_A },
                size: 1,
                cycles: 4,
                branch_cycles: None,
            });
        }
        0xA8 => {
            // xor a, b
            return Ok(Instruction {
                op: XOR { y: R8_B },
                size: 1,
                cycles: 4,
                branch_cycles: None,
            });
        }
        0xA9 => {
            // xor a, c
            return Ok(Instruction {
                op: XOR { y: R8_C },
                size: 1,
                cycles: 4,
                branch_cycles: None,
            });
        }
        0xAA => {
            // xor a, d
            return Ok(Instruction {
                op: XOR { y: R8_D },
                size: 1,
                cycles: 4,
                branch_cycles: None,
            });
        }
        0xAB => {
            // xor a, e
            return Ok(Instruction {
                op: XOR { y: R8_E },
                size: 1,
                cycles: 4,
                branch_cycles: None,
            });
        }
        0xAC => {
            // xor a, h
            return Ok(Instruction {
                op: XOR { y: R8_H },
                size: 1,
                cycles: 4,
                branch_cycles: None,
            });
        }
        0xAD => {
            // xor a, l
            return Ok(Instruction {
                op: XOR { y: R8_L },
                size: 1,
                cycles: 4,
                branch_cycles: None,
            });
        }
        0xAE => {
            // xor a, (hl)
            return Ok(Instruction {
                op: XOR {
                    y: PTR(Box::new(R16_HL)),
                },
                size: 1,
                cycles: 4,
                branch_cycles: None,
            });
        }
        0xAF => {
            // xor a, a
            return Ok(Instruction {
                op: XOR { y: R8_A },
                size: 1,
                cycles: 4,
                branch_cycles: None,
            });
        }
        0xB0 => {
            // or a, b
            return Ok(Instruction {
                op: OR { y: R8_B },
                size: 1,
                cycles: 4,
                branch_cycles: None,
            });
        }
        0xB1 => {
            // or a, c
            return Ok(Instruction {
                op: OR { y: R8_C },
                size: 1,
                cycles: 4,
                branch_cycles: None,
            });
        }
        0xB2 => {
            // or a, d
            return Ok(Instruction {
                op: OR { y: R8_D },
                size: 1,
                cycles: 4,
                branch_cycles: None,
            });
        }
        0xB3 => {
            // or a, e
            return Ok(Instruction {
                op: OR { y: R8_E },
                size: 1,
                cycles: 4,
                branch_cycles: None,
            });
        }
        0xB4 => {
            // or a, h
            return Ok(Instruction {
                op: OR { y: R8_H },
                size: 1,
                cycles: 4,
                branch_cycles: None,
            });
        }
        0xB5 => {
            // or a, l
            return Ok(Instruction {
                op: OR { y: R8_L },
                size: 1,
                cycles: 4,
                branch_cycles: None,
            });
        }
        0xB6 => {
            // or a, (hl)
            return Ok(Instruction {
                op: OR {
                    y: PTR(Box::new(R16_HL)),
                },
                size: 1,
                cycles: 8,
                branch_cycles: None,
            });
        }
        0xB7 => {
            // or a, a
            return Ok(Instruction {
                op: OR { y: R8_A },
                size: 1,
                cycles: 4,
                branch_cycles: None,
            });
        }
        0xBE => {
            // cp a, (hl)
            return Ok(Instruction {
                op: CP {
                    y: PTR(Box::new(R16_HL)),
                },
                size: 1,
                cycles: 8,
                branch_cycles: None,
            });
        }
        0xC0 => {
            // ret nz
            return Ok(Instruction {
                op: RET_CC { cc: CC_NZ },
                size: 1,
                cycles: 8,
                branch_cycles: Some(20),
            });
        }
        0xC1 => {
            // pop bc
            return Ok(Instruction {
                op: POP { reg: R16_BC },
                size: 1,
                cycles: 12,
                branch_cycles: None,
            });
        }
        0xC3 => {
            // jp imm16
            return Ok(Instruction {
                op: JP { addr: IMM16(imm16) },
                size: 3,
                cycles: 16,
                branch_cycles: None,
            });
        }
        0xC4 => {
            // call nz
            return Ok(Instruction {
                op: CALL_CC {
                    cc: CC_NZ,
                    proc: IMM16(imm16),
                },
                size: 3,
                cycles: 12,
                branch_cycles: Some(24),
            });
        }
        0xC5 => {
            // push bc
            return Ok(Instruction {
                op: PUSH { reg: R16_BC },
                size: 1,
                cycles: 16,
                branch_cycles: None,
            });
        }
        0xC6 => {
            // add a, imm8
            return Ok(Instruction {
                op: ADD {
                    x: R8_A,
                    y: IMM8(imm8),
                },
                size: 2,
                cycles: 8,
                branch_cycles: None,
            });
        }
        0xC7 => {
            // rst 00
            return Ok(Instruction {
                op: RST {
                    addr: Operand::IMM16(0x00),
                },
                size: 1,
                cycles: 16,
                branch_cycles: None,
            });
        }
        0xC8 => {
            // ret z
            return Ok(Instruction {
                op: RET_CC { cc: CC_Z },
                size: 1,
                cycles: 8,
                branch_cycles: Some(20),
            });
        }
        0xC9 => {
            // ret
            return Ok(Instruction {
                op: RET,
                size: 1,
                cycles: 16,
                branch_cycles: None,
            });
        }
        0xCB => {
            //prefixed bit manipulation instructions
            match imm8 {
                0x11 => {
                    // rl c
                    return Ok(Instruction {
                        op: RL { x: R8_C },
                        size: 2,
                        cycles: 8,
                        branch_cycles: None,
                    });
                }
                0x30 => {
                    // swap b
                    return Ok(Instruction {
                        op: SWAP { x: R8_B },
                        size: 2,
                        cycles: 8,
                        branch_cycles: None,
                    });
                }
                0x31 => {
                    // swap c
                    return Ok(Instruction {
                        op: SWAP { x: R8_C },
                        size: 2,
                        cycles: 8,
                        branch_cycles: None,
                    });
                }
                0x32 => {
                    // swap d
                    return Ok(Instruction {
                        op: SWAP { x: R8_D },
                        size: 2,
                        cycles: 8,
                        branch_cycles: None,
                    });
                }
                0x33 => {
                    // swap e
                    return Ok(Instruction {
                        op: SWAP { x: R8_E },
                        size: 2,
                        cycles: 8,
                        branch_cycles: None,
                    });
                }
                0x34 => {
                    // swap h
                    return Ok(Instruction {
                        op: SWAP { x: R8_H },
                        size: 2,
                        cycles: 8,
                        branch_cycles: None,
                    });
                }
                0x35 => {
                    // swap l
                    return Ok(Instruction {
                        op: SWAP { x: R8_L },
                        size: 2,
                        cycles: 8,
                        branch_cycles: None,
                    });
                }
                0x36 => {
                    // swap (hl)
                    return Ok(Instruction {
                        op: SWAP {
                            x: PTR(Box::new(R16_HL)),
                        },
                        size: 2,
                        cycles: 16,
                        branch_cycles: None,
                    });
                }
                0x37 => {
                    // swap a
                    return Ok(Instruction {
                        op: SWAP { x: R8_A },
                        size: 2,
                        cycles: 8,
                        branch_cycles: None,
                    });
                }
                0x7C => {
                    // bit 7, h
                    return Ok(Instruction {
                        op: BIT { bit: 7, src: R8_H },
                        size: 2,
                        cycles: 8,
                        branch_cycles: None,
                    });
                }
                _ => {
                    return Err(EmulationError {
                        ty: EmulationErrorType::UnhandledInstructionDecode(0xCB00 + imm8 as u16),
                        pc: Some(address),
                    });
                }
            }
        }
        0xCC => {
            // call z, imm16
            return Ok(Instruction {
                op: CALL_CC {
                    cc: CC_Z,
                    proc: IMM16(imm16),
                },
                size: 3,
                cycles: 12,
                branch_cycles: Some(24),
            });
        }
        0xCD => {
            // call imm16
            return Ok(Instruction {
                op: CALL { proc: IMM16(imm16) },
                size: 3,
                cycles: 24,
                branch_cycles: None,
            });
        }
        0xCF => {
            // rst 08
            return Ok(Instruction {
                op: RST {
                    addr: Operand::IMM16(0x08),
                },
                size: 1,
                cycles: 16,
                branch_cycles: None,
            });
        }
        0xD0 => {
            // ret nc
            return Ok(Instruction {
                op: RET_CC { cc: CC_NC },
                size: 1,
                cycles: 8,
                branch_cycles: Some(20),
            });
        }
        0xD1 => {
            // pop de
            return Ok(Instruction {
                op: POP { reg: R16_DE },
                size: 1,
                cycles: 12,
                branch_cycles: None,
            });
        }
        0xD4 => {
            // call nc, imm16
            return Ok(Instruction {
                op: CALL_CC {
                    cc: CC_NC,
                    proc: IMM16(imm16),
                },
                size: 3,
                cycles: 12,
                branch_cycles: Some(24),
            });
        }
        0xD5 => {
            // push de
            return Ok(Instruction {
                op: PUSH { reg: R16_DE },
                size: 1,
                cycles: 16,
                branch_cycles: None,
            });
        }
        0xD6 => {
            // sub a, imm8
            return Ok(Instruction {
                op: SUB { y: IMM8(imm8) },
                size: 2,
                cycles: 8,
                branch_cycles: None,
            });
        }
        0xD7 => {
            // rst 10
            return Ok(Instruction {
                op: RST {
                    addr: Operand::IMM16(0x10),
                },
                size: 1,
                cycles: 16,
                branch_cycles: None,
            });
        }
        0xD8 => {
            // ret c
            return Ok(Instruction {
                op: RET_CC { cc: CC_C },
                size: 1,
                cycles: 8,
                branch_cycles: Some(20),
            });
        }
        0xDC => {
            // call c, imm16
            return Ok(Instruction {
                op: CALL_CC {
                    cc: CC_C,
                    proc: IMM16(imm16),
                },
                size: 3,
                cycles: 12,
                branch_cycles: Some(24),
            });
        }
        0xDF => {
            // rst 18
            return Ok(Instruction {
                op: RST {
                    addr: Operand::IMM16(0x18),
                },
                size: 1,
                cycles: 16,
                branch_cycles: None,
            });
        }
        0xE0 => {
            // ld (imm8), a
            return Ok(Instruction {
                op: LD {
                    dst: PTR(Box::new(IMM8(imm8))),
                    src: R8_A,
                },
                size: 2,
                cycles: 12,
                branch_cycles: None,
            });
        }
        0xE1 => {
            // pop hl
            return Ok(Instruction {
                op: POP { reg: R16_HL },
                size: 1,
                cycles: 12,
                branch_cycles: None,
            });
        }
        0xE2 => {
            // ld (c), a
            return Ok(Instruction {
                op: LD {
                    dst: PTR(Box::new(R8_C)),
                    src: R8_A,
                },
                size: 1,
                cycles: 8,
                branch_cycles: None,
            });
        }
        0xE5 => {
            // push hl
            return Ok(Instruction {
                op: PUSH { reg: R16_HL },
                size: 1,
                cycles: 16,
                branch_cycles: None,
            });
        }
        0xE6 => {
            // and a, imm8
            return Ok(Instruction {
                op: AND { y: IMM8(imm8) },
                size: 2,
                cycles: 8,
                branch_cycles: None,
            });
        }
        0xE7 => {
            // rst 20
            return Ok(Instruction {
                op: RST {
                    addr: Operand::IMM16(0x20),
                },
                size: 1,
                cycles: 16,
                branch_cycles: None,
            });
        }
        0xEA => {
            // ld (imm16), a
            return Ok(Instruction {
                op: LD {
                    dst: PTR(Box::new(IMM16(imm16))),
                    src: R8_A,
                },
                size: 3,
                cycles: 16,
                branch_cycles: None,
            });
        }
        0xEF => {
            // rst 28
            return Ok(Instruction {
                op: RST {
                    addr: Operand::IMM16(0x28),
                },
                size: 1,
                cycles: 16,
                branch_cycles: None,
            });
        }
        0xF0 => {
            // ld a, (imm8)
            return Ok(Instruction {
                op: LD {
                    dst: R8_A,
                    src: PTR(Box::new(IMM8(imm8))),
                },
                size: 2,
                cycles: 12,
                branch_cycles: None,
            });
        }
        0xF1 => {
            // pop af
            return Ok(Instruction {
                op: POP { reg: R16_AF },
                size: 1,
                cycles: 12,
                branch_cycles: None,
            });
        }
        0xF2 => {
            // ld a, (c)
            return Ok(Instruction {
                op: LD {
                    dst: R8_A,
                    src: PTR(Box::new(R8_C)),
                },
                size: 1,
                cycles: 8,
                branch_cycles: None,
            });
        }
        0xF3 => {
            // di
            return Ok(Instruction {
                op: DI,
                size: 1,
                cycles: 4,
                branch_cycles: None,
            });
        }
        0xF5 => {
            // push af
            return Ok(Instruction {
                op: PUSH { reg: R16_AF },
                size: 1,
                cycles: 16,
                branch_cycles: None,
            });
        }
        0xF6 => {
            // or a, imm8
            return Ok(Instruction {
                op: OR { y: IMM8(imm8) },
                size: 2,
                cycles: 8,
                branch_cycles: None,
            });
        }
        0xF7 => {
            // rst 30
            return Ok(Instruction {
                op: RST {
                    addr: Operand::IMM16(0x30),
                },
                size: 1,
                cycles: 16,
                branch_cycles: None,
            });
        }
        0xFA => {
            // ld a, (imm16)
            return Ok(Instruction {
                op: LD {
                    dst: R8_A,
                    src: PTR(Box::new(IMM16(imm16))),
                },
                size: 3,
                cycles: 16,
                branch_cycles: None,
            });
        }
        0xFB => {
            // ei
            return Ok(Instruction {
                op: EI,
                size: 1,
                cycles: 4,
                branch_cycles: None,
            });
        }
        0xFE => {
            // cp imm8
            return Ok(Instruction {
                op: CP { y: IMM8(imm8) },
                size: 2,
                cycles: 8,
                branch_cycles: None,
            });
        }
        0xFF => {
            // rst 38
            return Ok(Instruction {
                op: RST {
                    addr: Operand::IMM16(0x38),
                },
                size: 1,
                cycles: 16,
                branch_cycles: None,
            });
        }
        _ => {
            return Err(EmulationError {
                ty: EmulationErrorType::UnhandledInstructionDecode(instr as u16),
                pc: Some(address),
            });
        }
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
        Operand::CC_NC => String::from("nc"),
        Operand::CC_C => String::from("c"),
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
        Operation::CALL_CC { cc, proc } => format!("call {cc}, {proc}"),
        Operation::RST { addr } => format!("rst {addr}"),
        Operation::RET => String::from("ret"),
        Operation::RET_CC { cc } => format!("ret {cc}"),
        Operation::PUSH { reg: word } => format!("push {word}"),
        Operation::POP { reg: word } => format!("pop {word}"),
        Operation::DEC { x } => format!("dec {x}"),
        Operation::INC { x } => format!("inc {x}"),
        Operation::ADD { x, y } => format!("add {x}, {y}"),
        Operation::SUB { y } => format!("sub a, {y}"),
        Operation::OR { y } => format!("or a, {y}"),
        Operation::XOR { y } => format!("xor a, {y}"),
        Operation::AND { y } => format!("and a, {y}"),
        Operation::BIT { bit, src: r8 } => format!("bit {bit}, {r8}"),
        Operation::SWAP { x } => format!("swap {x}"),
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
        Operation::EI => String::from("ei"),
        Operation::CPL => String::from("cpl"),
    }
}

impl fmt::Display for Instruction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", instruction_to_string(self))
    }
}
