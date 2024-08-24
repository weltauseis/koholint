use log::debug;
use paste::paste;

use crate::{cpu::CPU, memory::Memory};

pub struct Gameboy {
    cpu: CPU,
    mem: Memory,
}

impl Gameboy {
    // constructor
    pub fn new(rom: Vec<u8>) -> Gameboy {
        let mut mem = Memory::new();
        mem.load_rom(rom);
        return Gameboy {
            cpu: CPU::blank(),
            mem,
        };
    }

    // functions

    pub fn step(&mut self) {
        let pc = self.cpu.get_pc();
        let instr = self.mem.read_byte(pc);

        // https://rgbds.gbdev.io/docs
        // https://gbdev.io/gb-opcodes//optables/

        #[macro_export]
        macro_rules! ld_r16_imm16 {
            ($self:ident, $reg:ident) => {
                paste! {
                    let imm16 = $self.mem.read_word($self.cpu.get_pc() + 1);
                    $self.cpu.[<set_ $reg>](imm16);
                    $self.cpu.increment_pc(3);
                }
            };
        }

        #[macro_export]
        macro_rules! ld_r8_imm8 {
            ($self:ident, $reg:ident) => {
                paste! {
                    let imm8 = $self.mem.read_byte($self.cpu.get_pc() + 1);
                    $self.cpu.[<set_ $reg>](imm8);
                    $self.cpu.increment_pc(2);
                }
            };
        }

        #[macro_export]
        macro_rules! xor_a_r8 {
            ($self:ident, $reg:ident) => {
                paste! {
                    let a = $self.cpu.get_a();

                    let r8 = self.cpu.[<get_ $reg>]();

                    $self.cpu.set_a(a ^ r8);
                    $self.cpu.increment_pc(1);

                    $self.cpu.set_z_flag($self.cpu.get_a() == 0);

                }
            };
        }

        match instr {
            // ld r16, imm16
            0x01 => {
                debug!("ld bc, imm16");
                ld_r16_imm16!(self, bc);
            }
            0x06 => {
                debug!("ld b, imm8");
                ld_r8_imm8!(self, b);
            }
            0x0E => {
                debug!("ld c, imm8");
                ld_r8_imm8!(self, e);
            }
            0x11 => {
                debug!("ld de, imm16");
                ld_r16_imm16!(self, de);
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
                self.cpu.increment_pc(1);
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
                self.cpu.increment_pc(1);
            }
            0xAF => {
                debug!("xor a, a");
                xor_a_r8!(self, a);
            }
            0xC3 => {
                debug!("jp imm16");
                let address = self.mem.read_word(pc + 1);
                self.cpu.set_pc(address);
            }

            _ => panic!("UNHANDLED INSTRUCTION ({:#04X}) at PC {:#06X}", instr, pc),
        };
    }
}
