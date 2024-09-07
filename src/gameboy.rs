use crate::{
    cpu::CPU,
    decoding::{self, Instruction, Operand, Operation},
    memory::Memory,
};

pub struct Gameboy {
    cpu: CPU,
    memory: Memory,
}

impl Gameboy {
    // constructor
    pub fn new(rom: Vec<u8>) -> Gameboy {
        let mut mem = Memory::new();
        mem.load_rom(rom);
        return Gameboy {
            cpu: CPU::blank(),
            memory: mem,
        };
    }

    // accessors to watch values
    pub fn cpu(&self) -> &CPU {
        return &self.cpu;
    }

    pub fn memory(&self) -> &Memory {
        return &self.memory;
    }

    // functions

    pub fn step(&mut self) {
        let instr = decoding::decode_next_instruction(&self);
        self.execute_instruction(instr);
    }

    // returns a 256 * 256 image (32 * 32 tiles)
    // the gameboy holds only 384 tiles, so that's 32 * 12
    // so a good chunk of the atlas is empty
    pub fn get_tiles_as_rgba8unorm_atlas(&self) -> [u8; 4 * 256 * 256] {
        let mut img = [0u8; 4 * 256 * 256];

        // https://gbdev.io/pandocs/Tile_Data.html
        // each tile is 16 bytes in memory
        // each couple of bytes encodes a line of the tile

        // for each tile
        for id in 0..384usize {
            let address = (0x8000 + id * 16) as u16;

            // for each line of the tile
            for y in 0..8 {
                let byte_1 = self.memory.read_byte(address + y * 2);
                let byte_2 = self.memory.read_byte(address + y * 2 + 1);

                for x in 0..8 {
                    let mut value: u8 = 0;

                    let bit_0 = byte_1 >> (7 - x) & 1;
                    let bit_1 = byte_2 >> (7 - x) & 1;

                    value |= bit_0;
                    value |= bit_1 << 1;

                    let pixel =
                        // tile start                              | pixel start
                        (8 * (id % 32) + (8 * 8 * 32) * (id / 32) + ((y as usize) * 8 * 32) + (x as usize)) * 4;

                    img[pixel..(pixel + 4)].copy_from_slice(match value {
                        0 => &[15, 15, 27, /* alpha */ 255],
                        1 => &[86, 90, 117, /* alpha */ 255],
                        2 => &[198, 183, 190, /* alpha */ 255],
                        3 => &[250, 251, 246, /* alpha */ 255],
                        _ => &[255, 0, 0, 255, /* alpha */ 255],
                    })
                }
            }
        }

        /* for pixel in 0..(256 * 256) {
            img[(pixel * 4)..(pixel * 4 + 4)].copy_from_slice(
                if (((pixel % 256) / 32) % 2 == 0) && (((pixel / 256) / 32) % 2 == 1) {
                    &[255, 0, 0, 255]
                } else {
                    &[0, 0, 255, 255]
                },
            );
        } */

        return img;
    }

    pub fn get_tile_map(&self) -> [u32; 32 * 32] {
        //https://gbdev.io/pandocs/Tile_Maps.html
        let mut tilemap = [0; 32 * 32];
        let _addressing_mode_bit = self.memory.read_lcd_ctrl_flag(4);

        for i in 0..(32 * 32) {
            let mem_index = self.memory.read_byte(0x9800 + i);
            tilemap[i as usize] = mem_index as u32;
            /* tilemap[i as usize] = if addressing_mode_bit {
                mem_index as u32
            } else {
                if (mem_index as usize) < 128 {
                    256 + mem_index as u32
                } else {
                    255 - mem_index as u32
                }
            }; */
        }

        return tilemap;
    }

    fn execute_instruction(&mut self, instr: Instruction) {
        use Operand::*;

        // store instruction pc for crash messages
        let pc = self.cpu.read_program_counter();

        // increment PC before everything
        // seems consistent with the fact that relative jumps
        // are relative to the end of the jr instructionss

        self.cpu.increment_program_counter(instr.size);

        match instr.op {
            Operation::NOP => {
                // nothing to do
            }
            Operation::LD { dst, src } => {
                // some instructions auto-increment the hl register
                // the timing is important
                let mut decrement_hl = false;
                let mut increment_hl = false;

                match dst {
                    // load into a 8-bit register
                    R8_A | R8_B | R8_C | R8_D | R8_E | R8_H | R8_L => {
                        let byte = match src {
                            // load from immediate byte
                            IMM8(imm8) => imm8,

                            // load from another 8-bit register
                            R8_A | R8_B | R8_C | R8_D | R8_E | R8_H | R8_L => {
                                self.cpu.read_r8(&src)
                            }

                            // load from memory with pointer
                            PTR(ptr) => match *ptr {
                                R16_BC | R16_DE | R16_HL | R16_HLD | R16_HLI => {
                                    if matches!(*ptr, R16_HLI) {
                                        increment_hl = true;
                                    }
                                    if matches!(*ptr, R16_HLD) {
                                        decrement_hl = true;
                                    }

                                    let address = self.cpu.read_r16(&ptr);
                                    self.memory.read_byte(address)
                                }
                                // address from imm8 : IO memory
                                IMM8(imm8) => self.memory.read_byte(0xFF00 + imm8 as u16),
                                _ => panic!("(CRITICAL) LD : ILLEGAL POINTER {ptr} at {pc:#06X}"),
                            },
                            _ => panic!("(CRITICAL) LD : ILLEGAL SRC {src} at {pc:#06X}"),
                        };

                        self.cpu.write_r8(&dst, byte);
                    }
                    // load into a 16-bit register
                    R16_BC | R16_DE | R16_HL | R16_SP => {
                        let word = match src {
                            // load from immediate word
                            IMM16(imm16) => imm16,

                            // load from another 16-bit register
                            R16_BC | R16_DE | R16_HL | R16_SP => self.cpu.read_r16(&src),

                            // load from memory
                            // for 16-bit load, the memory location is always
                            // relative to the stack pointer, with a signed offset
                            PTR(ptr) => match *ptr {
                                IMM8_SIGNED(offset) => {
                                    let sp = self.cpu.read_r16(&R16_SP);
                                    sp.wrapping_add(offset as u16)
                                }
                                _ => panic!("(CRITICAL) LD : ILLEGAL STACK POINTER OFFSET {ptr} at {pc:#06X}"),
                            },
                            _ => panic!("(CRITICAL) LD : ILLEGAL SRC {src} at {pc:#06X}"),
                        };

                        self.cpu.write_r16(&dst, word);
                    }
                    // load into memory
                    PTR(ptr) => {
                        let address = match *ptr {
                            // address from pointer in r16
                            R16_BC | R16_DE | R16_HL | R16_HLD | R16_HLI => {
                                if matches!(*ptr, R16_HLI) {
                                    increment_hl = true;
                                }
                                if matches!(*ptr, R16_HLD) {
                                    decrement_hl = true;
                                }
                                self.cpu.read_r16(&ptr)
                            }
                            // address from immediate word
                            IMM16(address) => address,
                            // address from r8 : IO memory
                            R8_C => 0xFF00 + self.cpu.read_r8(&R8_C) as u16,
                            // address from imm8 : IO memory
                            IMM8(imm8) => 0xFF00 + imm8 as u16,
                            _ => {
                                panic!("(CRITICAL) LD : ILLEGAL DST POINTER {ptr} at {pc:#06X}")
                            }
                        };

                        match src {
                            // load byte from r8
                            R8_A | R8_B | R8_C | R8_D | R8_E | R8_H | R8_L => {
                                self.memory.write_byte(address, self.cpu.read_r8(&src));
                            }
                            // load word from sp register
                            R16_SP => {
                                self.memory.write_word(address, self.cpu.read_r16(&R16_SP));
                            }
                            // load immediate byte
                            IMM8(imm8) => {
                                self.memory.write_byte(address, imm8);
                            }
                            _ => panic!("(CRITICAL) LD : ILLEGAL SRC {src} at {pc:#06X}"),
                        }
                    }
                    _ => panic!("LD : UNHANDLED DESTINATION {dst} at {pc:#06X}"),
                }

                if increment_hl {
                    self.cpu
                        .write_r16(&R16_HL, self.cpu.read_r16(&R16_HL).wrapping_add(1));
                }
                if decrement_hl {
                    self.cpu
                        .write_r16(&R16_HL, self.cpu.read_r16(&R16_HL).wrapping_sub(1));
                }
            }
            Operation::INC { x } => match x {
                // increment 8-bit register
                R8_A | R8_B | R8_C | R8_D | R8_E | R8_H | R8_L => {
                    let reg = self.cpu.read_r8(&x);
                    let result = reg.wrapping_add(1);
                    self.cpu.write_r8(&x, result);

                    // inc flags : Z 0 H -
                    self.cpu.write_z_flag(result == 0);
                    self.cpu.write_n_flag(false);
                    self.cpu.write_h_flag((reg & 0xF) == 0xF);
                }
                // increment 16-bit register
                R16_BC | R16_DE | R16_HL | R16_SP => {
                    let reg = self.cpu.read_r16(&x);
                    let result = reg.wrapping_add(1);
                    self.cpu.write_r16(&x, result);

                    // no flags for 16-bit increment
                }
                // memory at address in hl
                PTR(ptr) => match *ptr {
                    R16_HL => {
                        let address = self.cpu.read_r16(&R16_HL);
                        let byte = self.memory.read_byte(address);
                        let result = byte.wrapping_add(1);
                        self.memory.write_byte(address, result);

                        // inc flags : Z 0 H -
                        self.cpu.write_z_flag(result == 0);
                        self.cpu.write_n_flag(false);
                        self.cpu.write_h_flag((byte & 0xF) == 0xF);
                    }
                    _ => panic!("(CRITICAL) INC : ILLEGAL POINTER {ptr} at {pc:#06X}"),
                },

                _ => panic!("(CRITICAL) INC : ILLEGAL OPERAND {x} at {pc:#06X}"),
            },
            Operation::DEC { x } => match x {
                // decrement 8-bit register
                R8_A | R8_B | R8_C | R8_D | R8_E | R8_H | R8_L => {
                    let reg = self.cpu.read_r8(&x);
                    let result = reg.wrapping_sub(1);
                    self.cpu.write_r8(&x, result);

                    // dec flags : Z 1 H -
                    self.cpu.write_z_flag(result == 0);
                    self.cpu.write_n_flag(true);
                    self.cpu.write_h_flag((reg & 0xF) == 0);
                }
                // decrement 16-bit register
                R16_BC | R16_DE | R16_HL | R16_SP => {
                    let reg = self.cpu.read_r16(&x);
                    let result = reg.wrapping_sub(1);
                    self.cpu.write_r16(&x, result);
                }
                // memory at address in hl
                PTR(ptr) => match *ptr {
                    R16_HL => {
                        let address = self.cpu.read_r16(&R16_HL);
                        let byte = self.memory.read_byte(address);
                        let result = byte.wrapping_sub(1);
                        self.memory.write_byte(address, result);

                        // dec flags : Z 1 H -
                        self.cpu.write_z_flag(result == 0);
                        self.cpu.write_n_flag(true);
                        self.cpu.write_h_flag((byte & 0xF) == 0);
                    }
                    _ => panic!("(CRITICAL) DEC : ILLEGAL POINTER {ptr} at {pc:#06X}"),
                },

                _ => panic!("(CRITICAL) DEC : ILLEGAL OPERAND {x} at {pc:#06X}"),
            },
            Operation::ADD_8 { y } => {
                // add does a + y and stores the result in a
                let a = self.cpu.read_a_register();
                let value = match y {
                    // subtract 8-bit register
                    R8_A | R8_B | R8_C | R8_D | R8_E | R8_H | R8_L => self.cpu.read_r8(&y),
                    // subtract imm8
                    IMM8(imm8) => imm8,
                    // subtract from memory with pointer in hl
                    PTR(ptr) => match *ptr {
                        R16_HL => {
                            let hl = self.cpu.read_r16(&ptr);
                            self.memory.read_byte(hl)
                        }
                        _ => panic!("(CRITICAL) ADD : ILLEGAL POINTER {ptr} at {pc:#06X}"),
                    },
                    _ => panic!("(CRITICAL) ADD : ILLEGAL SECOND OPERAND {y} at {pc:#06X}"),
                };

                let result = a.wrapping_add(value);
                self.cpu.write_a_register(result);

                // flags : z 0 h c
                self.cpu.write_z_flag(result == 0);
                self.cpu.write_n_flag(false);
                self.cpu.write_h_flag((a & 0xF) + (value & 0xF) > 0xF);
                self.cpu.write_c_flag(a < value);
            }
            Operation::SUB { y } => {
                // sub does a - y and stores the result in a
                let a = self.cpu.read_a_register();
                let value = match y {
                    // subtract 8-bit register
                    R8_A | R8_B | R8_C | R8_D | R8_E | R8_H | R8_L => self.cpu.read_r8(&y),
                    // subtract imm8
                    IMM8(imm8) => imm8,
                    // subtract from memory with pointer in hl
                    PTR(ptr) => match *ptr {
                        R16_HL => {
                            let hl = self.cpu.read_r16(&ptr);
                            self.memory.read_byte(hl)
                        }
                        _ => panic!("(CRITICAL) SUB : ILLEGAL POINTER {ptr} at {pc:#06X}"),
                    },
                    _ => panic!("(CRITICAL) SUB : ILLEGAL SECOND OPERAND {y} at {pc:#06X}"),
                };

                let result = a.wrapping_sub(value);
                self.cpu.write_a_register(result);

                // flags : z 1 h c
                self.cpu.write_z_flag(result == 0);
                self.cpu.write_n_flag(true);
                self.cpu.write_h_flag((a & 0xF) < (value & 0xF));
                self.cpu.write_c_flag(a < value);
            }
            Operation::XOR { y } => {
                // xor is always done with the a register as first operand (x)
                let a = self.cpu.read_r8(&R8_A);
                let other = match y {
                    // second operand can only be another 8-bit register or pointer in hl
                    R8_A | R8_B | R8_C | R8_D | R8_E | R8_H | R8_L => self.cpu.read_r8(&y),
                    PTR(ptr) => match *ptr {
                        R16_HL => self.memory.read_byte(self.cpu.read_r16(&R16_HL)),
                        _ => panic!("(CRITICAL) XOR : ILLEGAL POINTER {ptr} at {pc:#06X}"),
                    },
                    _ => panic!("(CRITICAL) XOR : ILLEGAL SECOND OPERAND {y} at {pc:#06X}"),
                };

                self.cpu.write_r8(&R8_A, a ^ other);

                // xor flags : Z 0 0 0
                self.cpu.write_z_flag(self.cpu.read_r8(&R8_A) == 0);
                self.cpu.write_n_flag(false);
                self.cpu.write_h_flag(false);
                self.cpu.write_c_flag(false);
            }
            Operation::BIT { bit, src } => {
                // test bit in register / memory, set the zero flag to complement of bit

                let byte = match src {
                    // test bit in 8-bit register
                    R8_A | R8_B | R8_C | R8_D | R8_E | R8_H | R8_L => self.cpu.read_r8(&src),
                    // test bit in memory
                    PTR(ptr) => match *ptr {
                        R16_HL => self.memory.read_byte(self.cpu.read_r16(&R16_HL)),
                        _ => panic!("(CRITICAL) BIT : ILLEGAL POINTER {ptr} at {pc:#06X}"),
                    },
                    _ => panic!("(CRITICAL) BIT : ILLEGAL SRC {src} at {pc:#06X}"),
                };

                // bit instruction flags : Z 0 1 -
                self.cpu.write_z_flag((byte >> bit) & 1 == 0); // true if bit is 0, false if bit is 1
                self.cpu.write_n_flag(false);
                self.cpu.write_h_flag(true);
            }
            Operation::RL { x } => {
                match x {
                    // rotate 8-bit register
                    R8_A | R8_B | R8_C | R8_D | R8_E | R8_H | R8_L => {
                        let mut to_rotate = self.cpu.read_r8(&x);

                        // b7 to carry
                        let previous_carry: u8 = if self.cpu.read_c_flag() { 1 } else { 0 };
                        self.cpu.write_c_flag((to_rotate >> 7) & 1 == 1);

                        // rotate number left with carry
                        to_rotate <<= 1;
                        to_rotate |= previous_carry;

                        // write back the number
                        self.cpu.write_r8(&x, to_rotate);

                        // flags : z 0 0 c
                        self.cpu.write_z_flag(to_rotate == 0);
                        self.cpu.write_n_flag(false);
                        self.cpu.write_h_flag(false);
                    }
                    // rotate memory byte
                    PTR(ptr) => match *ptr {
                        R16_HL => {
                            let address = self.cpu.read_r16(&ptr);
                            let mut to_rotate = self.memory.read_byte(address);

                            // b7 to carry
                            let previous_carry: u8 = if self.cpu.read_c_flag() { 1 } else { 0 };
                            self.cpu.write_c_flag((to_rotate >> 7) & 1 == 1);

                            // rotate number left with carry
                            to_rotate <<= 1;
                            to_rotate |= previous_carry;

                            // write back the number
                            self.memory.write_byte(address, to_rotate);

                            // flags : z 0 0 c
                            self.cpu.write_z_flag(to_rotate == 0);
                            self.cpu.write_n_flag(false);
                            self.cpu.write_h_flag(false);
                        }
                        _ => panic!("(CRITICAL) RL : ILLEGAL POINTER {ptr} at {pc:#06X}"),
                    },
                    _ => panic!("(CRITICAL) RL : ILLEGAL OPERAND {x} at {pc:#06X}"),
                }
            }
            Operation::RR { x } => {
                match x {
                    // rotate 8-bit register
                    R8_A | R8_B | R8_C | R8_D | R8_E | R8_H | R8_L => {
                        let mut to_rotate = self.cpu.read_r8(&x);

                        // b0 to carry
                        let previous_carry: u8 = if self.cpu.read_c_flag() { 1 } else { 0 };
                        self.cpu.write_c_flag((to_rotate) & 1 == 1);

                        // rotate number right with carry
                        to_rotate >>= 1;
                        to_rotate |= previous_carry << 7;

                        // write back the number
                        self.cpu.write_r8(&x, to_rotate);

                        // flags : z 0 0 c
                        self.cpu.write_z_flag(to_rotate == 0);
                        self.cpu.write_n_flag(false);
                        self.cpu.write_h_flag(false);
                    }
                    // rotate memory byte
                    PTR(ptr) => match *ptr {
                        R16_HL => {
                            let address = self.cpu.read_r16(&ptr);
                            let mut to_rotate = self.memory.read_byte(address);

                            // b0 to carry
                            let previous_carry: u8 = if self.cpu.read_c_flag() { 1 } else { 0 };
                            self.cpu.write_c_flag((to_rotate) & 1 == 1);

                            // rotate number right with carry
                            to_rotate >>= 1;
                            to_rotate |= previous_carry << 7;

                            // write back the number
                            self.memory.write_byte(address, to_rotate);

                            // flags : z 0 0 c
                            self.cpu.write_z_flag(to_rotate == 0);
                            self.cpu.write_n_flag(false);
                            self.cpu.write_h_flag(false);
                        }
                        _ => panic!("(CRITICAL) RR : ILLEGAL POINTER {ptr} at {pc:#06X}"),
                    },
                    _ => panic!("(CRITICAL) RR : ILLEGAL OPERAND {x} at {pc:#06X}"),
                }
            }
            Operation::RLC { x } => {
                match x {
                    // rotate 8-bit register
                    R8_A | R8_B | R8_C | R8_D | R8_E | R8_H | R8_L => {
                        let mut to_rotate = self.cpu.read_r8(&x);

                        // b7 to carry
                        let previous_b7: u8 = (to_rotate >> 7) & 1;
                        self.cpu.write_c_flag(previous_b7 == 1);

                        // rotate number left with carry
                        to_rotate <<= 1;
                        to_rotate |= previous_b7;

                        // write back the number
                        self.cpu.write_r8(&x, to_rotate);

                        // flags : z 0 0 c
                        self.cpu.write_z_flag(to_rotate == 0);
                        self.cpu.write_n_flag(false);
                        self.cpu.write_h_flag(false);
                    }
                    // rotate memory byte
                    PTR(ptr) => match *ptr {
                        R16_HL => {
                            let address = self.cpu.read_r16(&ptr);
                            let mut to_rotate = self.memory.read_byte(address);

                            // b7 to carry
                            let previous_b7: u8 = (to_rotate >> 7) & 1;
                            self.cpu.write_c_flag(previous_b7 == 1);

                            // rotate number left with carry
                            to_rotate <<= 1;
                            to_rotate |= previous_b7;

                            // write back the number
                            self.memory.write_byte(address, to_rotate);

                            // flags : z 0 0 c
                            self.cpu.write_z_flag(to_rotate == 0);
                            self.cpu.write_n_flag(false);
                            self.cpu.write_h_flag(false);
                        }
                        _ => panic!("(CRITICAL) RLC : ILLEGAL POINTER {ptr} at {pc:#06X}"),
                    },
                    _ => panic!("(CRITICAL) RLC : ILLEGAL OPERAND {x} at {pc:#06X}"),
                }
            }
            Operation::RRC { x } => {
                match x {
                    // rotate 8-bit register
                    R8_A | R8_B | R8_C | R8_D | R8_E | R8_H | R8_L => {
                        let mut to_rotate = self.cpu.read_r8(&x);

                        // b0 to carry
                        let previous_b0: u8 = (to_rotate) & 1;
                        self.cpu.write_c_flag(previous_b0 == 1);

                        // rotate number left with carry
                        to_rotate >>= 1;
                        to_rotate |= previous_b0 << 7;

                        // write back the number
                        self.cpu.write_r8(&x, to_rotate);

                        // flags : z 0 0 c
                        self.cpu.write_z_flag(to_rotate == 0);
                        self.cpu.write_n_flag(false);
                        self.cpu.write_h_flag(false);
                    }
                    // rotate memory byte
                    PTR(ptr) => match *ptr {
                        R16_HL => {
                            let address = self.cpu.read_r16(&ptr);
                            let mut to_rotate = self.memory.read_byte(address);

                            // b0 to carry
                            let previous_b0: u8 = (to_rotate) & 1;
                            self.cpu.write_c_flag(previous_b0 == 1);

                            // rotate number left with carry
                            to_rotate >>= 1;
                            to_rotate |= previous_b0 << 7;

                            // write back the number
                            self.memory.write_byte(address, to_rotate);

                            // flags : z 0 0 c
                            self.cpu.write_z_flag(to_rotate == 0);
                            self.cpu.write_n_flag(false);
                            self.cpu.write_h_flag(false);
                        }
                        _ => panic!("(CRITICAL) RRC : ILLEGAL POINTER {ptr} at {pc:#06X}"),
                    },
                    _ => panic!("(CRITICAL) RRC : ILLEGAL OPERAND {x} at {pc:#06X}"),
                }
            }
            Operation::RLA => {
                // rotate a register
                let mut to_rotate = self.cpu.read_r8(&R8_A);

                // b7 to carry
                let previous_carry: u8 = if self.cpu.read_c_flag() { 1 } else { 0 };
                self.cpu.write_c_flag((to_rotate >> 7) & 1 == 1);

                // rotate number left with carry
                to_rotate <<= 1;
                to_rotate |= previous_carry;

                // write back the number
                self.cpu.write_r8(&R8_A, to_rotate);

                // flags : 0 0 0 c
                self.cpu.write_z_flag(false);
                self.cpu.write_n_flag(false);
                self.cpu.write_h_flag(false);
            }
            Operation::RRA => {
                // rotate a register
                let mut to_rotate = self.cpu.read_r8(&R8_A);

                // b0 to carry
                let previous_carry: u8 = if self.cpu.read_c_flag() { 1 } else { 0 };
                self.cpu.write_c_flag((to_rotate) & 1 == 1);

                // rotate number left with carry
                to_rotate >>= 1;
                to_rotate |= previous_carry << 7;

                // write back the number
                self.cpu.write_r8(&R8_A, to_rotate);

                // flags : 0 0 0 c
                self.cpu.write_z_flag(false);
                self.cpu.write_n_flag(false);
                self.cpu.write_h_flag(false);
            }
            Operation::RLCA => {
                let mut to_rotate = self.cpu.read_r8(&R8_A);

                // b7 to carry
                let previous_b7: u8 = (to_rotate >> 7) & 1;
                self.cpu.write_c_flag(previous_b7 == 1);

                // rotate number left with carry
                to_rotate <<= 1;
                to_rotate |= previous_b7;

                // write back the number
                self.cpu.write_r8(&R8_A, to_rotate);

                // flags : 0 0 0 c
                self.cpu.write_z_flag(false);
                self.cpu.write_n_flag(false);
                self.cpu.write_h_flag(false);
            }
            Operation::RRCA {} => {
                // rotate 8-bit register

                let mut to_rotate = self.cpu.read_r8(&R8_A);

                // b0 to carry
                let previous_b0: u8 = (to_rotate) & 1;
                self.cpu.write_c_flag(previous_b0 == 1);

                // rotate number left with carry
                to_rotate >>= 1;
                to_rotate |= previous_b0 << 7;

                // write back the number
                self.cpu.write_r8(&R8_A, to_rotate);

                // flags : 0 0 0 c
                self.cpu.write_z_flag(false);
                self.cpu.write_n_flag(false);
                self.cpu.write_h_flag(false);
            }
            Operation::JR { offset_oprd } => {
                let offset = match offset_oprd {
                    IMM8_SIGNED(offset) => offset,
                    _ => panic!("(CRITICAL) JR : ILLEGAL OFFSET {offset_oprd} at {pc:#06X}"),
                };

                self.cpu.offset_program_counter(offset);
            }
            Operation::JR_CC { cc, offset_oprd } => {
                let should_jump = self.cpu.get_cc(&cc);
                if should_jump {
                    let offset = match offset_oprd {
                        IMM8_SIGNED(offset) => offset,
                        _ => panic!("(CRITICAL) JR_CC : ILLEGAL OFFSET {offset_oprd} at {pc:#06X}"),
                    };

                    self.cpu.offset_program_counter(offset);
                }
            }
            Operation::JP { addr } => {
                // jp instruction only takes either an imm16 or the hl register
                let address = match addr {
                    IMM16(imm16) => imm16,
                    R16_HL => self.cpu.read_hl_register(),
                    _ => panic!("(CRITICAL) JP : ILLEGAL ADDRESS {addr} at {pc:#06X}"),
                };

                self.cpu.write_program_counter(address);
            }
            Operation::CALL { proc } => {
                let address = match proc {
                    IMM16(imm16) => imm16,
                    _ => {
                        panic!("(CRITICAL) CALL : ILLEGAL PROCEDURE ADDRESS {proc} at {pc:#06X}")
                    }
                };

                // push the return address to the stack
                let current_pc = self.cpu.read_program_counter();
                self.push_word(current_pc);

                // jump to the procedure
                self.cpu.write_program_counter(address);
            }

            Operation::RET => {
                let return_address = self.pop_word();

                // jump to where the procedure was called
                self.cpu.write_program_counter(return_address);
            }

            Operation::PUSH { reg } => {
                let to_push = match reg {
                    R16_BC | R16_DE | R16_HL | R16_AF => self.cpu.read_r16(&reg),
                    _ => panic!("(CRITICAL) PUSH : ILLEGAL OPERAND {reg} at {pc:#06X}"),
                };

                self.push_word(to_push);
            }

            Operation::POP { reg } => {
                match reg {
                    R16_BC | R16_DE | R16_HL | R16_AF => {
                        let word = self.pop_word();
                        self.cpu.write_r16(&reg, word);
                    }
                    _ => panic!("(CRITICAL) POP : ILLEGAL OPERAND {reg} at {pc:#06X}"),
                };
            }

            Operation::CP { y } => {
                // compare register a with another value
                let a = self.cpu.read_r8(&R8_A);
                let other = match y {
                    // second operand can be another 8-bit register, imm8 or pointer in hl
                    R8_A | R8_B | R8_C | R8_D | R8_E | R8_H | R8_L => self.cpu.read_r8(&y),
                    IMM8(imm8) => imm8,
                    PTR(ptr) => match *ptr {
                        R16_HL => self.memory.read_byte(self.cpu.read_r16(&R16_HL)),
                        _ => panic!("(CRITICAL) CP : ILLEGAL POINTER {ptr} at {pc:#06X}"),
                    },
                    _ => panic!("(CRITICAL) CP : ILLEGAL SECOND OPERAND {y} at {pc:#06X}"),
                };

                // cp flags : Z 1 H C
                self.cpu.write_z_flag(a == other);
                self.cpu.write_n_flag(true);
                self.cpu.write_h_flag((a & 0xF) < (other & 0xF));
                self.cpu.write_c_flag(a < other);
            }
            _ => panic!(
                "EXECUTION : UNHANDLED INSTRUCTION ({instr}) at PC {:#06X}",
                self.cpu.read_program_counter()
            ),
        }
    }
    // utilities common to multiple opcodes

    /* fn push_byte(&mut self, byte: u8) {
        // decrement stack pointer
        self.cpu.offset_stack_pointer(-1);

        // write byte
        self.memory.write_byte(self.cpu.read_stack_pointer(), byte);
    } */

    fn push_word(&mut self, word: u16) {
        // decrement stack pointer
        self.cpu.offset_stack_pointer(-2);

        // write word
        self.memory.write_word(self.cpu.read_stack_pointer(), word);
    }

    /* fn pop_byte(&mut self) -> u8 {
        // read byte
        let byte = self.memory.read_byte(self.cpu.read_stack_pointer());

        // decrement stack pointer
        self.cpu.offset_stack_pointer(1);

        return byte;
    } */

    fn pop_word(&mut self) -> u16 {
        // read word
        let word = self.memory.read_word(self.cpu.read_stack_pointer());

        // decrement stack pointer
        self.cpu.offset_stack_pointer(2);

        return word;
    }
}
