use std::fmt::Display;

use crate::decoding::Instruction;

#[derive(Debug)]
pub struct EmulationError {
    pub ty: EmulationErrorType,
    pub pc: Option<u16>,
}

#[derive(Debug)]
pub enum EmulationErrorType {
    UnhandledInstructionDecode(u16),
    UnhandledInstructionExec(Instruction),
    UnauthorizedWrite(u16),
}

impl Display for EmulationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} {}",
            match &self.ty {
                EmulationErrorType::UnhandledInstructionDecode(opcode) => format!(
                    "Unhandled instruction during decoding : {}",
                    if *opcode > 0xFF {
                        format!("{opcode:#06X}")
                    } else {
                        format!("{opcode:#04X}")
                    }
                ),
                EmulationErrorType::UnhandledInstructionExec(instr) =>
                    format!("Unhandled instruction during execution : {}", instr),
                EmulationErrorType::UnauthorizedWrite(address) =>
                    format!("Unauthorized write (Address : {:#06X})", address),
            },
            match self.pc {
                Some(pc) => format!("at PC {:#06X}", pc),
                None => "".to_string(),
            }
        )
    }
}
