use crate::errors::{InternalError, MsError};
use crate::value::Value;
use crate::vm::chunk::ConstantIndex;
use std::result;
use strum_macros::EnumMessage;

use super::{register::StackIndex, Vm};

///
/// As a rule of thumb, first argument is the "target" of a bytecode operation
#[non_exhaustive]
#[derive(EnumMessage, Debug, Clone)]
pub enum OpCode {
    #[strum(message = "Returns a value at a register (0)")]
    Return(StackIndex),

    #[strum(message = "Sets register at an index to a constant numeric value")]
    SetNumber(StackIndex, f64),
    #[strum(message = "Sets register at an index to a null value")]
    SetNull(StackIndex),
    #[strum(message = "Assigns a register (0) to a constant string at index (1) in current chunk")]
    SetString(StackIndex, ConstantIndex),

    #[strum(
        message = "Attempts to find a value identified by (1) in all visible contexts and write it to index (0)"
    )]
    ReadVariable(StackIndex, String),

    // Function calls
    #[strum(message = "Calls a function with 0 arguments")]
    Call0 {
        function: StackIndex,
        output: StackIndex,
    },
    #[strum(message = "Calls a function with one argument")]
    Call1 {
        function: StackIndex,
        output: StackIndex,
        arg: StackIndex,
    },
    #[strum(message = "Calls a function with many argumetns")]
    Call {
        function: StackIndex,
        output: StackIndex,
        argument_count: u8,
    },

    // Binary operators
    #[strum(message = "Adds values at (lhs) and (rhs) and writes result to (output)")]
    Add {
        output: StackIndex,
        lhs: StackIndex,
        rhs: StackIndex,
    },
    #[strum(message = "Subtracts value at (rhs) from (lhs) and writes result to (output)")]
    Subtract {
        output: StackIndex,
        lhs: StackIndex,
        rhs: StackIndex,
    },
    #[strum(message = "Multiplies values at (lhs) and (rhs) and writes result to (output)")]
    Multiply {
        output: StackIndex,
        lhs: StackIndex,
        rhs: StackIndex,
    },
    #[strum(message = "Subtracts value at (lhs) by (rhs) and writes result to (output)")]
    Divide {
        output: StackIndex,
        lhs: StackIndex,
        rhs: StackIndex,
    },
    #[strum(message = "Raises value at (lhs) to the power of (rhs) and writes result to (output)")]
    Pow {
        output: StackIndex,
        lhs: StackIndex,
        rhs: StackIndex,
    },

    #[strum(message = "Copies value from one index to another")]
    Copy {
        source: StackIndex,
        output: StackIndex,
    },

    #[strum(message = "Prints a value at a register (0)")]
    Print(StackIndex),
}

impl OpCode {
    pub fn run(&self, vm: &mut Vm) -> Result<(), MsError> {
        match self {
            OpCode::Return(_) => Err(InternalError::NotImplemented.into()),
            OpCode::SetNumber(to, number) => {
                vm[to] = Value::Number(*number);
                Ok(())
            }
            OpCode::SetNull(to) => {
                vm[to] = Value::Null;
                Ok(())
            }
            OpCode::SetString(_, _) => todo!(),
            OpCode::ReadVariable(_, _) => todo!(),
            OpCode::Call0 { output, function } => {
                let function = &vm[function];
                match function {
                    Value::Null | Value::Number(_) => vm[output] = function.clone(),
                }
                Ok(())
            }
            OpCode::Call1 { .. } => todo!(),
            OpCode::Call { .. } => todo!(),
            OpCode::Add { lhs, rhs, output } => {
                vm[output] = Value::from(vm[lhs].as_f64() + vm[rhs].as_f64());
                Ok(())
            }
            OpCode::Subtract { lhs, rhs, output } => {
                vm[output] = Value::from(vm[lhs].as_f64() - vm[rhs].as_f64());
                Ok(())
            }
            OpCode::Multiply { lhs, rhs, output } => {
                vm[output] = Value::from(vm[lhs].as_f64() * vm[rhs].as_f64());
                Ok(())
            }
            OpCode::Divide { lhs, rhs, output } => {
                vm[output] = Value::from(vm[lhs].as_f64() / vm[rhs].as_f64());
                Ok(())
            }
            OpCode::Pow { lhs, rhs, output } => {
                let lhs = vm[lhs].as_f64();
                let rhs = vm[rhs].as_f64();
                let mut result = 0.;
                cfg_if::cfg_if! {
                    if #[cfg(feature = "libm")] {
                        result = libm::pow(lhs, rhs);
                    } else {
                        result = lhs.powf(rhs);
                    }
                }
                vm[output] = Value::from(result);
                Ok(())
            }
            OpCode::Copy { source, output } => {
                vm[output] = vm[source].clone();
                Ok(())
            }
            OpCode::Print(at) => {
                let value = &vm[at];
                dbg!(value);
                Ok(())
            }
        }
    }
}
