use serde_json::Value;

use things::thing::Thing;

use crate::vm::operation_result::OperationResult;
use crate::vm::state::VmState;
use crate::vm::VmErrors;

#[derive(Debug, Clone)]
pub enum ValueSource {
    Const(Value),
    Variable(String),
}

impl ValueSource {
    pub fn get_value(&self, state: &VmState) -> anyhow::Result<Value> {
        match self {
            ValueSource::Const(value) => Ok(value.clone()),
            ValueSource::Variable(name) => state.get_value(name).map_or_else(
                || Err(VmErrors::VariableNotFound(name.clone()).into()),
                |x| Ok(x.clone()),
            ),
        }
    }
}

#[derive(Debug, Clone)]
pub enum ValueTarget {
    Ignored,
    Variable(String),
    NewVariable(String),
}

impl ValueTarget {
    pub fn set_value(&self, state: &mut VmState, value: Value, thing: &Thing) -> OperationResult {
        match self {
            ValueTarget::Ignored => OperationResult::Ok,
            ValueTarget::Variable(name) => state.set_or_else(name, value, |state, value| {
                state.new_variable(name, value, thing.clone());
                OperationResult::Ok
            }),
            ValueTarget::NewVariable(name) => {
                state.new_variable(name, value, thing.clone());
                OperationResult::Ok
            }
        }
    }
}

pub struct TypedVariable<'a> {
    name: &'a str,
    thing: &'a Thing,
}

impl<'a> TypedVariable<'a> {
    pub fn new(name: &'a str, thing: &'a Thing) -> Self {
        TypedVariable { name, thing }
    }

    pub fn get_name(&self) -> &'a str {
        self.name
    }

    pub fn get_type(&self) -> &'a Thing {
        self.thing
    }
}
