use crate::errors::{InternalError, MsError, MsErrorType, RuntimeError};
use crate::value::Value;
use crate::vm::chunk::ConstantIndex;
use std::fmt::format;
use std::result;
use strum_macros::EnumMessage;

use super::{register::StackIndex, Vm};

///
/// As a rule of thumb, first argument is the "target" of a bytecode operation
#[non_exhaustive]
#[derive(EnumMessage, Debug, Clone)]
pub enum OpCode {
    #[strum(message = "Returns a value at a register (0)")]
    Return(Option<StackIndex>),

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
    #[strum(message = "")]
    FuzzyOr {
        output: StackIndex,
        lhs: StackIndex,
        rhs: StackIndex,
    },
    #[strum(message = "")]
    FuzzyAnd {
        output: StackIndex,
        lhs: StackIndex,
        rhs: StackIndex,
    },
    #[strum(message = "")]
    And {
        output: StackIndex,
        lhs: StackIndex,
        rhs: StackIndex,
    },
    #[strum(message = "")]
    Equals {
        output: StackIndex,
        lhs: StackIndex,
        rhs: StackIndex,
    },
    #[strum(message = "")]
    NotEquals {
        output: StackIndex,
        lhs: StackIndex,
        rhs: StackIndex,
    },
    #[strum(message = "")]
    GreaterThan {
        output: StackIndex,
        lhs: StackIndex,
        rhs: StackIndex,
    },
    #[strum(message = "")]
    LessThan {
        output: StackIndex,
        lhs: StackIndex,
        rhs: StackIndex,
    },
    #[strum(message = "")]
    GreaterOrEquals {
        output: StackIndex,
        lhs: StackIndex,
        rhs: StackIndex,
    },
    #[strum(message = "")]
    LessOrEquals {
        output: StackIndex,
        lhs: StackIndex,
        rhs: StackIndex,
    },

    // Control flow
    JumpIfFalse(StackIndex, usize),
    JumpIfTrue(StackIndex, usize),
    JumpIfAbsOneOrGreater(StackIndex, usize),
    Jump(usize),

    #[strum(message = "Copies value from one index to another")]
    Copy {
        source: StackIndex,
        output: StackIndex,
    },

    #[strum(message = "Prints a value at a register (0)")]
    Print {
        value: StackIndex,
        output: StackIndex,
    },

    #[strum(message = "Raises a runtime error with a provided message")]
    Error(BytecodeError),
}

#[inline(always)]
fn simple_op<T: Fn(&Value, &Value) -> Value>(
    vm: &mut Vm,
    lhs: &StackIndex,
    rhs: &StackIndex,
    output: &StackIndex,
    op: T,
) -> Result<(), MsErrorType> {
    vm[output] = op(&vm[lhs], &vm[rhs]);
    Ok(())
}

impl OpCode {
    pub fn step(&self, vm: &mut Vm) -> Result<(), MsErrorType> {
        vm.cursor += 1;
        match self {
            OpCode::Return(value) => {
                if value.is_none() {
                    return Ok(());
                }
                Err(InternalError::NotImplemented.into())
            }
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
            OpCode::Add { lhs, rhs, output } => simple_op(vm, lhs, rhs, output, |a, b| a + b),
            OpCode::Subtract { lhs, rhs, output } => simple_op(vm, lhs, rhs, output, |a, b| a - b),
            OpCode::Multiply { lhs, rhs, output } => simple_op(vm, lhs, rhs, output, |a, b| a * b),
            OpCode::Divide { lhs, rhs, output } => simple_op(vm, lhs, rhs, output, |a, b| a / b),
            OpCode::Pow { lhs, rhs, output } => simple_op(vm, lhs, rhs, output, |a, b| a.pow(b)),
            OpCode::FuzzyOr { lhs, rhs, output } => {
                simple_op(vm, lhs, rhs, output, |a, b| a.fuzzy_or(b))
            }
            OpCode::FuzzyAnd { lhs, rhs, output } => {
                simple_op(vm, lhs, rhs, output, |a, b| a.fuzzy_and(b))
            }
            OpCode::And { lhs, rhs, output } => simple_op(vm, lhs, rhs, output, |a, b| a.and(b)),
            OpCode::Equals { lhs, rhs, output } => {
                simple_op(vm, lhs, rhs, output, |a, b| Value::from(a == b))
            }
            OpCode::NotEquals { lhs, rhs, output } => {
                simple_op(vm, lhs, rhs, output, |a, b| Value::from(a != b))
            }
            OpCode::GreaterThan { lhs, rhs, output } => {
                simple_op(vm, lhs, rhs, output, |a, b| a.gt(b))
            }
            OpCode::LessThan { lhs, rhs, output } => {
                simple_op(vm, lhs, rhs, output, |a, b| a.lt(b))
            }
            OpCode::GreaterOrEquals { lhs, rhs, output } => {
                simple_op(vm, lhs, rhs, output, |a, b| a.gte(b))
            }
            OpCode::LessOrEquals { lhs, rhs, output } => {
                simple_op(vm, lhs, rhs, output, |a, b| a.lte(b))
            }
            OpCode::Copy { source, output } => {
                vm[output] = vm[source].clone();
                Ok(())
            }
            OpCode::JumpIfFalse(condition, target) => {
                if !vm[condition].as_bool() {
                    vm.cursor = *target
                }
                Ok(())
            }
            OpCode::JumpIfTrue(condition, target) => {
                if vm[condition].as_bool() {
                    vm.cursor = *target
                }
                Ok(())
            }
            OpCode::JumpIfAbsOneOrGreater(condition, target) => {
                if vm[condition].as_f64() >= 1. {
                    vm.cursor = *target
                }
                Ok(())
            }
            OpCode::Jump(target) => {
                vm.cursor = *target;
                Ok(())
            }
            OpCode::Print { output, value } => {
                let value = &vm[value];
                println!("{:?}", value);
                vm[output] = Value::Null;
                Ok(())
            }
            OpCode::Error(err) => Err(err.error(vm.cursor - 1, vm)),
        }
    }

    pub fn pretty_print(&self) -> String {
        match self {
            OpCode::Return(index) => match index {
                None => "return".to_string(),
                Some(index) => format!("return ${index}"),
            },
            OpCode::SetNumber(to, num) => format!("${to} = {num}"),
            OpCode::SetNull(to) => format!("${to} = null"),
            OpCode::SetString(to, idx) => format!("${to} = \"{}\"", idx.raw()),
            OpCode::ReadVariable(to, ident) => format!("${to} = {ident}"),
            OpCode::Call0 { output, function } => format!("${output} = ${function}()"),
            OpCode::Call1 {
                output,
                function,
                arg,
            } => format!("{output} = ${function}( ${arg} )"),
            OpCode::Call { .. } => todo!(),
            OpCode::Add { output, lhs, rhs } => format!("${output} = ${lhs} + ${rhs}"),
            OpCode::Subtract { output, lhs, rhs } => format!("${output} = ${lhs} - ${rhs}"),
            OpCode::Multiply { output, lhs, rhs } => format!("${output} = ${lhs} * ${rhs}"),
            OpCode::Divide { output, lhs, rhs } => format!("${output} = ${lhs} / ${rhs}"),
            OpCode::Pow { output, lhs, rhs } => format!("${output} = ${lhs} ^ ${rhs}"),
            OpCode::FuzzyOr { output, lhs, rhs } => {
                format!("${output} = ${lhs} fuzzy_or ${rhs}")
            }
            OpCode::FuzzyAnd { output, lhs, rhs } => {
                format!("${output} = ${lhs} fuzzy_and ${rhs}")
            }
            OpCode::And { output, lhs, rhs } => {
                format!("${output} = ${lhs} and ${rhs}")
            }
            OpCode::Equals { output, lhs, rhs } => {
                format!("${output} = ${lhs} == ${rhs}")
            }
            OpCode::NotEquals { output, lhs, rhs } => {
                format!("${output} = ${lhs} != ${rhs}")
            }
            OpCode::GreaterThan { output, lhs, rhs } => {
                format!("${output} = ${lhs} > ${rhs}")
            }
            OpCode::LessThan { output, lhs, rhs } => {
                format!("${output} = ${lhs} < ${rhs}")
            }
            OpCode::GreaterOrEquals { output, lhs, rhs } => {
                format!("${output} = ${lhs} >= ${rhs}")
            }
            OpCode::LessOrEquals { output, lhs, rhs } => {
                format!("${output} = ${lhs} <= ${rhs}")
            }
            OpCode::Copy { source, output } => format!("${output} = ${source}"),
            OpCode::JumpIfFalse(condition, target) => format!("if not ${condition} goto {target}"),
            OpCode::JumpIfTrue(condition, target) => format!("if ${condition} goto {target}"),
            OpCode::JumpIfAbsOneOrGreater(condition, target) => {
                format!("if abs(${condition}) >= 1 goto {target}")
            }
            OpCode::Jump(target) => format!("goto {target}"),
            OpCode::Print { output, value } => format!("${output} = print ${value}"),
            OpCode::Error(err) => err.pretty_print(),
        }
    }
}

#[derive(Debug, Clone)]
pub enum BytecodeError {
    Message(String),
    Register(StackIndex),
    UnpatchedOpCode,
}

impl BytecodeError {
    fn error(&self, id: usize, vm: &Vm) -> MsErrorType {
        match self {
            BytecodeError::Message(msg) => todo!(),
            BytecodeError::Register(idx) => RuntimeError::Custom(vm[idx].clone()).into(),
            BytecodeError::UnpatchedOpCode => InternalError::UnpatchedOpCode(id).into(),
        }
    }

    fn pretty_print(&self) -> String {
        match self {
            BytecodeError::Message(msg) => format!("throw \"{msg}\""),
            BytecodeError::Register(target) => format!("throw ${target}"),
            BytecodeError::UnpatchedOpCode => format!("FATAL! unpatched OpCode"),
        }
    }
}
