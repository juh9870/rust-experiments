use crate::errors::MsError;
use strum_macros::EnumMessage;

use super::{
    register::{StackOffset, VmStackExtension},
    Vm,
};

#[non_exhaustive]
#[derive(EnumMessage, Debug, Clone, Copy)]
pub enum OpCode {
    #[strum(message = "Returns a value at a register (0)")]
    Return(StackOffset),
    #[strum(message = "Assigns a register (0) to a constant value at index (1) in current chunk")]
    SetConstant(StackOffset, usize),
    #[strum(message = "Prints a value at a register (0)")]
    Print(StackOffset),
}

impl OpCode {
    pub fn run(&self, vm: &mut Vm) -> Result<(), MsError> {
        match self {
            OpCode::Return(_) => Err(MsError::NotImplemented),
            OpCode::SetConstant(to, index) => {
                let value = vm.chunk.constants[*index].clone();
                vm.stack.set(*to, value)?;
                Ok(())
            }
            OpCode::Print(at) => {
                let value = vm.stack.get(*at)?;
                dbg!(value);
                Ok(())
            }
        }
    }
}
