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
            Operation::PUSH { reg } => {
                let to_push = match reg {
                    R16_BC | R16_DE | R16_HL | R16_AF => self.cpu.read_r16(&reg),
                    _ => panic!("(CRITICAL) PUSH : ILLEGAL OPERAND {reg} at {pc:#06X}"),
                };

                self.push_word(to_push);
            }
            /* Operation::JP_IMM16 { imm16 } => {
                self.cpu.set_program_counter(imm16);
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

    // utilities common to multiple opcodes

    fn push_byte(&mut self, byte: u8) {
        // decrement stack pointer
        self.cpu.offset_stack_pointer(-1);

        // write byte
        self.memory.write_byte(self.cpu.read_stack_pointer(), byte);
    }

    fn push_word(&mut self, word: u16) {
        // decrement stack pointer
        self.cpu.offset_stack_pointer(-2);

        // write word
        self.memory.write_word(self.cpu.read_stack_pointer(), word);
    }
}
