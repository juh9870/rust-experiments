use num_traits::Num;
use std::fmt::Display;

use crate::errors::MsError;

#[non_exhaustive]
#[derive(Debug, Clone)]
pub enum Value {
    Null,
    Number(f64),
}

macro_rules! numeric_as {
    ($name:ident, $checked:ident, $type:ty) => {
        pub fn $checked(&self) -> Option<$type> {
            self.as_f64_checked()
                .map(|e| e as $type)
                .filter(|e| (<$type>::MIN..<$type>::MAX).contains(e))
        }

        pub fn $name(&self) -> $type {
            self.as_f64() as $type
        }
    };
}

macro_rules! numeric_from {
    ($type:ty) => {
        // impl From<Value> for $type {
        //     fn from(value: Value) -> Self {
        //         value.as_f64() as $type
        //     }
        // }

        impl From<&Value> for $type {
            fn from(value: &Value) -> Self {
                value.as_f64() as $type
            }
        }
    };
}

impl Value {
    pub fn as_f64(&self) -> f64 {
        match self {
            Value::Number(val) => *val,
            _ => 0.,
        }
    }

    pub fn as_f64_checked(&self) -> Option<f64> {
        match self {
            Value::Number(val) => Some(*val),
            _ => None,
        }
    }

    numeric_as!(as_f32, as_f32_checked, f32);

    numeric_as!(as_u8, as_u8_checked, u8);
    numeric_as!(as_u16, as_u16_checked, u16);
    numeric_as!(as_u32, as_u32_checked, u32);
    numeric_as!(as_u64, as_u64_checked, u64);
    numeric_as!(as_u128, as_u128_checked, u128);

    numeric_as!(as_i8, as_i8_checked, i8);
    numeric_as!(as_i16, as_i16_checked, i16);
    numeric_as!(as_i32, as_i32_checked, i32);
    numeric_as!(as_i64, as_i64_checsked, i64);
    numeric_as!(as_i128, as_i128_checked, i128);
}

impl Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::Null => write!(f, "null"),
            Value::Number(val) => {
                write!(f, "{val}")
            }
        }
    }
}

impl<T: Into<f64> + Num> From<T> for Value {
    fn from(value: T) -> Self {
        Value::Number(value.into())
    }
}

numeric_from!(f32);
numeric_from!(f64);

numeric_from!(u8);
numeric_from!(u16);
numeric_from!(u32);
numeric_from!(u64);
numeric_from!(u128);

numeric_from!(i8);
numeric_from!(i16);
numeric_from!(i32);
numeric_from!(i64);
numeric_from!(i128);
