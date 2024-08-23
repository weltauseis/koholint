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

        // https://gb-archive.github.io/salvage/decoding_gbz80_opcodes/Decoding%20Gamboy%20Z80%20Opcodes.html
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
        macro_rules! xor_a_r8 {
            ($self:ident, $reg:ident) => {
                paste! {
                    let a = $self.cpu.get_a();
                    let r8 = $self.cpu.[<get_ $reg>]();

                    $self.cpu.set_a(a ^ r8);
                    $self.cpu.increment_pc(1);
                }
            };
        }

        match instr {
            // ld r16, imm16
            0x01 => {
                debug!("ld bc, imm16");
                ld_r16_imm16!(self, bc);
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
                let a = self.cpu.get_a();
                let byte = self.mem.read_byte(self.cpu.get_hl());
                self.cpu.set_a(a ^ byte);
                self.cpu.increment_pc(1);
            }
            0xAF => {
                debug!("xor a, a");
                xor_a_r8!(self, a);
            }

            _ => panic!("UNHANDLED INSTRUCTION ({:#X})", instr),
        };
    }
}
