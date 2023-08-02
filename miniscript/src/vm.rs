use crate::vm::chunk::Chunk;
use crate::vm::register::StackIndex;
use crate::{errors::MsError, value::Value};
use std::ops::{Index, IndexMut};

pub mod op_code;

pub mod chunk;
pub mod register;

pub struct Vm {
    pub cursor: usize,
    pub stack: Vec<Value>,
}

impl Vm {
    #[inline(always)]
    pub fn stack_offset(&self) -> usize {
        0
    }
}

impl Index<&StackIndex> for Vm {
    type Output = Value;

    fn index(&self, index: &StackIndex) -> &Self::Output {
        let offset = self.stack_offset();
        &(self.stack)[&index.with_offset(offset)]
    }
}

impl IndexMut<&StackIndex> for Vm {
    fn index_mut(&mut self, index: &StackIndex) -> &mut Self::Output {
        let offset = self.stack_offset();
        &mut (self.stack)[&index.with_offset(offset)]
    }
}

pub trait VmRunner {
    fn run(&mut self, chunk: &Chunk, vm: &mut Vm) -> Result<(), MsError>;
}

pub struct DefaultRunner;

impl VmRunner for DefaultRunner {
    fn run(&mut self, chunk: &Chunk, vm: &mut Vm) -> Result<(), MsError> {
        while vm.cursor < chunk.code().len() {
            let op_code = &chunk.code()[vm.cursor];
            op_code.run(vm)?;
            vm.cursor += 1;
        }

        Ok(())
    }
}
