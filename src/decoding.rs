use core::fmt;
use std::fmt::format;

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
    SP_PLUS_SIGNED_IMM8(i8),
    PTR(Box<Operand>),
}

#[allow(non_camel_case_types)]
#[derive(Debug)]
pub enum Operation {
    NOP,
    LD { dst: Operand, src: Operand },
    JP { addr: Operand },
    JP_CC { cc: Operand, addr: Operand },
    JR { offset_oprd: Operand },
    JR_CC { cc: Operand, offset_oprd: Operand },
    CALL { proc: Operand },
    CALL_CC { cc: Operand, proc: Operand },
    RST { addr: Operand },
    RET,
    RET_CC { cc: Operand },
    RETI,
    PUSH { reg: Operand },
    POP { reg: Operand },
    DEC { x: Operand },
    INC { x: Operand },
    ADD { x: Operand, y: Operand },
    SUB { y: Operand },
    ADC { y: Operand },
    OR { y: Operand },
    XOR { y: Operand },
    AND { y: Operand },
    BIT { bit: u8, src: Operand },
    RES { bit: u8, x: Operand },
    SWAP { x: Operand },
    RL { x: Operand },
    RR { x: Operand },
    RLC { x: Operand },
    RRC { x: Operand },
    RLA,
    RRA,
    RLCA,
    RRCA,
    SRL { x: Operand },
    SLA { x: Operand },
    CP { y: Operand },
    DI,
    EI,
    CPL,
    HALT,
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
        // nop
        0x00 => {
            return Ok(Instruction {
                op: NOP,
                size: 1,
                cycles: 4,
                branch_cycles: None,
            });
        }
        // ld bc, imm16
        0x01 => {
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
        // ld (bc), a
        0x02 => {
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
        // inc bc
        0x03 => {
            return Ok(Instruction {
                op: INC { x: R16_BC },
                size: 1,
                cycles: 8,
                branch_cycles: None,
            });
        }
        // inc b
        0x04 => {
            return Ok(Instruction {
                op: INC { x: R8_B },
                size: 1,
                cycles: 4,
                branch_cycles: None,
            });
        }
        // dec b
        0x05 => {
            return Ok(Instruction {
                op: DEC { x: R8_B },
                size: 1,
                cycles: 4,
                branch_cycles: None,
            });
        }
        // ld b, imm8
        0x06 => {
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
        // add hl, bc
        0x09 => {
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
        // ld a, (bc)
        0x0A => {
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
        // dec bc
        0x0B => {
            return Ok(Instruction {
                op: DEC { x: R16_BC },
                size: 1,
                cycles: 8,
                branch_cycles: None,
            });
        }
        // inc c
        0x0C => {
            return Ok(Instruction {
                op: INC { x: R8_C },
                size: 1,
                cycles: 4,
                branch_cycles: None,
            });
        }
        // dec c
        0x0D => {
            return Ok(Instruction {
                op: DEC { x: R8_C },
                size: 1,
                cycles: 4,
                branch_cycles: None,
            });
        }
        // ld c, imm8
        0x0E => {
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
        // rrca
        0x0F => {
            return Ok(Instruction {
                op: RRCA,
                size: 1,
                cycles: 4,
                branch_cycles: None,
            });
        }
        // ld de, imm16
        0x11 => {
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
        // ld (de), a
        0x12 => {
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
        // inc de
        0x13 => {
            return Ok(Instruction {
                op: INC { x: R16_DE },
                size: 1,
                cycles: 8,
                branch_cycles: None,
            });
        }
        // inc d
        0x14 => {
            return Ok(Instruction {
                op: INC { x: R8_D },
                size: 1,
                cycles: 4,
                branch_cycles: None,
            });
        }
        // dec d
        0x15 => {
            return Ok(Instruction {
                op: DEC { x: R8_D },
                size: 1,
                cycles: 4,
                branch_cycles: None,
            });
        }
        // ld d, imm8
        0x16 => {
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
        // rla
        0x17 => {
            return Ok(Instruction {
                op: RLA,
                size: 1,
                cycles: 8,
                branch_cycles: None,
            });
        }
        // jr imm8
        0x18 => {
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
        // add hl, de
        0x19 => {
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
        // ld a, (de)
        0x1A => {
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
        // dec de
        0x1B => {
            return Ok(Instruction {
                op: DEC { x: R16_DE },
                size: 1,
                cycles: 8,
                branch_cycles: None,
            });
        }
        // inc e
        0x1C => {
            return Ok(Instruction {
                op: INC { x: R8_E },
                size: 1,
                cycles: 4,
                branch_cycles: None,
            });
        }
        // dec e
        0x1D => {
            return Ok(Instruction {
                op: DEC { x: R8_E },
                size: 1,
                cycles: 8,
                branch_cycles: None,
            });
        }
        // ld e, imm8
        0x1E => {
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
        // RRA
        0x1F => {
            return Ok(Instruction {
                op: RRA,
                size: 1,
                cycles: 4,
                branch_cycles: None,
            });
        }
        // jr nz, imm8
        0x20 => {
            return Ok(Instruction {
                op: JR_CC {
                    cc: CC_NZ,
                    // the relative jump offset is signed
                    offset_oprd: IMM8_SIGNED(i8::from_le_bytes([imm8])),
                },
                size: 2,
                cycles: 8,
                branch_cycles: Some(12),
            });
        }
        // ld hl, imm16
        0x21 => {
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
        // ld (hl+), a
        0x22 => {
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
        // inc hl
        0x23 => {
            return Ok(Instruction {
                op: INC { x: R16_HL },
                size: 1,
                cycles: 8,
                branch_cycles: None,
            });
        }
        // inc h
        0x24 => {
            return Ok(Instruction {
                op: INC { x: R8_H },
                size: 1,
                cycles: 4,
                branch_cycles: None,
            });
        }
        // dec h
        0x25 => {
            return Ok(Instruction {
                op: DEC { x: R8_H },
                size: 1,
                cycles: 4,
                branch_cycles: None,
            });
        }
        // ld h, imm8
        0x26 => {
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
        // jr z, imm8
        0x28 => {
            return Ok(Instruction {
                op: JR_CC {
                    cc: CC_Z,
                    offset_oprd: IMM8_SIGNED(i8::from_le_bytes([imm8])),
                },
                size: 2,
                cycles: 8,
                branch_cycles: Some(12),
            });
        }
        // add hl, hl
        0x29 => {
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
        // ld a, (hl+)
        0x2A => {
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
        // dec hl
        0x2B => {
            return Ok(Instruction {
                op: DEC { x: R16_HL },
                size: 1,
                cycles: 8,
                branch_cycles: None,
            });
        }
        // inc l
        0x2C => {
            return Ok(Instruction {
                op: INC { x: R8_L },
                size: 1,
                cycles: 4,
                branch_cycles: None,
            });
        }
        // dec l
        0x2D => {
            return Ok(Instruction {
                op: DEC { x: R8_L },
                size: 1,
                cycles: 4,
                branch_cycles: None,
            });
        }
        // ld l, imm8
        0x2E => {
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
        // cpl
        0x2F => {
            return Ok(Instruction {
                op: CPL,
                size: 1,
                cycles: 4,
                branch_cycles: None,
            });
        }
        // jr nc, imm8
        0x30 => {
            return Ok(Instruction {
                op: JR_CC {
                    cc: CC_NC,
                    offset_oprd: IMM8_SIGNED(i8::from_le_bytes([imm8])),
                },
                size: 2,
                cycles: 8,
                branch_cycles: Some(12),
            });
        }
        // ld sp, imm16
        0x31 => {
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
        // ld (hl-), a
        0x32 => {
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
        // inc sp
        0x33 => {
            return Ok(Instruction {
                op: INC { x: R16_SP },
                size: 1,
                cycles: 8,
                branch_cycles: None,
            });
        }
        // inc (hl)
        0x34 => {
            return Ok(Instruction {
                op: INC {
                    x: PTR(Box::new(R16_HL)),
                },
                size: 1,
                cycles: 12,
                branch_cycles: None,
            });
        }
        // dec (hl)
        0x35 => {
            return Ok(Instruction {
                op: DEC {
                    x: PTR(Box::new(R16_HL)),
                },
                size: 1,
                cycles: 12,
                branch_cycles: None,
            });
        }
        // ld (hl), imm8
        0x36 => {
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
        // jr c, imm8
        0x38 => {
            return Ok(Instruction {
                op: JR_CC {
                    cc: CC_C,
                    offset_oprd: IMM8_SIGNED(i8::from_le_bytes([imm8])),
                },
                size: 2,
                cycles: 8,
                branch_cycles: Some(12),
            });
        }
        // add hl, sp
        0x39 => {
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
        // ld a, (hl-)
        0x3A => {
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
        // dec sp
        0x3B => {
            return Ok(Instruction {
                op: DEC { x: R16_SP },
                size: 1,
                cycles: 8,
                branch_cycles: None,
            });
        }
        // inc a
        0x3C => {
            return Ok(Instruction {
                op: INC { x: R8_A },
                size: 1,
                cycles: 4,
                branch_cycles: None,
            });
        }
        // dec a
        0x3D => {
            return Ok(Instruction {
                op: DEC { x: R8_A },
                size: 1,
                cycles: 4,
                branch_cycles: None,
            });
        }
        // ld a, imm8
        0x3E => {
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
        // ld b, b
        0x40 => {
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
        // ld b, c
        0x41 => {
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
        // ld b, d
        0x42 => {
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
        // ld b, e
        0x43 => {
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
        // ld b, h
        0x44 => {
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
        // ld b, l
        0x45 => {
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
        // ld b, (hl)
        0x46 => {
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
        // ld b, a
        0x47 => {
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
        // ld c, b
        0x48 => {
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
        // ld c, c
        0x49 => {
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
        // ld c, d
        0x4A => {
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
        // ld c, e
        0x4B => {
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
        // ld c, h
        0x4C => {
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
        // ld c, l
        0x4D => {
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
        // ld c, (hl)
        0x4E => {
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
        // ld c, a
        0x4F => {
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
        // ld d, b
        0x50 => {
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
        // ld d, c
        0x51 => {
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
        // ld d, d
        0x52 => {
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
        // ld d, e
        0x53 => {
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
        // ld d, h
        0x54 => {
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
        // ld d, l
        0x55 => {
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
        // ld d, (hl)
        0x56 => {
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
        // ld d, a
        0x57 => {
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
        // ld e, b
        0x58 => {
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
        // ld e, c
        0x59 => {
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
        // ld e, d
        0x5A => {
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
        // ld e, e
        0x5B => {
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
        // ld e, h
        0x5C => {
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
        // ld e, l
        0x5D => {
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
        // ld e, (hl)
        0x5E => {
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
        // ld e, a
        0x5F => {
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
        // ld h, b
        0x60 => {
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
        // ld h, c
        0x61 => {
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
        // ld h, d
        0x62 => {
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
        // ld h, e
        0x63 => {
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
        // ld h, h
        0x64 => {
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
        // ld h, l
        0x65 => {
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
        // ld h, (hl)
        0x66 => {
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
        // ld h, a
        0x67 => {
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
        // ld l, b
        0x68 => {
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
        // ld l, c
        0x69 => {
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
        // ld l, d
        0x6A => {
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
        // ld l, e
        0x6B => {
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
        // ld l, h
        0x6C => {
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
        // ld l, l
        0x6D => {
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
        // ld l, (hl)
        0x6E => {
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
        // ld l, a
        0x6F => {
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
        // ld (hl), b
        0x70 => {
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
        // ld (hl), c
        0x71 => {
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
        // ld (hl), d
        0x72 => {
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
        // ld (hl), e
        0x73 => {
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
        // ld (hl), h
        0x74 => {
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
        // ld (hl), l
        0x75 => {
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
        // halt
        0x76 => {
            return Ok(Instruction {
                op: HALT,
                size: 1,
                cycles: 4,
                branch_cycles: None,
            });
        }
        // ld (hl), a
        0x77 => {
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
        // ld a, b
        0x78 => {
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
        // ld a, c
        0x79 => {
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
        // ld a, d
        0x7A => {
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
        // ld a, e
        0x7B => {
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
        // ld a, h
        0x7C => {
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
        // ld a, l
        0x7D => {
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
        // ld a, (hl)
        0x7E => {
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
        // ld a, a
        0x7F => {
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
        // add a, b
        0x80 => {
            return Ok(Instruction {
                op: ADD { x: R8_A, y: R8_B },
                size: 1,
                cycles: 4,
                branch_cycles: None,
            });
        }
        // add a, c
        0x81 => {
            return Ok(Instruction {
                op: ADD { x: R8_A, y: R8_C },
                size: 1,
                cycles: 4,
                branch_cycles: None,
            });
        }
        // add a, d
        0x82 => {
            return Ok(Instruction {
                op: ADD { x: R8_A, y: R8_D },
                size: 1,
                cycles: 4,
                branch_cycles: None,
            });
        }
        // add a, e
        0x83 => {
            return Ok(Instruction {
                op: ADD { x: R8_A, y: R8_E },
                size: 1,
                cycles: 4,
                branch_cycles: None,
            });
        }
        // add a, h
        0x84 => {
            return Ok(Instruction {
                op: ADD { x: R8_A, y: R8_H },
                size: 1,
                cycles: 4,
                branch_cycles: None,
            });
        }
        // add a, l
        0x85 => {
            return Ok(Instruction {
                op: ADD { x: R8_A, y: R8_L },
                size: 1,
                cycles: 4,
                branch_cycles: None,
            });
        }
        // add a, (hl)
        0x86 => {
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
        // add a, a
        0x87 => {
            return Ok(Instruction {
                op: ADD { x: R8_A, y: R8_A },
                size: 1,
                cycles: 4,
                branch_cycles: None,
            });
        }
        // adc a, b
        0x88 => {
            return Ok(Instruction {
                op: ADC { y: R8_B },
                size: 1,
                cycles: 4,
                branch_cycles: None,
            });
        }
        // adc a, c
        0x89 => {
            return Ok(Instruction {
                op: ADC { y: R8_C },
                size: 1,
                cycles: 4,
                branch_cycles: None,
            });
        }
        // adc a, d
        0x8A => {
            return Ok(Instruction {
                op: ADC { y: R8_D },
                size: 1,
                cycles: 4,
                branch_cycles: None,
            });
        }
        // adc a, e
        0x8B => {
            return Ok(Instruction {
                op: ADC { y: R8_E },
                size: 1,
                cycles: 4,
                branch_cycles: None,
            });
        }
        // adc a, h
        0x8C => {
            return Ok(Instruction {
                op: ADC { y: R8_H },
                size: 1,
                cycles: 4,
                branch_cycles: None,
            });
        }
        // adc a, l
        0x8D => {
            return Ok(Instruction {
                op: ADC { y: R8_L },
                size: 1,
                cycles: 4,
                branch_cycles: None,
            });
        }
        // adc a, (hl)
        0x8E => {
            return Ok(Instruction {
                op: ADC {
                    y: PTR(Box::new(R16_HL)),
                },
                size: 1,
                cycles: 8,
                branch_cycles: None,
            });
        }
        // adc a, a
        0x8F => {
            return Ok(Instruction {
                op: ADC { y: R8_A },
                size: 1,
                cycles: 4,
                branch_cycles: None,
            });
        }
        // sub a, b
        0x90 => {
            return Ok(Instruction {
                op: SUB { y: R8_B },
                size: 1,
                cycles: 4,
                branch_cycles: None,
            });
        }
        // sub a, c
        0x91 => {
            return Ok(Instruction {
                op: SUB { y: R8_C },
                size: 1,
                cycles: 4,
                branch_cycles: None,
            });
        }
        // sub a, d
        0x92 => {
            return Ok(Instruction {
                op: SUB { y: R8_D },
                size: 1,
                cycles: 4,
                branch_cycles: None,
            });
        }
        // sub a, e
        0x93 => {
            return Ok(Instruction {
                op: SUB { y: R8_E },
                size: 1,
                cycles: 4,
                branch_cycles: None,
            });
        }
        // sub a, h
        0x94 => {
            return Ok(Instruction {
                op: SUB { y: R8_H },
                size: 1,
                cycles: 4,
                branch_cycles: None,
            });
        }
        // sub a, l
        0x95 => {
            return Ok(Instruction {
                op: SUB { y: R8_L },
                size: 1,
                cycles: 4,
                branch_cycles: None,
            });
        }
        // sub a, (hl)
        0x96 => {
            return Ok(Instruction {
                op: SUB {
                    y: PTR(Box::new(R16_HL)),
                },
                size: 1,
                cycles: 8,
                branch_cycles: None,
            });
        }
        // sub a, a
        0x97 => {
            return Ok(Instruction {
                op: SUB { y: R8_A },
                size: 1,
                cycles: 4,
                branch_cycles: None,
            });
        }
        // and a, b
        0xA0 => {
            return Ok(Instruction {
                op: AND { y: R8_B },
                size: 1,
                cycles: 4,
                branch_cycles: None,
            });
        }
        // and a, c
        0xA1 => {
            return Ok(Instruction {
                op: AND { y: R8_C },
                size: 1,
                cycles: 4,
                branch_cycles: None,
            });
        }
        // and a, d
        0xA2 => {
            return Ok(Instruction {
                op: AND { y: R8_D },
                size: 1,
                cycles: 4,
                branch_cycles: None,
            });
        }
        // and a, e
        0xA3 => {
            return Ok(Instruction {
                op: AND { y: R8_E },
                size: 1,
                cycles: 4,
                branch_cycles: None,
            });
        }
        // and a, h
        0xA4 => {
            return Ok(Instruction {
                op: AND { y: R8_H },
                size: 1,
                cycles: 4,
                branch_cycles: None,
            });
        }
        // and a, l
        0xA5 => {
            return Ok(Instruction {
                op: AND { y: R8_L },
                size: 1,
                cycles: 4,
                branch_cycles: None,
            });
        }
        // and a, (hl)
        0xA6 => {
            return Ok(Instruction {
                op: AND {
                    y: PTR(Box::new(R16_HL)),
                },
                size: 1,
                cycles: 8,
                branch_cycles: None,
            });
        }
        // and a, a
        0xA7 => {
            return Ok(Instruction {
                op: AND { y: R8_A },
                size: 1,
                cycles: 4,
                branch_cycles: None,
            });
        }
        // xor a, b
        0xA8 => {
            return Ok(Instruction {
                op: XOR { y: R8_B },
                size: 1,
                cycles: 4,
                branch_cycles: None,
            });
        }
        // xor a, c
        0xA9 => {
            return Ok(Instruction {
                op: XOR { y: R8_C },
                size: 1,
                cycles: 4,
                branch_cycles: None,
            });
        }
        // xor a, d
        0xAA => {
            return Ok(Instruction {
                op: XOR { y: R8_D },
                size: 1,
                cycles: 4,
                branch_cycles: None,
            });
        }
        // xor a, e
        0xAB => {
            return Ok(Instruction {
                op: XOR { y: R8_E },
                size: 1,
                cycles: 4,
                branch_cycles: None,
            });
        }
        // xor a, h
        0xAC => {
            return Ok(Instruction {
                op: XOR { y: R8_H },
                size: 1,
                cycles: 4,
                branch_cycles: None,
            });
        }
        // xor a, l
        0xAD => {
            return Ok(Instruction {
                op: XOR { y: R8_L },
                size: 1,
                cycles: 4,
                branch_cycles: None,
            });
        }
        // xor a, (hl)
        0xAE => {
            return Ok(Instruction {
                op: XOR {
                    y: PTR(Box::new(R16_HL)),
                },
                size: 1,
                cycles: 4,
                branch_cycles: None,
            });
        }
        // xor a, a
        0xAF => {
            return Ok(Instruction {
                op: XOR { y: R8_A },
                size: 1,
                cycles: 4,
                branch_cycles: None,
            });
        }
        // or a, b
        0xB0 => {
            return Ok(Instruction {
                op: OR { y: R8_B },
                size: 1,
                cycles: 4,
                branch_cycles: None,
            });
        }
        // or a, c
        0xB1 => {
            return Ok(Instruction {
                op: OR { y: R8_C },
                size: 1,
                cycles: 4,
                branch_cycles: None,
            });
        }
        // or a, d
        0xB2 => {
            return Ok(Instruction {
                op: OR { y: R8_D },
                size: 1,
                cycles: 4,
                branch_cycles: None,
            });
        }
        // or a, e
        0xB3 => {
            return Ok(Instruction {
                op: OR { y: R8_E },
                size: 1,
                cycles: 4,
                branch_cycles: None,
            });
        }
        // or a, h
        0xB4 => {
            return Ok(Instruction {
                op: OR { y: R8_H },
                size: 1,
                cycles: 4,
                branch_cycles: None,
            });
        }
        // or a, l
        0xB5 => {
            return Ok(Instruction {
                op: OR { y: R8_L },
                size: 1,
                cycles: 4,
                branch_cycles: None,
            });
        }
        // or a, (hl)
        0xB6 => {
            return Ok(Instruction {
                op: OR {
                    y: PTR(Box::new(R16_HL)),
                },
                size: 1,
                cycles: 8,
                branch_cycles: None,
            });
        }
        // or a, a
        0xB7 => {
            return Ok(Instruction {
                op: OR { y: R8_A },
                size: 1,
                cycles: 4,
                branch_cycles: None,
            });
        }
        // cp a, b
        0xB8 => {
            return Ok(Instruction {
                op: CP { y: R8_B },
                size: 1,
                cycles: 4,
                branch_cycles: None,
            });
        }
        // cp a, c
        0xB9 => {
            return Ok(Instruction {
                op: CP { y: R8_C },
                size: 1,
                cycles: 4,
                branch_cycles: None,
            });
        }
        // cp a, d
        0xBA => {
            return Ok(Instruction {
                op: CP { y: R8_D },
                size: 1,
                cycles: 4,
                branch_cycles: None,
            });
        }
        // cp a, e
        0xBB => {
            return Ok(Instruction {
                op: CP { y: R8_E },
                size: 1,
                cycles: 4,
                branch_cycles: None,
            });
        }
        // cp a, h
        0xBC => {
            return Ok(Instruction {
                op: CP { y: R8_H },
                size: 1,
                cycles: 4,
                branch_cycles: None,
            });
        }
        // cp a, l
        0xBD => {
            return Ok(Instruction {
                op: CP { y: R8_L },
                size: 1,
                cycles: 4,
                branch_cycles: None,
            });
        }
        // cp a, (hl)
        0xBE => {
            return Ok(Instruction {
                op: CP {
                    y: PTR(Box::new(R16_HL)),
                },
                size: 1,
                cycles: 8,
                branch_cycles: None,
            });
        }
        // cp a, a
        0xBF => {
            return Ok(Instruction {
                op: CP { y: R8_A },
                size: 1,
                cycles: 4,
                branch_cycles: None,
            });
        }
        // ret nz
        0xC0 => {
            return Ok(Instruction {
                op: RET_CC { cc: CC_NZ },
                size: 1,
                cycles: 8,
                branch_cycles: Some(20),
            });
        }
        // pop bc
        0xC1 => {
            return Ok(Instruction {
                op: POP { reg: R16_BC },
                size: 1,
                cycles: 12,
                branch_cycles: None,
            });
        }
        // jp nz, imm16
        0xC2 => {
            return Ok(Instruction {
                op: JP_CC {
                    cc: CC_NZ,
                    addr: IMM16(imm16),
                },
                size: 3,
                cycles: 12,
                branch_cycles: Some(16),
            });
        }
        // jp imm16
        0xC3 => {
            return Ok(Instruction {
                op: JP { addr: IMM16(imm16) },
                size: 3,
                cycles: 16,
                branch_cycles: None,
            });
        }
        // call nz
        0xC4 => {
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
        // push bc
        0xC5 => {
            return Ok(Instruction {
                op: PUSH { reg: R16_BC },
                size: 1,
                cycles: 16,
                branch_cycles: None,
            });
        }
        // add a, imm8
        0xC6 => {
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
        // rst 00
        0xC7 => {
            return Ok(Instruction {
                op: RST {
                    addr: Operand::IMM16(0x00),
                },
                size: 1,
                cycles: 16,
                branch_cycles: None,
            });
        }
        // ret z
        0xC8 => {
            return Ok(Instruction {
                op: RET_CC { cc: CC_Z },
                size: 1,
                cycles: 8,
                branch_cycles: Some(20),
            });
        }
        // ret
        0xC9 => {
            return Ok(Instruction {
                op: RET,
                size: 1,
                cycles: 16,
                branch_cycles: None,
            });
        }
        // jp z, imm16
        0xCA => {
            return Ok(Instruction {
                op: JP_CC {
                    cc: CC_Z,
                    addr: IMM16(imm16),
                },
                size: 3,
                cycles: 12,
                branch_cycles: Some(16),
            });
        }
        // prefixed
        0xCB => {
            //prefixed bit manipulation instructions
            match imm8 {
                // rl c
                0x11 => {
                    return Ok(Instruction {
                        op: RL { x: R8_C },
                        size: 2,
                        cycles: 8,
                        branch_cycles: None,
                    });
                }
                // rr b
                0x18 => {
                    return Ok(Instruction {
                        op: RR { x: R8_B },
                        size: 2,
                        cycles: 8,
                        branch_cycles: None,
                    });
                }
                // rr c
                0x19 => {
                    return Ok(Instruction {
                        op: RR { x: R8_C },
                        size: 2,
                        cycles: 8,
                        branch_cycles: None,
                    });
                }
                // sla b
                0x20 => {
                    return Ok(Instruction {
                        op: SLA { x: R8_B },
                        size: 2,
                        cycles: 8,
                        branch_cycles: None,
                    });
                }
                // sla c
                0x21 => {
                    return Ok(Instruction {
                        op: SLA { x: R8_C },
                        size: 2,
                        cycles: 8,
                        branch_cycles: None,
                    });
                }
                // sla d
                0x22 => {
                    return Ok(Instruction {
                        op: SLA { x: R8_D },
                        size: 2,
                        cycles: 8,
                        branch_cycles: None,
                    });
                }
                // sla e
                0x23 => {
                    return Ok(Instruction {
                        op: SLA { x: R8_E },
                        size: 2,
                        cycles: 8,
                        branch_cycles: None,
                    });
                }
                // sla h
                0x24 => {
                    return Ok(Instruction {
                        op: SLA { x: R8_H },
                        size: 2,
                        cycles: 8,
                        branch_cycles: None,
                    });
                }
                // sla l
                0x25 => {
                    return Ok(Instruction {
                        op: SLA { x: R8_L },
                        size: 2,
                        cycles: 8,
                        branch_cycles: None,
                    });
                }
                // sla (hl)
                0x26 => {
                    return Ok(Instruction {
                        op: SLA {
                            x: PTR(Box::new(R16_HL)),
                        },
                        size: 2,
                        cycles: 16,
                        branch_cycles: None,
                    });
                }
                // sla a
                0x27 => {
                    return Ok(Instruction {
                        op: SLA { x: R8_A },
                        size: 2,
                        cycles: 8,
                        branch_cycles: None,
                    });
                }
                // rr d
                0x1A => {
                    return Ok(Instruction {
                        op: RR { x: R8_D },
                        size: 2,
                        cycles: 8,
                        branch_cycles: None,
                    });
                }
                // rr e
                0x1B => {
                    return Ok(Instruction {
                        op: RR { x: R8_E },
                        size: 2,
                        cycles: 8,
                        branch_cycles: None,
                    });
                }
                // rr h
                0x1C => {
                    return Ok(Instruction {
                        op: RR { x: R8_H },
                        size: 2,
                        cycles: 8,
                        branch_cycles: None,
                    });
                }
                // rr l
                0x1D => {
                    return Ok(Instruction {
                        op: RR { x: R8_L },
                        size: 2,
                        cycles: 8,
                        branch_cycles: None,
                    });
                }
                // rr (hl)
                0x1E => {
                    return Ok(Instruction {
                        op: RR {
                            x: PTR(Box::new(R16_HL)),
                        },
                        size: 2,
                        cycles: 16,
                        branch_cycles: None,
                    });
                }
                // rr a
                0x1F => {
                    return Ok(Instruction {
                        op: RR { x: R8_A },
                        size: 2,
                        cycles: 8,
                        branch_cycles: None,
                    });
                }

                // swap b
                0x30 => {
                    return Ok(Instruction {
                        op: SWAP { x: R8_B },
                        size: 2,
                        cycles: 8,
                        branch_cycles: None,
                    });
                }
                // swap c
                0x31 => {
                    return Ok(Instruction {
                        op: SWAP { x: R8_C },
                        size: 2,
                        cycles: 8,
                        branch_cycles: None,
                    });
                }
                // swap d
                0x32 => {
                    return Ok(Instruction {
                        op: SWAP { x: R8_D },
                        size: 2,
                        cycles: 8,
                        branch_cycles: None,
                    });
                }
                // swap e
                0x33 => {
                    return Ok(Instruction {
                        op: SWAP { x: R8_E },
                        size: 2,
                        cycles: 8,
                        branch_cycles: None,
                    });
                }
                // swap h
                0x34 => {
                    return Ok(Instruction {
                        op: SWAP { x: R8_H },
                        size: 2,
                        cycles: 8,
                        branch_cycles: None,
                    });
                }
                // swap l
                0x35 => {
                    return Ok(Instruction {
                        op: SWAP { x: R8_L },
                        size: 2,
                        cycles: 8,
                        branch_cycles: None,
                    });
                }
                // swap (hl)
                0x36 => {
                    return Ok(Instruction {
                        op: SWAP {
                            x: PTR(Box::new(R16_HL)),
                        },
                        size: 2,
                        cycles: 16,
                        branch_cycles: None,
                    });
                }
                // swap a
                0x37 => {
                    return Ok(Instruction {
                        op: SWAP { x: R8_A },
                        size: 2,
                        cycles: 8,
                        branch_cycles: None,
                    });
                }
                // srl b
                0x38 => {
                    return Ok(Instruction {
                        op: SRL { x: R8_B },
                        size: 2,
                        cycles: 8,
                        branch_cycles: None,
                    });
                }
                // srl c
                0x39 => {
                    return Ok(Instruction {
                        op: SRL { x: R8_C },
                        size: 2,
                        cycles: 8,
                        branch_cycles: None,
                    });
                }
                // srl d
                0x3A => {
                    return Ok(Instruction {
                        op: SRL { x: R8_D },
                        size: 2,
                        cycles: 8,
                        branch_cycles: None,
                    });
                }
                // srl e
                0x3B => {
                    return Ok(Instruction {
                        op: SRL { x: R8_E },
                        size: 2,
                        cycles: 8,
                        branch_cycles: None,
                    });
                }
                // srl h
                0x3C => {
                    return Ok(Instruction {
                        op: SRL { x: R8_H },
                        size: 2,
                        cycles: 8,
                        branch_cycles: None,
                    });
                }
                // srl l
                0x3D => {
                    return Ok(Instruction {
                        op: SRL { x: R8_L },
                        size: 2,
                        cycles: 8,
                        branch_cycles: None,
                    });
                }
                // srl (hl)
                0x3E => {
                    return Ok(Instruction {
                        op: SRL {
                            x: PTR(Box::new(R16_HL)),
                        },
                        size: 2,
                        cycles: 16,
                        branch_cycles: None,
                    });
                }
                // srl a
                0x3F => {
                    return Ok(Instruction {
                        op: SRL { x: R8_A },
                        size: 2,
                        cycles: 8,
                        branch_cycles: None,
                    });
                }
                // bit 0, b
                0x40 => {
                    return Ok(Instruction {
                        op: BIT { bit: 0, src: R8_B },
                        size: 2,
                        cycles: 8,
                        branch_cycles: None,
                    });
                }
                // bit 0, c
                0x41 => {
                    return Ok(Instruction {
                        op: BIT { bit: 0, src: R8_C },
                        size: 2,
                        cycles: 8,
                        branch_cycles: None,
                    });
                }
                // bit 0, d
                0x42 => {
                    return Ok(Instruction {
                        op: BIT { bit: 0, src: R8_D },
                        size: 2,
                        cycles: 8,
                        branch_cycles: None,
                    });
                }
                // bit 0, e
                0x43 => {
                    return Ok(Instruction {
                        op: BIT { bit: 0, src: R8_E },
                        size: 2,
                        cycles: 8,
                        branch_cycles: None,
                    });
                }
                // bit 0, h
                0x44 => {
                    return Ok(Instruction {
                        op: BIT { bit: 0, src: R8_H },
                        size: 2,
                        cycles: 8,
                        branch_cycles: None,
                    });
                }
                // bit 0, l
                0x45 => {
                    return Ok(Instruction {
                        op: BIT { bit: 0, src: R8_L },
                        size: 2,
                        cycles: 8,
                        branch_cycles: None,
                    });
                }
                // bit 0, (hl)
                0x46 => {
                    return Ok(Instruction {
                        op: BIT {
                            bit: 0,
                            src: PTR(Box::new(R16_HL)),
                        },
                        size: 2,
                        cycles: 16,
                        branch_cycles: None,
                    });
                }
                // bit 0, a
                0x47 => {
                    return Ok(Instruction {
                        op: BIT { bit: 0, src: R8_A },
                        size: 2,
                        cycles: 8,
                        branch_cycles: None,
                    });
                }
                // bit 1, b
                0x48 => {
                    return Ok(Instruction {
                        op: BIT { bit: 1, src: R8_B },
                        size: 2,
                        cycles: 8,
                        branch_cycles: None,
                    });
                }
                // bit 1, c
                0x49 => {
                    return Ok(Instruction {
                        op: BIT { bit: 1, src: R8_C },
                        size: 2,
                        cycles: 8,
                        branch_cycles: None,
                    });
                }
                // bit 1, d
                0x4A => {
                    return Ok(Instruction {
                        op: BIT { bit: 1, src: R8_D },
                        size: 2,
                        cycles: 8,
                        branch_cycles: None,
                    });
                }
                // bit 1, e
                0x4B => {
                    return Ok(Instruction {
                        op: BIT { bit: 1, src: R8_E },
                        size: 2,
                        cycles: 8,
                        branch_cycles: None,
                    });
                }
                // bit 1, h
                0x4C => {
                    return Ok(Instruction {
                        op: BIT { bit: 1, src: R8_H },
                        size: 2,
                        cycles: 8,
                        branch_cycles: None,
                    });
                }
                // bit 1, l
                0x4D => {
                    return Ok(Instruction {
                        op: BIT { bit: 1, src: R8_L },
                        size: 2,
                        cycles: 8,
                        branch_cycles: None,
                    });
                }
                // bit 1, (hl)
                0x4E => {
                    return Ok(Instruction {
                        op: BIT {
                            bit: 1,
                            src: PTR(Box::new(R16_HL)),
                        },
                        size: 2,
                        cycles: 16,
                        branch_cycles: None,
                    });
                }
                // bit 1, a
                0x4F => {
                    return Ok(Instruction {
                        op: BIT { bit: 1, src: R8_A },
                        size: 2,
                        cycles: 8,
                        branch_cycles: None,
                    });
                }
                // bit 2, b
                0x50 => {
                    return Ok(Instruction {
                        op: BIT { bit: 2, src: R8_B },
                        size: 2,
                        cycles: 8,
                        branch_cycles: None,
                    });
                }
                // bit 2, c
                0x51 => {
                    return Ok(Instruction {
                        op: BIT { bit: 2, src: R8_C },
                        size: 2,
                        cycles: 8,
                        branch_cycles: None,
                    });
                }
                // bit 2, d
                0x52 => {
                    return Ok(Instruction {
                        op: BIT { bit: 2, src: R8_D },
                        size: 2,
                        cycles: 8,
                        branch_cycles: None,
                    });
                }
                // bit 2, e
                0x53 => {
                    return Ok(Instruction {
                        op: BIT { bit: 2, src: R8_E },
                        size: 2,
                        cycles: 8,
                        branch_cycles: None,
                    });
                }
                // bit 2, h
                0x54 => {
                    return Ok(Instruction {
                        op: BIT { bit: 2, src: R8_H },
                        size: 2,
                        cycles: 8,
                        branch_cycles: None,
                    });
                }
                // bit 2, l
                0x55 => {
                    return Ok(Instruction {
                        op: BIT { bit: 2, src: R8_L },
                        size: 2,
                        cycles: 8,
                        branch_cycles: None,
                    });
                }
                // bit 2, (hl)
                0x56 => {
                    return Ok(Instruction {
                        op: BIT {
                            bit: 2,
                            src: PTR(Box::new(R16_HL)),
                        },
                        size: 2,
                        cycles: 16,
                        branch_cycles: None,
                    });
                }
                // bit 2, a
                0x57 => {
                    return Ok(Instruction {
                        op: BIT { bit: 2, src: R8_A },
                        size: 2,
                        cycles: 8,
                        branch_cycles: None,
                    });
                }
                // bit 3, b
                0x58 => {
                    return Ok(Instruction {
                        op: BIT { bit: 3, src: R8_B },
                        size: 2,
                        cycles: 8,
                        branch_cycles: None,
                    });
                }
                // bit 3, c
                0x59 => {
                    return Ok(Instruction {
                        op: BIT { bit: 3, src: R8_C },
                        size: 2,
                        cycles: 8,
                        branch_cycles: None,
                    });
                }
                // bit 3, d
                0x5A => {
                    return Ok(Instruction {
                        op: BIT { bit: 3, src: R8_D },
                        size: 2,
                        cycles: 8,
                        branch_cycles: None,
                    });
                }
                // bit 3, e
                0x5B => {
                    return Ok(Instruction {
                        op: BIT { bit: 3, src: R8_E },
                        size: 2,
                        cycles: 8,
                        branch_cycles: None,
                    });
                }
                // bit 3, h
                0x5C => {
                    return Ok(Instruction {
                        op: BIT { bit: 3, src: R8_H },
                        size: 2,
                        cycles: 8,
                        branch_cycles: None,
                    });
                }
                // bit 3, l
                0x5D => {
                    return Ok(Instruction {
                        op: BIT { bit: 3, src: R8_L },
                        size: 2,
                        cycles: 8,
                        branch_cycles: None,
                    });
                }
                // bit 3, (hl)
                0x5E => {
                    return Ok(Instruction {
                        op: BIT {
                            bit: 3,
                            src: PTR(Box::new(R16_HL)),
                        },
                        size: 2,
                        cycles: 16,
                        branch_cycles: None,
                    });
                }
                // bit 3, a
                0x5F => {
                    return Ok(Instruction {
                        op: BIT { bit: 3, src: R8_A },
                        size: 2,
                        cycles: 8,
                        branch_cycles: None,
                    });
                }
                // bit 4, b
                0x60 => {
                    return Ok(Instruction {
                        op: BIT { bit: 4, src: R8_B },
                        size: 2,
                        cycles: 8,
                        branch_cycles: None,
                    });
                }
                // bit 4, c
                0x61 => {
                    return Ok(Instruction {
                        op: BIT { bit: 4, src: R8_C },
                        size: 2,
                        cycles: 8,
                        branch_cycles: None,
                    });
                }
                // bit 4, d
                0x62 => {
                    return Ok(Instruction {
                        op: BIT { bit: 4, src: R8_D },
                        size: 2,
                        cycles: 8,
                        branch_cycles: None,
                    });
                }
                // bit 4, e
                0x63 => {
                    return Ok(Instruction {
                        op: BIT { bit: 4, src: R8_E },
                        size: 2,
                        cycles: 8,
                        branch_cycles: None,
                    });
                }
                // bit 4, h
                0x64 => {
                    return Ok(Instruction {
                        op: BIT { bit: 4, src: R8_H },
                        size: 2,
                        cycles: 8,
                        branch_cycles: None,
                    });
                }
                // bit 4, l
                0x65 => {
                    return Ok(Instruction {
                        op: BIT { bit: 4, src: R8_L },
                        size: 2,
                        cycles: 8,
                        branch_cycles: None,
                    });
                }
                // bit 4, (hl)
                0x66 => {
                    return Ok(Instruction {
                        op: BIT {
                            bit: 4,
                            src: PTR(Box::new(R16_HL)),
                        },
                        size: 2,
                        cycles: 16,
                        branch_cycles: None,
                    });
                }
                // bit 4, a
                0x67 => {
                    return Ok(Instruction {
                        op: BIT { bit: 4, src: R8_A },
                        size: 2,
                        cycles: 8,
                        branch_cycles: None,
                    });
                }
                // bit 5, b
                0x68 => {
                    return Ok(Instruction {
                        op: BIT { bit: 5, src: R8_B },
                        size: 2,
                        cycles: 8,
                        branch_cycles: None,
                    });
                }
                // bit 5, c
                0x69 => {
                    return Ok(Instruction {
                        op: BIT { bit: 5, src: R8_C },
                        size: 2,
                        cycles: 8,
                        branch_cycles: None,
                    });
                }
                // bit 5, d
                0x6A => {
                    return Ok(Instruction {
                        op: BIT { bit: 5, src: R8_D },
                        size: 2,
                        cycles: 8,
                        branch_cycles: None,
                    });
                }
                // bit 5, e
                0x6B => {
                    return Ok(Instruction {
                        op: BIT { bit: 5, src: R8_E },
                        size: 2,
                        cycles: 8,
                        branch_cycles: None,
                    });
                }
                // bit 5, h
                0x6C => {
                    return Ok(Instruction {
                        op: BIT { bit: 5, src: R8_H },
                        size: 2,
                        cycles: 8,
                        branch_cycles: None,
                    });
                }
                // bit 5, l
                0x6D => {
                    return Ok(Instruction {
                        op: BIT { bit: 5, src: R8_L },
                        size: 2,
                        cycles: 8,
                        branch_cycles: None,
                    });
                }
                // bit 5, (hl)
                0x6E => {
                    return Ok(Instruction {
                        op: BIT {
                            bit: 5,
                            src: PTR(Box::new(R16_HL)),
                        },
                        size: 2,
                        cycles: 16,
                        branch_cycles: None,
                    });
                }
                // bit 5, a
                0x6F => {
                    return Ok(Instruction {
                        op: BIT { bit: 5, src: R8_A },
                        size: 2,
                        cycles: 8,
                        branch_cycles: None,
                    });
                }
                // bit 6, b
                0x70 => {
                    return Ok(Instruction {
                        op: BIT { bit: 6, src: R8_B },
                        size: 2,
                        cycles: 8,
                        branch_cycles: None,
                    });
                }
                // bit 6, c
                0x71 => {
                    return Ok(Instruction {
                        op: BIT { bit: 6, src: R8_C },
                        size: 2,
                        cycles: 8,
                        branch_cycles: None,
                    });
                }
                // bit 6, d
                0x72 => {
                    return Ok(Instruction {
                        op: BIT { bit: 6, src: R8_D },
                        size: 2,
                        cycles: 8,
                        branch_cycles: None,
                    });
                }
                // bit 6, e
                0x73 => {
                    return Ok(Instruction {
                        op: BIT { bit: 6, src: R8_E },
                        size: 2,
                        cycles: 8,
                        branch_cycles: None,
                    });
                }
                // bit 6, h
                0x74 => {
                    return Ok(Instruction {
                        op: BIT { bit: 6, src: R8_H },
                        size: 2,
                        cycles: 8,
                        branch_cycles: None,
                    });
                }
                // bit 6, l
                0x75 => {
                    return Ok(Instruction {
                        op: BIT { bit: 6, src: R8_L },
                        size: 2,
                        cycles: 8,
                        branch_cycles: None,
                    });
                }
                // bit 6, (hl)
                0x76 => {
                    return Ok(Instruction {
                        op: BIT {
                            bit: 6,
                            src: PTR(Box::new(R16_HL)),
                        },
                        size: 2,
                        cycles: 16,
                        branch_cycles: None,
                    });
                }
                // bit 6, a
                0x77 => {
                    return Ok(Instruction {
                        op: BIT { bit: 6, src: R8_A },
                        size: 2,
                        cycles: 8,
                        branch_cycles: None,
                    });
                }
                // bit 7, b
                0x78 => {
                    return Ok(Instruction {
                        op: BIT { bit: 7, src: R8_B },
                        size: 2,
                        cycles: 8,
                        branch_cycles: None,
                    });
                }
                // bit 7, c
                0x79 => {
                    return Ok(Instruction {
                        op: BIT { bit: 7, src: R8_C },
                        size: 2,
                        cycles: 8,
                        branch_cycles: None,
                    });
                }
                // bit 7, d
                0x7A => {
                    return Ok(Instruction {
                        op: BIT { bit: 7, src: R8_D },
                        size: 2,
                        cycles: 8,
                        branch_cycles: None,
                    });
                }
                // bit 7, e
                0x7B => {
                    return Ok(Instruction {
                        op: BIT { bit: 7, src: R8_E },
                        size: 2,
                        cycles: 8,
                        branch_cycles: None,
                    });
                }
                // bit 7, h
                0x7C => {
                    return Ok(Instruction {
                        op: BIT { bit: 7, src: R8_H },
                        size: 2,
                        cycles: 8,
                        branch_cycles: None,
                    });
                }
                // bit 7, l
                0x7D => {
                    return Ok(Instruction {
                        op: BIT { bit: 7, src: R8_L },
                        size: 2,
                        cycles: 8,
                        branch_cycles: None,
                    });
                }
                // bit 7, (hl)
                0x7E => {
                    return Ok(Instruction {
                        op: BIT {
                            bit: 7,
                            src: PTR(Box::new(R16_HL)),
                        },
                        size: 2,
                        cycles: 16,
                        branch_cycles: None,
                    });
                }
                // bit 7, a
                0x7F => {
                    return Ok(Instruction {
                        op: BIT { bit: 7, src: R8_A },
                        size: 2,
                        cycles: 8,
                        branch_cycles: None,
                    });
                }
                // res 0, b
                0x80 => {
                    return Ok(Instruction {
                        op: RES { bit: 0, x: R8_B },
                        size: 2,
                        cycles: 8,
                        branch_cycles: None,
                    });
                }
                // res 0, c
                0x81 => {
                    return Ok(Instruction {
                        op: RES { bit: 0, x: R8_C },
                        size: 2,
                        cycles: 8,
                        branch_cycles: None,
                    });
                }
                // res 0, d
                0x82 => {
                    return Ok(Instruction {
                        op: RES { bit: 0, x: R8_D },
                        size: 2,
                        cycles: 8,
                        branch_cycles: None,
                    });
                }
                // res 0, e
                0x83 => {
                    return Ok(Instruction {
                        op: RES { bit: 0, x: R8_E },
                        size: 2,
                        cycles: 8,
                        branch_cycles: None,
                    });
                }
                // res 0, h
                0x84 => {
                    return Ok(Instruction {
                        op: RES { bit: 0, x: R8_H },
                        size: 2,
                        cycles: 8,
                        branch_cycles: None,
                    });
                }
                // res 0, l
                0x85 => {
                    return Ok(Instruction {
                        op: RES { bit: 0, x: R8_L },
                        size: 2,
                        cycles: 8,
                        branch_cycles: None,
                    });
                }
                // res 0, (hl)
                0x86 => {
                    return Ok(Instruction {
                        op: RES {
                            bit: 0,
                            x: PTR(Box::new(R16_HL)),
                        },
                        size: 2,
                        cycles: 16,
                        branch_cycles: None,
                    });
                }
                // res 0, a
                0x87 => {
                    return Ok(Instruction {
                        op: RES { bit: 0, x: R8_A },
                        size: 2,
                        cycles: 8,
                        branch_cycles: None,
                    });
                }
                // res 1, b
                0x88 => {
                    return Ok(Instruction {
                        op: RES { bit: 1, x: R8_B },
                        size: 2,
                        cycles: 8,
                        branch_cycles: None,
                    });
                }
                // res 1, c
                0x89 => {
                    return Ok(Instruction {
                        op: RES { bit: 1, x: R8_C },
                        size: 2,
                        cycles: 8,
                        branch_cycles: None,
                    });
                }
                // res 1, d
                0x8A => {
                    return Ok(Instruction {
                        op: RES { bit: 1, x: R8_D },
                        size: 2,
                        cycles: 8,
                        branch_cycles: None,
                    });
                }
                // res 1, e
                0x8B => {
                    return Ok(Instruction {
                        op: RES { bit: 1, x: R8_E },
                        size: 2,
                        cycles: 8,
                        branch_cycles: None,
                    });
                }
                // res 1, h
                0x8C => {
                    return Ok(Instruction {
                        op: RES { bit: 1, x: R8_H },
                        size: 2,
                        cycles: 8,
                        branch_cycles: None,
                    });
                }
                // res 1, l
                0x8D => {
                    return Ok(Instruction {
                        op: RES { bit: 1, x: R8_L },
                        size: 2,
                        cycles: 8,
                        branch_cycles: None,
                    });
                }
                // res 1, (hl)
                0x8E => {
                    return Ok(Instruction {
                        op: RES {
                            bit: 1,
                            x: PTR(Box::new(R16_HL)),
                        },
                        size: 2,
                        cycles: 16,
                        branch_cycles: None,
                    });
                }
                // res 1, a
                0x8F => {
                    return Ok(Instruction {
                        op: RES { bit: 1, x: R8_A },
                        size: 2,
                        cycles: 8,
                        branch_cycles: None,
                    });
                }
                // res 2, b
                0x90 => {
                    return Ok(Instruction {
                        op: RES { bit: 2, x: R8_B },
                        size: 2,
                        cycles: 8,
                        branch_cycles: None,
                    });
                }
                // res 2, c
                0x91 => {
                    return Ok(Instruction {
                        op: RES { bit: 2, x: R8_C },
                        size: 2,
                        cycles: 8,
                        branch_cycles: None,
                    });
                }
                // res 2, d
                0x92 => {
                    return Ok(Instruction {
                        op: RES { bit: 2, x: R8_D },
                        size: 2,
                        cycles: 8,
                        branch_cycles: None,
                    });
                }
                // res 2, e
                0x93 => {
                    return Ok(Instruction {
                        op: RES { bit: 2, x: R8_E },
                        size: 2,
                        cycles: 8,
                        branch_cycles: None,
                    });
                }
                // res 2, h
                0x94 => {
                    return Ok(Instruction {
                        op: RES { bit: 2, x: R8_H },
                        size: 2,
                        cycles: 8,
                        branch_cycles: None,
                    });
                }
                // res 2, l
                0x95 => {
                    return Ok(Instruction {
                        op: RES { bit: 2, x: R8_L },
                        size: 2,
                        cycles: 8,
                        branch_cycles: None,
                    });
                }
                // res 2, (hl)
                0x96 => {
                    return Ok(Instruction {
                        op: RES {
                            bit: 2,
                            x: PTR(Box::new(R16_HL)),
                        },
                        size: 2,
                        cycles: 16,
                        branch_cycles: None,
                    });
                }
                // res 2, a
                0x97 => {
                    return Ok(Instruction {
                        op: RES { bit: 2, x: R8_A },
                        size: 2,
                        cycles: 8,
                        branch_cycles: None,
                    });
                }
                // res 3, b
                0x98 => {
                    return Ok(Instruction {
                        op: RES { bit: 3, x: R8_B },
                        size: 2,
                        cycles: 8,
                        branch_cycles: None,
                    });
                }
                // res 3, c
                0x99 => {
                    return Ok(Instruction {
                        op: RES { bit: 3, x: R8_C },
                        size: 2,
                        cycles: 8,
                        branch_cycles: None,
                    });
                }
                // res 3, d
                0x9A => {
                    return Ok(Instruction {
                        op: RES { bit: 3, x: R8_D },
                        size: 2,
                        cycles: 8,
                        branch_cycles: None,
                    });
                }
                // res 3, e
                0x9B => {
                    return Ok(Instruction {
                        op: RES { bit: 3, x: R8_E },
                        size: 2,
                        cycles: 8,
                        branch_cycles: None,
                    });
                }
                // res 3, h
                0x9C => {
                    return Ok(Instruction {
                        op: RES { bit: 3, x: R8_H },
                        size: 2,
                        cycles: 8,
                        branch_cycles: None,
                    });
                }
                // res 3, l
                0x9D => {
                    return Ok(Instruction {
                        op: RES { bit: 3, x: R8_L },
                        size: 2,
                        cycles: 8,
                        branch_cycles: None,
                    });
                }
                // res 3, (hl)
                0x9E => {
                    return Ok(Instruction {
                        op: RES {
                            bit: 3,
                            x: PTR(Box::new(R16_HL)),
                        },
                        size: 2,
                        cycles: 16,
                        branch_cycles: None,
                    });
                }
                // res 3, a
                0x9F => {
                    return Ok(Instruction {
                        op: RES { bit: 3, x: R8_A },
                        size: 2,
                        cycles: 8,
                        branch_cycles: None,
                    });
                }
                // res 4, b
                0xA0 => {
                    return Ok(Instruction {
                        op: RES { bit: 4, x: R8_B },
                        size: 2,
                        cycles: 8,
                        branch_cycles: None,
                    });
                }
                // res 4, c
                0xA1 => {
                    return Ok(Instruction {
                        op: RES { bit: 4, x: R8_C },
                        size: 2,
                        cycles: 8,
                        branch_cycles: None,
                    });
                }
                // res 4, d
                0xA2 => {
                    return Ok(Instruction {
                        op: RES { bit: 4, x: R8_D },
                        size: 2,
                        cycles: 8,
                        branch_cycles: None,
                    });
                }
                // res 4, e
                0xA3 => {
                    return Ok(Instruction {
                        op: RES { bit: 4, x: R8_E },
                        size: 2,
                        cycles: 8,
                        branch_cycles: None,
                    });
                }
                // res 4, h
                0xA4 => {
                    return Ok(Instruction {
                        op: RES { bit: 4, x: R8_H },
                        size: 2,
                        cycles: 8,
                        branch_cycles: None,
                    });
                }
                // res 4, l
                0xA5 => {
                    return Ok(Instruction {
                        op: RES { bit: 4, x: R8_L },
                        size: 2,
                        cycles: 8,
                        branch_cycles: None,
                    });
                }
                // res 4, (hl)
                0xA6 => {
                    return Ok(Instruction {
                        op: RES {
                            bit: 4,
                            x: PTR(Box::new(R16_HL)),
                        },
                        size: 2,
                        cycles: 16,
                        branch_cycles: None,
                    });
                }
                // res 4, a
                0xA7 => {
                    return Ok(Instruction {
                        op: RES { bit: 4, x: R8_A },
                        size: 2,
                        cycles: 8,
                        branch_cycles: None,
                    });
                }
                // res 5, b
                0xA8 => {
                    return Ok(Instruction {
                        op: RES { bit: 5, x: R8_B },
                        size: 2,
                        cycles: 8,
                        branch_cycles: None,
                    });
                }
                // res 5, c
                0xA9 => {
                    return Ok(Instruction {
                        op: RES { bit: 5, x: R8_C },
                        size: 2,
                        cycles: 8,
                        branch_cycles: None,
                    });
                }
                // res 5, d
                0xAA => {
                    return Ok(Instruction {
                        op: RES { bit: 5, x: R8_D },
                        size: 2,
                        cycles: 8,
                        branch_cycles: None,
                    });
                }
                // res 5, e
                0xAB => {
                    return Ok(Instruction {
                        op: RES { bit: 5, x: R8_E },
                        size: 2,
                        cycles: 8,
                        branch_cycles: None,
                    });
                }
                // res 5, h
                0xAC => {
                    return Ok(Instruction {
                        op: RES { bit: 5, x: R8_H },
                        size: 2,
                        cycles: 8,
                        branch_cycles: None,
                    });
                }
                // res 5, l
                0xAD => {
                    return Ok(Instruction {
                        op: RES { bit: 5, x: R8_L },
                        size: 2,
                        cycles: 8,
                        branch_cycles: None,
                    });
                }
                // res 5, (hl)
                0xAE => {
                    return Ok(Instruction {
                        op: RES {
                            bit: 5,
                            x: PTR(Box::new(R16_HL)),
                        },
                        size: 2,
                        cycles: 16,
                        branch_cycles: None,
                    });
                }
                // res 5, a
                0xAF => {
                    return Ok(Instruction {
                        op: RES { bit: 5, x: R8_A },
                        size: 2,
                        cycles: 8,
                        branch_cycles: None,
                    });
                }
                // res 6, b
                0xB0 => {
                    return Ok(Instruction {
                        op: RES { bit: 6, x: R8_B },
                        size: 2,
                        cycles: 8,
                        branch_cycles: None,
                    });
                }
                // res 6, c
                0xB1 => {
                    return Ok(Instruction {
                        op: RES { bit: 6, x: R8_C },
                        size: 2,
                        cycles: 8,
                        branch_cycles: None,
                    });
                }
                // res 6, d
                0xB2 => {
                    return Ok(Instruction {
                        op: RES { bit: 6, x: R8_D },
                        size: 2,
                        cycles: 8,
                        branch_cycles: None,
                    });
                }
                // res 6, e
                0xB3 => {
                    return Ok(Instruction {
                        op: RES { bit: 6, x: R8_E },
                        size: 2,
                        cycles: 8,
                        branch_cycles: None,
                    });
                }
                // res 6, h
                0xB4 => {
                    return Ok(Instruction {
                        op: RES { bit: 6, x: R8_H },
                        size: 2,
                        cycles: 8,
                        branch_cycles: None,
                    });
                }
                // res 6, l
                0xB5 => {
                    return Ok(Instruction {
                        op: RES { bit: 6, x: R8_L },
                        size: 2,
                        cycles: 8,
                        branch_cycles: None,
                    });
                }
                // res 6, (hl)
                0xB6 => {
                    return Ok(Instruction {
                        op: RES {
                            bit: 6,
                            x: PTR(Box::new(R16_HL)),
                        },
                        size: 2,
                        cycles: 16,
                        branch_cycles: None,
                    });
                }
                // res 6, a
                0xB7 => {
                    return Ok(Instruction {
                        op: RES { bit: 6, x: R8_A },
                        size: 2,
                        cycles: 8,
                        branch_cycles: None,
                    });
                }
                // res 7, b
                0xB8 => {
                    return Ok(Instruction {
                        op: RES { bit: 7, x: R8_B },
                        size: 2,
                        cycles: 8,
                        branch_cycles: None,
                    });
                }
                // res 7, c
                0xB9 => {
                    return Ok(Instruction {
                        op: RES { bit: 7, x: R8_C },
                        size: 2,
                        cycles: 8,
                        branch_cycles: None,
                    });
                }
                // res 7, d
                0xBA => {
                    return Ok(Instruction {
                        op: RES { bit: 7, x: R8_D },
                        size: 2,
                        cycles: 8,
                        branch_cycles: None,
                    });
                }
                // res 7, e
                0xBB => {
                    return Ok(Instruction {
                        op: RES { bit: 7, x: R8_E },
                        size: 2,
                        cycles: 8,
                        branch_cycles: None,
                    });
                }
                // res 7, h
                0xBC => {
                    return Ok(Instruction {
                        op: RES { bit: 7, x: R8_H },
                        size: 2,
                        cycles: 8,
                        branch_cycles: None,
                    });
                }
                // res 7, l
                0xBD => {
                    return Ok(Instruction {
                        op: RES { bit: 7, x: R8_L },
                        size: 2,
                        cycles: 8,
                        branch_cycles: None,
                    });
                }
                // res 7, (hl)
                0xBE => {
                    return Ok(Instruction {
                        op: RES {
                            bit: 7,
                            x: PTR(Box::new(R16_HL)),
                        },
                        size: 2,
                        cycles: 16,
                        branch_cycles: None,
                    });
                }
                // res 7, a
                0xBF => {
                    return Ok(Instruction {
                        op: RES { bit: 7, x: R8_A },
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
        // call z, imm16
        0xCC => {
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
        // call imm16
        0xCD => {
            return Ok(Instruction {
                op: CALL { proc: IMM16(imm16) },
                size: 3,
                cycles: 24,
                branch_cycles: None,
            });
        }
        // adc a, imm8
        0xCE => {
            return Ok(Instruction {
                op: ADC { y: IMM8(imm8) },
                size: 2,
                cycles: 8,
                branch_cycles: None,
            });
        }
        // rst 08
        0xCF => {
            return Ok(Instruction {
                op: RST {
                    addr: Operand::IMM16(0x08),
                },
                size: 1,
                cycles: 16,
                branch_cycles: None,
            });
        }
        // ret nc
        0xD0 => {
            return Ok(Instruction {
                op: RET_CC { cc: CC_NC },
                size: 1,
                cycles: 8,
                branch_cycles: Some(20),
            });
        }
        // pop de
        0xD1 => {
            return Ok(Instruction {
                op: POP { reg: R16_DE },
                size: 1,
                cycles: 12,
                branch_cycles: None,
            });
        }
        // call nc, imm16
        0xD4 => {
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
        // push de
        0xD5 => {
            return Ok(Instruction {
                op: PUSH { reg: R16_DE },
                size: 1,
                cycles: 16,
                branch_cycles: None,
            });
        }
        // sub a, imm8
        0xD6 => {
            return Ok(Instruction {
                op: SUB { y: IMM8(imm8) },
                size: 2,
                cycles: 8,
                branch_cycles: None,
            });
        }
        // rst 10
        0xD7 => {
            return Ok(Instruction {
                op: RST {
                    addr: Operand::IMM16(0x10),
                },
                size: 1,
                cycles: 16,
                branch_cycles: None,
            });
        }
        // ret c
        0xD8 => {
            return Ok(Instruction {
                op: RET_CC { cc: CC_C },
                size: 1,
                cycles: 8,
                branch_cycles: Some(20),
            });
        }
        // reti
        0xD9 => {
            return Ok(Instruction {
                op: RETI,
                size: 1,
                cycles: 16,
                branch_cycles: None,
            });
        }
        // call c, imm16
        0xDC => {
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
        // rst 18
        0xDF => {
            return Ok(Instruction {
                op: RST {
                    addr: Operand::IMM16(0x18),
                },
                size: 1,
                cycles: 16,
                branch_cycles: None,
            });
        }
        // ld (imm8), a
        0xE0 => {
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
        // pop hl
        0xE1 => {
            return Ok(Instruction {
                op: POP { reg: R16_HL },
                size: 1,
                cycles: 12,
                branch_cycles: None,
            });
        }
        // ld (c), a
        0xE2 => {
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
        // push hl
        0xE5 => {
            return Ok(Instruction {
                op: PUSH { reg: R16_HL },
                size: 1,
                cycles: 16,
                branch_cycles: None,
            });
        }
        // and a, imm8
        0xE6 => {
            return Ok(Instruction {
                op: AND { y: IMM8(imm8) },
                size: 2,
                cycles: 8,
                branch_cycles: None,
            });
        }
        // rst 20
        0xE7 => {
            return Ok(Instruction {
                op: RST {
                    addr: Operand::IMM16(0x20),
                },
                size: 1,
                cycles: 16,
                branch_cycles: None,
            });
        }
        // jp hl
        0xE9 => {
            return Ok(Instruction {
                op: JP { addr: R16_HL },
                size: 1,
                cycles: 4,
                branch_cycles: None,
            });
        }
        // ld (imm16), a
        0xEA => {
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
        // xor a, imm8
        0xEE => {
            return Ok(Instruction {
                op: XOR { y: IMM8(imm8) },
                size: 2,
                cycles: 8,
                branch_cycles: None,
            });
        }
        // rst 28
        0xEF => {
            return Ok(Instruction {
                op: RST {
                    addr: Operand::IMM16(0x28),
                },
                size: 1,
                cycles: 16,
                branch_cycles: None,
            });
        }
        // ld a, (imm8)
        0xF0 => {
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
        // pop af
        0xF1 => {
            return Ok(Instruction {
                op: POP { reg: R16_AF },
                size: 1,
                cycles: 12,
                branch_cycles: None,
            });
        }
        // ld a, (c)
        0xF2 => {
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
        // di
        0xF3 => {
            return Ok(Instruction {
                op: DI,
                size: 1,
                cycles: 4,
                branch_cycles: None,
            });
        }
        // push af
        0xF5 => {
            return Ok(Instruction {
                op: PUSH { reg: R16_AF },
                size: 1,
                cycles: 16,
                branch_cycles: None,
            });
        }
        // or a, imm8
        0xF6 => {
            return Ok(Instruction {
                op: OR { y: IMM8(imm8) },
                size: 2,
                cycles: 8,
                branch_cycles: None,
            });
        }
        // rst 30
        0xF7 => {
            return Ok(Instruction {
                op: RST {
                    addr: Operand::IMM16(0x30),
                },
                size: 1,
                cycles: 16,
                branch_cycles: None,
            });
        }
        // ld hl, sp + imm8
        0xF8 => {
            return Ok(Instruction {
                op: LD {
                    dst: R16_HL,
                    src: SP_PLUS_SIGNED_IMM8(i8::from_le_bytes([imm8])),
                },
                size: 2,
                cycles: 12,
                branch_cycles: None,
            });
        }
        // ld sp, hl
        0xF9 => {
            return Ok(Instruction {
                op: LD {
                    dst: R16_SP,
                    src: R16_HL,
                },
                size: 1,
                cycles: 8,
                branch_cycles: None,
            });
        }
        // ld a, (imm16)
        0xFA => {
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
        // ei
        0xFB => {
            return Ok(Instruction {
                op: EI,
                size: 1,
                cycles: 4,
                branch_cycles: None,
            });
        }
        // cp imm8
        0xFE => {
            return Ok(Instruction {
                op: CP { y: IMM8(imm8) },
                size: 2,
                cycles: 8,
                branch_cycles: None,
            });
        }
        // rst 38
        0xFF => {
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
        Operand::SP_PLUS_SIGNED_IMM8(imm8) => format!("sp + {}", imm8),
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
        Operation::JP_CC { cc, addr } => format!("jp {cc}, {addr}"),
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
        Operation::RETI => String::from("reti"),
        Operation::PUSH { reg: word } => format!("push {word}"),
        Operation::POP { reg: word } => format!("pop {word}"),
        Operation::DEC { x } => format!("dec {x}"),
        Operation::INC { x } => format!("inc {x}"),
        Operation::ADD { x, y } => format!("add {x}, {y}"),
        Operation::ADC { y } => format!("adc a, {y}"),
        Operation::SUB { y } => format!("sub a, {y}"),
        Operation::OR { y } => format!("or a, {y}"),
        Operation::XOR { y } => format!("xor a, {y}"),
        Operation::AND { y } => format!("and a, {y}"),
        Operation::BIT { bit, src: r8 } => format!("bit {bit}, {r8}"),
        Operation::RES { bit, x: src } => format!("res {bit}, {src}"),
        Operation::SWAP { x } => format!("swap {x}"),
        Operation::RL { x } => format!("rl {x}"),
        Operation::RR { x } => format!("rr {x}"),
        Operation::RLC { x } => format!("rlc {x}"),
        Operation::RRC { x } => format!("rrc {x}"),
        Operation::RLA => String::from("rla"),
        Operation::RRA => String::from("rra"),
        Operation::RLCA => String::from("rlca"),
        Operation::RRCA => String::from("rrca"),
        Operation::SRL { x } => format!("srl {x}"),
        Operation::SLA { x } => format!("sla {x}"),
        Operation::CP { y } => format!("cp {y}"),
        Operation::DI => String::from("di"),
        Operation::EI => String::from("ei"),
        Operation::CPL => String::from("cpl"),
        Operation::HALT => String::from("halt"),
    }
}

impl fmt::Display for Instruction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", instruction_to_string(self))
    }
}
