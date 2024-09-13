use std::fmt::Display;

#[derive(Debug)]
pub struct EmulationError {
    pub ty: EmulationErrorType,
    pub pc: u16,
}

#[derive(Debug)]
pub enum OpCode {
    Op(u8),
    Ext(u8),
}

#[derive(Debug)]
pub enum EmulationErrorType {
    UnhandledInstruction(OpCode),
    UnauthorizedRead(),
    UnauthorizedWrite(u16),
}

impl Display for EmulationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} at PC {:#06X}",
            match &self.ty {
                EmulationErrorType::UnhandledInstruction(opcode) => match opcode {
                    OpCode::Op(simple) =>
                        format!("Unhandled instruction during decoding : {:#04X}", simple),
                    OpCode::Ext(extended) => format!(
                        "Unhandled instruction during decoding : 0xCB{:02X}",
                        extended
                    ),
                },
                EmulationErrorType::UnauthorizedRead() => todo!(),
                EmulationErrorType::UnauthorizedWrite(address) =>
                    format!("Unauthorized write (Address : {:#06X})", address),
            },
            self.pc
        )
    }
}
