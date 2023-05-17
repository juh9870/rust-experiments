use std::fmt::{Display, Formatter};

use as_any::Downcast;
use serde_json::Value;
use strum::IntoEnumIterator;
use strum_macros::EnumIter;
use thiserror::Error;

use crate::thing::{Refinement, Relation};

#[derive(Error, Debug)]
pub struct TypeRefinementError {
    pub expected: PrimitiveRefinement,
    pub got: ValueType,
}

#[derive(Error, Debug)]
#[error(".0")]
pub struct InapplicableRefinementError(String);

impl Display for TypeRefinementError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "Got {} where {} was expected", self.got, self.expected)
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum ValueType {
    Null,
    Bool,
    Number,
    String,
    Array,
    Object,
}

impl Display for ValueType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                ValueType::Null => "Null",
                ValueType::Bool => "Boolean",
                ValueType::Number => "Number",
                ValueType::String => "String",
                ValueType::Array => "Array",
                ValueType::Object => "Object",
            }
        )
    }
}

fn value_type(val: &Value) -> ValueType {
    match val {
        Value::Null => ValueType::Null,
        Value::Bool(_) => ValueType::Bool,
        Value::Number(_) => ValueType::Number,
        Value::String(_) => ValueType::String,
        Value::Array(_) => ValueType::Array,
        Value::Object(_) => ValueType::Object,
    }
}

pub fn inapplicable(refinement: &dyn Refinement, value: &Value) -> anyhow::Error {
    InapplicableRefinementError(format!(
        "Refinement {} can not be applied to value of type {}",
        refinement,
        value_type(value)
    ))
    .into()
}

#[derive(Error, Debug)]
#[error("{got} is not a subset of {expected}")]
pub struct RefinementCastError<T: Display + Refinement> {
    pub expected: T,
    pub got: T,
}

#[derive(EnumIter, Debug, Copy, Clone, PartialEq, Eq)]
#[cfg_attr(test, derive(test_strategy::Arbitrary))]
pub enum PrimitiveRefinement {
    Number,
    String,
    Bool,
    Array,
    Object,
    Null,
}

impl Display for PrimitiveRefinement {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let str = match self {
            PrimitiveRefinement::Number => "Number",
            PrimitiveRefinement::String => "String",
            PrimitiveRefinement::Bool => "Boolean",
            PrimitiveRefinement::Array => "Array",
            PrimitiveRefinement::Object => "Object",
            PrimitiveRefinement::Null => "Null",
        };
        write!(f, "{}", str)
    }
}

impl Refinement for PrimitiveRefinement {
    fn apply(&self, value: &mut Value, _warnings: &mut Vec<anyhow::Error>) -> anyhow::Result<()> {
        let valid = match self {
            PrimitiveRefinement::Number => value.is_number(),
            PrimitiveRefinement::String => value.is_string(),
            PrimitiveRefinement::Bool => value.is_boolean(),
            PrimitiveRefinement::Array => value.is_array(),
            PrimitiveRefinement::Object => value.is_object(),
            PrimitiveRefinement::Null => value.is_null(),
        };
        if valid {
            Ok(())
        } else {
            Err(anyhow::Error::new(TypeRefinementError {
                got: value_type(value),
                expected: *self,
            }))
        }
    }

    fn strict(&self) -> Box<dyn Refinement> {
        Box::new(*self)
    }

    fn clone(&self) -> Box<dyn Refinement> {
        Box::new(Clone::clone(self))
    }

    fn is_subset_of(&self, other: &dyn Refinement) -> Relation {
        let Some(other) = other.downcast_ref::<PrimitiveRefinement>() else { return Relation::Unrelated; };
        if other == self {
            return Relation::Subset;
        }
        Relation::Conflict(
            RefinementCastError {
                got: *self,
                expected: *other,
            }
            .into(),
        )
    }
}

pub fn validate_number(val: &Value) -> anyhow::Result<f64> {
    if let Some(num) = val.as_f64() {
        Ok(num)
    } else {
        Err(TypeRefinementError {
            expected: PrimitiveRefinement::Number,
            got: value_type(val),
        }
        .into())
    }
}

#[cfg(test)]
pub mod tests {
    use test_strategy::proptest;

    use crate::thing::primitives::{PrimitiveRefinement, RefinementCastError, TypeRefinementError};
    use crate::thing::tests_utils::{
        prop_unwrap, TestAssertionResult, TestConversionResult, TestRefinement,
    };
    use crate::thing::{Refinement, Thing};

    static PRIMITIVES: [PrimitiveRefinement; 6] = [
        PrimitiveRefinement::Number,
        PrimitiveRefinement::String,
        PrimitiveRefinement::Bool,
        PrimitiveRefinement::Array,
        PrimitiveRefinement::Object,
        PrimitiveRefinement::Null,
    ];

    fn other_primitives(cur: PrimitiveRefinement) -> impl Iterator<Item = PrimitiveRefinement> {
        PRIMITIVES.iter().copied().filter(move |r| *r != cur)
    }

    #[test]
    fn numeric_nan() {
        PrimitiveRefinement::Number
            .check(f64::NAN)
            .error::<TypeRefinementError>()
            .assert();
        PrimitiveRefinement::Null.check(f64::NAN).success().assert();
    }

    #[test]
    fn numeric_infinite() {
        PrimitiveRefinement::Number
            .check(f64::INFINITY)
            .error::<TypeRefinementError>()
            .assert();
        PrimitiveRefinement::Null
            .check(f64::INFINITY)
            .success()
            .assert();

        PrimitiveRefinement::Number
            .check(f64::NEG_INFINITY)
            .error::<TypeRefinementError>()
            .assert();
        PrimitiveRefinement::Null
            .check(f64::NEG_INFINITY)
            .success()
            .assert();
    }

    #[test]
    fn boolean() {
        PrimitiveRefinement::Bool.check(true).success().assert();
        PrimitiveRefinement::Bool.check(false).success().assert();
        other_primitives(PrimitiveRefinement::Bool)
            .all(|p| p.check(true).is_err() && p.check(false).is_err());
    }

    #[proptest]
    fn numeric_prop(n: f64) {
        prop_unwrap!(PrimitiveRefinement::Number.check(n).success());
        other_primitives(PrimitiveRefinement::Number).all(|p| p.check(n).is_err());
    }

    #[proptest]
    fn string_prop(n: String) {
        prop_unwrap!(PrimitiveRefinement::String.check(n.clone()).success());
        other_primitives(PrimitiveRefinement::String).all(|p| p.check(n.clone()).is_err());
    }

    #[proptest]
    fn cast(refinement: PrimitiveRefinement) {
        for other in other_primitives(refinement) {
            prop_unwrap!(Thing::from(refinement)
                .try_assign_to(&Thing::from(other))
                .error::<RefinementCastError<PrimitiveRefinement>>());
            prop_unwrap!(Thing::from(refinement)
                .try_assign_to(&Thing::from(refinement))
                .success());
        }
    }
}
