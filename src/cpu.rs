use std::rc::Rc;

use log::trace;

use crate::memory::Memory;

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
    pub fn get_a(&self) -> u8 {
        return self.a;
    }
    pub fn set_a(&mut self, value: u8) {
        self.a = value;
    }

    pub fn get_b(&self) -> u8 {
        return self.b;
    }

    pub fn set_b(&mut self, value: u8) {
        self.b = value;
    }

    pub fn get_c(&self) -> u8 {
        return self.c;
    }

    pub fn set_c(&mut self, value: u8) {
        self.c = value;
    }

    pub fn get_bc(&self) -> u16 {
        return u16::from_le_bytes([self.b, self.c]);
    }

    pub fn set_bc(&mut self, value: u16) {
        let bytes = value.to_le_bytes();
        self.b = bytes[0];
        self.c = bytes[1];
    }

    pub fn get_d(&self) -> u8 {
        return self.d;
    }

    pub fn set_d(&mut self, value: u8) {
        self.d = value;
    }

    pub fn get_e(&self) -> u8 {
        return self.e;
    }

    pub fn set_e(&mut self, value: u8) {
        self.e = value;
    }

    pub fn get_de(&self) -> u16 {
        return u16::from_le_bytes([self.d, self.e]);
    }

    pub fn set_de(&mut self, value: u16) {
        let bytes = value.to_le_bytes();
        self.d = bytes[0];
        self.e = bytes[1];
    }

    pub fn get_h(&self) -> u8 {
        return self.h;
    }

    pub fn set_h(&mut self, value: u8) {
        self.h = value;
    }

    pub fn get_l(&self) -> u8 {
        return self.l;
    }

    pub fn set_l(&mut self, value: u8) {
        self.l = value;
    }

    pub fn get_hl(&self) -> u16 {
        return u16::from_le_bytes([self.h, self.l]);
    }

    pub fn set_hl(&mut self, value: u16) {
        let bytes = value.to_le_bytes();
        self.h = bytes[0];
        self.l = bytes[1];
    }

    pub fn get_sp(&self) -> u16 {
        return self.sp;
    }

    pub fn set_sp(&mut self, value: u16) {
        self.sp = value;
    }

    pub fn get_pc(&self) -> u16 {
        return self.pc;
    }

    pub fn set_pc(&mut self, value: u16) {
        self.pc = value;
    }

    pub fn increment_pc(&mut self, value: u16) {
        self.pc += value;
    }
}
