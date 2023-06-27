use crate::{errors::MsError, value::Value};

pub mod op_code;
use self::op_code::OpCode;
pub mod register;

pub struct Chunk {
    code: Vec<OpCode>,
    constants: Vec<Value>,
}

pub struct Vm {
    pub chunk: Chunk,
    pub cursor: usize,
    pub stack: Vec<Value>,
}

pub trait VmRunner {
    fn run(&mut self, vm: &mut Vm) -> Result<(), MsError>;
}

pub struct DefaultRunner;

impl VmRunner for DefaultRunner {
    fn run(&mut self, vm: &mut Vm) -> Result<(), MsError> {
        while vm.cursor < vm.chunk.code.len() {
            let op_code = vm.chunk.code[vm.cursor];
            op_code.run(vm)?;
        }

        Ok(())
    }
}
