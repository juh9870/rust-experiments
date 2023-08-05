use crate::value::Value;
use std::fmt::{Display, Formatter};
use std::ops::{Index, IndexMut, Sub};

type Stack = Vec<Value>;

#[derive(Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd)]
pub struct StackIndex(pub(crate) usize);

impl StackIndex {
    pub fn with_offset(&self, offset: usize) -> Register {
        return Register(self.0 + offset);
    }
}

impl Display for StackIndex {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Register(usize);

// #[inline(always)]
// fn get_index(stack: &Vec<Value>, index: &Register) -> usize {
//     stack.len().sub(index.0 as usize)
// }

impl Index<&Register> for Vec<Value> {
    type Output = Value;

    #[inline(always)]
    fn index(&self, index: &Register) -> &Self::Output {
        &self[index.0]
    }
}

impl IndexMut<&Register> for Vec<Value> {
    #[inline(always)]
    fn index_mut(&mut self, index: &Register) -> &mut Self::Output {
        let index = index.0;
        &mut self[index]
    }
}
