use crate::{errors::MsError, value::Value};

#[derive(Debug, Clone, Copy)]
pub struct StackOffset(u32);

pub trait VmStackExtension {
    fn get(&self, index: StackOffset) -> Result<&Value, MsError>;
    fn get_mut(&mut self, index: StackOffset) -> Result<&mut Value, MsError>;
    fn set(&mut self, index: StackOffset, value: Value) -> Result<(), MsError>;
}

#[inline(always)]
fn get_index(stack: &Vec<Value>, index: StackOffset) -> Result<usize, MsError> {
    stack
        .len()
        .checked_sub(index.0 as usize)
        .ok_or_else(|| MsError::BadRegister(index, stack.len()))
}

impl VmStackExtension for Vec<Value> {
    #[inline(always)]
    fn get(&self, index: StackOffset) -> Result<&Value, MsError> {
        Ok(&self[get_index(self, index)?])
    }

    #[inline(always)]
    fn get_mut(&mut self, index: StackOffset) -> Result<&mut Value, MsError> {
        let idx = get_index(self, index)?;
        Ok(&mut self[idx])
    }

    #[inline(always)]
    fn set(&mut self, index: StackOffset, value: Value) -> Result<(), MsError> {
        let idx = get_index(self, index)?;
        self[idx] = value;
        Ok(())
    }
}
