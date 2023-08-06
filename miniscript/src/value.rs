use auto_ops::impl_op_ex;
use std::fmt::Display;
use std::ops::Neg;

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
        impl From<&Value> for $type {
            fn from(value: &Value) -> Self {
                value.as_f64() as $type
            }
        }
        impl From<$type> for Value {
            fn from(value: $type) -> Self {
                Value::Number(value as f64)
            }
        }
    };
}

#[inline(always)]
fn numeric_op<A: Fn(&f64, &f64) -> T, T: Into<Value>>(a: &Value, b: &Value, op: A) -> Value {
    match (a, b) {
        (Value::Number(a), Value::Number(b)) => op(a, b).into(),
        (Value::Number(a), Value::Null) => op(a, &0.).into(),
        _ => Value::Null,
    }
}

fn abs_clamp_01(mut num: f64) -> f64 {
    if num < 0. {
        num = -num;
    }
    if num > 1. {
        return 1.;
    }
    num
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
    numeric_as!(as_usize, as_usize_checked, usize);

    numeric_as!(as_i8, as_i8_checked, i8);
    numeric_as!(as_i16, as_i16_checked, i16);
    numeric_as!(as_i32, as_i32_checked, i32);
    numeric_as!(as_i64, as_i64_checsked, i64);
    numeric_as!(as_i128, as_i128_checked, i128);

    pub fn as_bool(&self) -> bool {
        match self {
            Value::Null => false,
            Value::Number(num) => *num > 0.,
        }
    }

    pub fn as_probability_number(&self) -> f64 {
        match self {
            Value::Number(num) => *num,
            val => val.as_bool() as usize as f64,
        }
    }

    pub fn lt(&self, other: &Value) -> Value {
        numeric_op(self, other, |a, b| a < b)
    }

    pub fn lte(&self, other: &Value) -> Value {
        numeric_op(self, other, |a, b| a <= b)
    }

    pub fn gt(&self, other: &Value) -> Value {
        numeric_op(self, other, |a, b| a > b)
    }

    pub fn gte(&self, other: &Value) -> Value {
        numeric_op(self, other, |a, b| a >= b)
    }

    pub fn pow(&self, other: &Value) -> Value {
        numeric_op(self, other, |a, b| {
            cfg_if::cfg_if! {
                if #[cfg(feature = "libm")] {
                    return libm::pow(*a, *b);
                } else {
                    return a.powf(*b);
                }
            }
        })
    }

    pub fn fuzzy_or(&self, other: &Value) -> Value {
        numeric_op(self, other, |a, b| abs_clamp_01(a + b - a * b))
    }

    pub fn fuzzy_and(&self, other: &Value) -> Value {
        numeric_op(self, other, |a, b| abs_clamp_01(a * b))
    }

    pub fn or(&self, other: &Value) -> Value {
        Value::from(self.as_bool() || other.as_bool())
    }

    pub fn and(&self, other: &Value) -> Value {
        Value::from(self.as_bool() && other.as_bool())
    }
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

impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Value::Null, Value::Null) => false,
            (Value::Number(a), Value::Number(b)) => a == b,
            _ => false,
        }
    }
}

impl Eq for Value {}

impl_op_ex!(+|a: &Value, b: &Value| -> Value { numeric_op(a, b, |a, b| a + b) });

impl_op_ex!(-|a: &Value, b: &Value| -> Value { numeric_op(a, b, |a, b| a - b) });

impl_op_ex!(*|a: &Value, b: &Value| -> Value { numeric_op(a, b, |a, b| a * b) });

impl_op_ex!(/|a: &Value, b: &Value| -> Value { numeric_op(a, b, |a, b| a / b) });

impl Neg for Value {
    type Output = Value;

    fn neg(self) -> Self::Output {
        match self {
            Value::Null => Value::Number(-0.),
            Value::Number(num) => Value::Number(-num),
        }
    }
}

impl From<bool> for Value {
    fn from(value: bool) -> Self {
        Self::from(value as usize)
    }
}

numeric_from!(f32);
numeric_from!(f64);

numeric_from!(u8);
numeric_from!(u16);
numeric_from!(u32);
numeric_from!(u64);
numeric_from!(u128);
numeric_from!(usize);

numeric_from!(i8);
numeric_from!(i16);
numeric_from!(i32);
numeric_from!(i64);
numeric_from!(i128);
