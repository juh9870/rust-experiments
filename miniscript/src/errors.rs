use crate::{value::Value, vm::register::StackOffset};
use strum_macros::{EnumDiscriminants, EnumIter};
use thiserror::Error;

#[derive(Debug, Error, EnumDiscriminants)]
#[strum_discriminants(derive(EnumIter))]
#[non_exhaustive]
pub enum MsError {
    #[error("Operation not implemented")]
    NotImplemented,
    #[error("Stack offset {0:?} was accessed on a stack of length {1}")]
    BadRegister(StackOffset, usize),
}

impl MsError {
    pub fn code(&self) -> u16 {
        MsErrorDiscriminants::from(self).code()
    }
}
impl MsErrorDiscriminants {
    pub const INTERNAL: u16 = 0;
    pub const RUNTIME: u16 = 1000;
    pub const COMPILE: u16 = 2000;
    pub fn code(&self) -> u16 {
        match self {
            MsErrorDiscriminants::NotImplemented => Self::INTERNAL + 0,
            MsErrorDiscriminants::BadRegister => Self::INTERNAL + 1,
        }
    }
}
