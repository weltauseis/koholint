use crate::decoding::Operand;

pub struct CPU {
    // 8 & 16 bits registers
    a: u8,
    f: u8, // flags registers
    b: u8, // BC
    c: u8,
    d: u8, // DE
    e: u8,
    h: u8, // HL
    l: u8,
    sp: u16, // stack pointer
    pc: u16, // program counter
}

enum Flags {
    Zero,
    Substraction,
    HalfCarry,
    Carry,
}

impl CPU {
    // constructor
    pub fn blank() -> CPU {
        return CPU {
            a: 0,
            f: 0,
            b: 0,
            c: 0,
            d: 0,
            e: 0,
            h: 0,
            l: 0,
            sp: 0,
            pc: 0,
        };
    }

    // accessors
    pub fn read_stack_pointer(&self) -> u16 {
        return self.sp;
    }

    // functions to access the registers using an instruction operand

    pub fn read_r8(&self, r8: &Operand) -> u8 {
        return match r8 {
            Operand::R8_A => self.a,
            Operand::R8_B => self.b,
            Operand::R8_C => self.c,
            Operand::R8_D => self.d,
            Operand::R8_E => self.e,
            Operand::R8_H => self.h,
            Operand::R8_L => self.l,
            _ => panic!("GET_R8 : INVALID REGISTER ({:?})", r8),
        };
    }

    pub fn write_r8(&mut self, r8: &Operand, value: u8) {
        match r8 {
            Operand::R8_A => {
                self.a = value;
            }
            Operand::R8_B => {
                self.b = value;
            }
            Operand::R8_C => {
                self.c = value;
            }
            Operand::R8_D => {
                self.d = value;
            }
            Operand::R8_E => {
                self.e = value;
            }
            Operand::R8_H => {
                self.h = value;
            }
            Operand::R8_L => {
                self.l = value;
            }
            _ => panic!("SET_R8 : INVALID REGISTER ({:?})", r8),
        };
    }

    pub fn read_r16(&self, r16: &Operand) -> u16 {
        return match r16 {
            Operand::R16_BC => u16::from_le_bytes([self.b, self.c]),
            Operand::R16_DE => u16::from_le_bytes([self.d, self.e]),
            Operand::R16_HL | Operand::R16_HLD => u16::from_le_bytes([self.h, self.l]),
            Operand::R16_SP => self.sp,
            _ => panic!("GET_R16 : INVALID REGISTER ({:?})", r16),
        };
    }

    pub fn write_r16(&mut self, r16: &Operand, value: u16) {
        let bytes = value.to_le_bytes();
        match r16 {
            Operand::R16_BC => {
                self.b = bytes[0];
                self.c = bytes[1];
            }
            Operand::R16_DE => {
                self.d = bytes[0];
                self.e = bytes[1];
            }
            Operand::R16_HL | Operand::R16_HLD => {
                self.h = bytes[0];
                self.l = bytes[1];
            }
            Operand::R16_SP => {
                self.sp = value;
            }
            _ => panic!("SET_R16 : INVALID REGISTER ({:?})", r16),
        };
    }

    pub fn read_program_counter(&self) -> u16 {
        return self.pc;
    }

    pub fn write_program_counter(&mut self, value: u16) {
        self.pc = value;
    }

    pub fn increment_program_counter(&mut self, increment: u16) {
        self.pc = self.pc.wrapping_add(increment);
    }

    pub fn offset_program_counter(&mut self, offset: i8) {
        if offset > 0 {
            self.pc = self.pc.wrapping_add(offset.abs() as u16);
        } else {
            self.pc = self.pc.wrapping_sub(offset.abs() as u16);
        }
    }

    // flags register :
    // 7 | 6 | 5 | 4 | 3 | 2 | 1 | 0 (bit n°)
    // Z | N | H | C | 0 | 0 | 0 | 0 (flag)

    pub fn read_z_flag(&self) -> bool {
        return ((self.f >> 7) & 1) == 1;
    }

    pub fn write_z_flag(&mut self, value: bool) {
        if value {
            self.f |= 0b_1000_0000;
        } else {
            self.f &= !0b_1000_0000;
        }
    }

    pub fn read_n_flag(&self) -> bool {
        return ((self.f >> 6) & 1) == 1;
    }

    pub fn write_n_flag(&mut self, value: bool) {
        if value {
            self.f |= 0b_0100_0000;
        } else {
            self.f &= !0b_0100_0000;
        }
    }

    pub fn read_h_flag(&self) -> bool {
        return ((self.f >> 5) & 1) == 1;
    }

    pub fn write_h_flag(&mut self, value: bool) {
        if value {
            self.f |= 0b_0010_0000;
        } else {
            self.f &= !0b_0010_0000;
        }
    }

    pub fn read_c_flag(&self) -> bool {
        return ((self.f >> 4) & 1) == 1;
    }

    pub fn write_c_flag(&mut self, value: bool) {
        if value {
            self.f |= 0b_0001_0000;
        } else {
            self.f &= !0b_0001_0000;
        }
    }

    // access flags using a condition operand
    pub fn get_cc(&self, cc: &Operand) -> bool {
        match cc {
            Operand::CC_NZ => !self.read_z_flag(),
            _ => panic!("GET_CC : INVALID CONDITION ({:?})", cc),
        }
    }
}
