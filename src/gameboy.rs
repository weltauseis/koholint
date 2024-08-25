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

    fn execute_instruction(&mut self, instr: Instruction) {
        use Operand::*;

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
                    R8_A | R8_B | R8_C | R8_D | R8_E | R8_H | R8_L => match src {
                        /*  IMM8(imm8) => {
                            todo!()
                        }
                        PTR(ptr) => {
                            todo!()
                        } */
                        _ => panic!("LD : UNHANDLED SOURCE"),
                    },
                    // load into a 16-bit register
                    R16_BC | R16_DE | R16_HL | R16_SP => match src {
                        IMM16(imm16) => {
                            self.cpu.write_r16(dst, imm16);
                        }
                        _ => panic!("LD : UNHANDLED SOURCE"),
                    },
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
                                self.cpu.read_r16(*ptr)
                            }
                            // address from immediate word
                            IMM16(address) => address,
                            // address from r8 : IO memory
                            R8_C => 0xFF00 + self.cpu.read_r8(R8_C) as u16,
                            // address from imm8 : IO memory
                            IMM8(imm8) => 0xFF00 + imm8 as u16,
                            _ => panic!("(CRITICAL) LD : ILLEGAL DST POINTER {ptr}"),
                        };

                        match src {
                            // load byte from r8
                            R8_A | R8_B | R8_C | R8_D | R8_E | R8_H | R8_L => {
                                self.memory.write_byte(address, self.cpu.read_r8(src));
                            }
                            // load word from sp register
                            R16_SP => {
                                self.memory.write_word(address, self.cpu.read_r16(R16_SP));
                            }
                            // load immediate byte
                            IMM8(imm8) => {
                                self.memory.write_byte(address, imm8);
                            }
                            _ => panic!("(CRITICAL) LD : ILLEGAL SRC {src}"),
                        }
                    }
                    _ => panic!("LD : UNHANDLED DESTINATION"),
                }

                if increment_hl {
                    self.cpu
                        .write_r16(R16_HL, self.cpu.read_r16(R16_HL).wrapping_add(1));
                }
                if decrement_hl {
                    self.cpu
                        .write_r16(R16_HL, self.cpu.read_r16(R16_HL).wrapping_sub(1));
                }
            }
            Operation::XOR { y } => {
                // xor is always done with the a register as first operand (x)
                let a = self.cpu.read_r8(R8_A);
                let other = match y {
                    // second operand can only be another 8-bit register or pointer in hl
                    R8_A | R8_B | R8_C | R8_D | R8_E | R8_H | R8_L => self.cpu.read_r8(y),
                    PTR(ptr) => match *ptr {
                        R16_HL => self.memory.read_byte(self.cpu.read_r16(R16_HL)),
                        _ => panic!("(CRITICAL) XOR : ILLEGAL POINTER {ptr}"),
                    },
                    _ => panic!("(CRITICAL) XOR : ILLEGAL SECOND OPERAND {y}"),
                };

                self.cpu.write_r8(R8_A, a ^ other);

                // xor flags : Z 0 0 0
                self.cpu.write_z_flag(self.cpu.read_r8(R8_A) == 0);
                self.cpu.write_n_flag(false);
                self.cpu.write_h_flag(false);
                self.cpu.write_c_flag(false);
            }
            Operation::BIT { bit, src } => {
                // test bit in register / memory, set the zero flag to complement of bit

                let byte = match src {
                    // test bit in 8-bit register
                    R8_A | R8_B | R8_C | R8_D | R8_E | R8_H | R8_L => self.cpu.read_r8(src),
                    // test bit in memory
                    PTR(ptr) => match *ptr {
                        R16_HL => self.memory.read_byte(self.cpu.read_r16(R16_HL)),
                        _ => panic!("(CRITICAL) BIT : ILLEGAL POINTER {ptr}"),
                    },
                    _ => panic!("(CRITICAL) BIT : ILLEGAL SRC {src}"),
                };

                // bit instruction flags : Z 0 1 -
                self.cpu.write_z_flag((byte >> bit) & 1 == 0); // true if bit is 0, false if bit is 1
                self.cpu.write_n_flag(false);
                self.cpu.write_h_flag(true);
            }
            /* Operation::JP_IMM16 { imm16 } => {
                self.cpu.set_program_counter(imm16);
            }
            Operation::JR_CC_R8 { cc, imm8 } => {
                // THE OFFSET IS SIGNED !!
                self.cpu.increment_program_counter(instr.size);
                if self.cpu.get_cc(cc) {
                    self.cpu.offset_program_counter(imm8);
                }
            }
            Operation::XOR_A_R8 { r8 } => {
                let a = self.cpu.get_r8(R8_A);
                let reg = self.cpu.get_r8(r8);
                self.cpu.set_r8(R8_A, a ^ reg);

                self.cpu.increment_program_counter(instr.size);

                self.cpu.set_z_flag(self.cpu.get_r8(R8_A) == 0);
            }
            Operation::LD_R8_IMM8 { r8, imm8 } => {
                self.cpu.set_r8(r8, imm8);

                self.cpu.increment_program_counter(instr.size);
            }
            Operation::LD_PTR_R8 { ptr, r8 } => {
                let address = self.cpu.get_r16(ptr);

                self.memory.write_byte(address, self.cpu.get_r8(r8));

                if matches!(ptr, R16_HLD) {
                    let hl = self.cpu.get_r16(R16_HL);
                    self.cpu.set_r16(R16_HL, hl.wrapping_sub(1));
                }

                self.cpu.increment_program_counter(instr.size);
            }
            Operation::DEC { x: r8 } => {
                let reg = self.cpu.get_r8(r8);
                let result = reg.wrapping_sub(1);

                self.cpu.set_z_flag(result == 0);
                self.cpu.set_n_flag(true);
                self.cpu.set_h_flag(reg == 0);

                self.cpu.set_r8(r8, result);

                self.cpu.increment_program_counter(instr.size);
            } */
            _ => panic!(
                "EXECUTION : UNHANDLED INSTRUCTION ({instr}) at PC {:#06X}",
                self.cpu.read_program_counter()
            ),
        }

        /*
            let instr = self.mem.read_byte(pc);

            // https://rgbds.gbdev.io/docs
            // https://gbdev.io/gb-opcodes//optables/

            #[macro_export]
            macro_rules! dec_r8 {
                ($self:ident, $reg:ident) => {
                    paste! {
                        let r8 = $self.cpu.[<get_ $reg>]();
                        let result = r8.wrapping_sub(1);

                        $self.cpu.set_z_flag(result == 0);
                        $self.cpu.set_n_flag(true);
                        $self.cpu.set_h_flag((r8 & 0xF) == 0);

                        $self.cpu.[<set_ $reg>](result);
                        $self.cpu.increment_pc(1);
                    }
                };
            }

            #[macro_export]
            macro_rules! jr_cc_imm8 {
                // THE OFFSET IS SIGNED !!
                ($self:ident, $flag:ident, $value: expr) => {
                    paste! {
                        let imm8 = $self.mem.read_byte($self.cpu.get_pc() + 1);

                        let offset = i8::from_le_bytes([imm8]);

                        trace!("Relative jump offset : {}", offset);

                        $self.cpu.increment_pc(2);
                        if $self.cpu.[<get_ $flag _flag>]() == $value {
                            $self.cpu.increment_pc(offset);
                        }
                    }
                };
            }

            //debug!("Executing instruction at pc = {:#06X} :", pc);
            match instr {
                // ld r16, imm16
                0x01 => {
                    debug!("ld bc, imm16");
                    ld_r16_imm16!(self, bc);
                }
                0x05 => {
                    debug!("dec b");
                    dec_r8!(self, b);
                }
                0x06 => {
                    debug!("ld b, imm8");
                    ld_r8_imm8!(self, b);
                }
                0x0D => {
                    debug!("dec c");
                    dec_r8!(self, c);
                }
                0x0E => {
                    debug!("ld c, imm8");
                    ld_r8_imm8!(self, e);
                }
                0x11 => {
                    debug!("ld de, imm16");
                    ld_r16_imm16!(self, de);
                }
                0x20 => {
                    debug!("jr nz, imm8");
                    jr_cc_imm8!(self, z, false);
                }
                0x21 => {
                    debug!("ld hl, imm16");
                    ld_r16_imm16!(self, hl);
                }
                0x31 => {
                    debug!("ld sp, imm16");
                    ld_r16_imm16!(self, sp);
                }
                0x32 => {
                    debug!("ld (hl-), a");
                    let hl = self.cpu.get_hl();
                    let a = self.cpu.get_a();
                    self.mem.write_byte(hl, a);
                    self.cpu.set_hl(hl - 1);
                    self.cpu.offset_program_counter(1);
                }
                // xor a, r8
                0xA8 => {
                    debug!("xor a, b");
                    xor_a_r8!(self, b);
                }
                0xA9 => {
                    debug!("xor a, c");
                    xor_a_r8!(self, c);
                }
                0xAA => {
                    debug!("xor a, d");
                    xor_a_r8!(self, d);
                }
                0xAB => {
                    debug!("xor a, e");
                    xor_a_r8!(self, e);
                }
                0xAC => {
                    debug!("xor a, h");
                    xor_a_r8!(self, h);
                }
                0xAD => {
                    debug!("xor a, l");
                    xor_a_r8!(self, l);
                }
                0xAE => {
                    debug!("xor a, (hl)");
                    // special case of byte loaded from address in register
                    let a = self.cpu.get_a();
                    let hl = self.cpu.get_hl();
                    let byte = self.mem.read_byte(hl);
                    self.cpu.set_a(a ^ byte);
                    self.cpu.offset_program_counter(1);
                }
                0xAF => {
                    debug!("xor a, a");
                    xor_a_r8!(self, a);
                }
                0xC3 => {
                    debug!("jp imm16");
                    let address = self.mem.read_word(pc + 1);
                    self.cpu.set_program_counter(address);
                }

                _ => panic!("UNHANDLED INSTRUCTION ({:#04X}) at PC {:#06X}", instr, pc),
            };
        }
          */
    }
}
