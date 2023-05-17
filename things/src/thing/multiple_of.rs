use std::fmt::{Display, Formatter};

use anyhow::Error;
use as_any::Downcast;
use serde_json::Value;
use thiserror::Error;

use crate::thing::primitives::{validate_number, RefinementCastError};
use crate::thing::{Refinement, Relation};

pub static INTEGER: MultipleOfRefinement = MultipleOfRefinement {
    factor: 1f64,
    coerce: true,
};
pub static STRICT_INTEGER: MultipleOfRefinement = MultipleOfRefinement {
    factor: 1f64,
    coerce: false,
};

#[derive(Debug, Clone, Copy)]
pub struct MultipleOfRefinement {
    pub factor: f64,
    pub coerce: bool,
}

impl Display for MultipleOfRefinement {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        if self.factor == 1f64 {
            write!(f, "Integer")
        } else {
            write!(f, "MultipleOf({})", self.factor)
        }
    }
}

impl Refinement for MultipleOfRefinement {
    fn apply(&self, value: &mut Value, warnings: &mut Vec<Error>) -> anyhow::Result<()> {
        let num = validate_number(value)?;
        if num % self.factor == 0.0 {
            return Ok(());
        };

        let err = BoundsRefinementError {
            value: num,
            factor: self.factor,
        }
        .into();
        if self.coerce {
            *value = Value::from((num / self.factor).round() * self.factor);
            warnings.push(err);
            Ok(())
        } else {
            Err(err)
        }
    }

    fn strict(&self) -> Box<dyn Refinement> {
        if self.coerce {
            Box::new(*self)
        } else {
            let mut copy = *self;
            copy.coerce = true;
            Box::new(copy)
        }
    }

    fn is_subset_of(&self, other: &dyn Refinement) -> Relation {
        let Some(other) = other.downcast_ref::<MultipleOfRefinement>() else { return Relation::Unrelated; };
        let err = (self.factor % other.factor).abs();
        let inv_err = (other.factor.abs() - err).abs();
        // This margin is enough to handle approximately i32 factor of
        // difference between values, which I consider to be a good
        // stopping point
        let margin = (other.factor * 1e-6).abs();
        if err < margin || inv_err < margin || other.coerce {
            return Relation::Subset;
        }
        Relation::Conflict(
            RefinementCastError {
                expected: *other,
                got: *self,
            }
            .into(),
        )
    }

    fn clone(&self) -> Box<dyn Refinement> {
        Box::new(Clone::clone(self))
    }
}

#[derive(Error, Debug)]
pub struct BoundsRefinementError {
    value: f64,
    factor: f64,
}

impl Display for BoundsRefinementError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        if self.factor == 1f64 {
            write!(f, "{} is not an integer", self.value)
        } else {
            write!(f, "{} is not a multiple of {}", self.value, self.factor)
        }
    }
}

#[cfg(test)]
pub mod tests {
    use test_strategy::proptest;

    use crate::thing::multiple_of::{
        BoundsRefinementError, MultipleOfRefinement, INTEGER, STRICT_INTEGER,
    };
    use crate::thing::tests_utils::{
        do_assert, prop_assert_type, prop_unwrap, TestAssertionResult, TestConversionResult,
        TestConversionResultWithWarnings, TestRefinement,
    };
    use crate::thing::Thing;

    fn check_coercion(n: f64, base: f64, margin: f64) -> anyhow::Result<()> {
        let refinement = MultipleOfRefinement {
            factor: base,
            coerce: true,
        };
        let strict = MultipleOfRefinement {
            factor: base,
            coerce: false,
        };

        if n % base == 0.0 {
            refinement.check(n).with_warnings(0)?;
        } else {
            let coerced = (refinement.check(n).with_warnings(1))?.0;

            let num = prop_assert_type!(coerced, Number)?;
            let remainder = (num % base).abs();
            let base_error = (remainder - base.abs()).abs();

            do_assert(
                remainder <= margin || base_error <= margin,
                "Coercion failed",
            )?;
            strict.check(n).error::<BoundsRefinementError>()?;
        }
        Ok(())
    }

    #[proptest]
    fn valid_integers(n: i64) {
        prop_unwrap!(INTEGER.check(n).success());
        prop_unwrap!(STRICT_INTEGER.check(n).success());
    }

    #[proptest]
    fn coerce_integers(#[filter(# n % 1.0 != 0.0)] n: f64) {
        prop_unwrap!(check_coercion(n, 1.0, 0.0));
        prop_unwrap!(STRICT_INTEGER.check(n).error::<BoundsRefinementError>());
    }

    // Only explicitly support "sane" values, float precision tends to go crazy at higher values
    #[proptest]
    fn coerce(
        #[map(| base: i64 | (base as f64) / 100.0)] n: f64,
        #[map(| base: i64 | (base as f64) / 100.0)]
        #[filter(# base != 0.0)]
        base: f64,
    ) {
        prop_unwrap!(check_coercion(n, base, (base * 1e-10).abs()))
    }

    #[proptest]
    fn cast_to_self(#[filter(# factor != 0.0)] factor: f64) {
        let refinement = MultipleOfRefinement {
            factor,
            coerce: false,
        };

        prop_unwrap!(Thing::from(refinement)
            .try_assign_to(&Thing::from(refinement))
            .success());
    }

    #[proptest]
    fn cast_to_compatible(
        #[map(| base: i32 | (base as f64) / 100.0)]
        #[filter(# factor != 0.0)]
        factor: f64,
        #[map(| base: i32 | base as f64)]
        #[filter(( # factor * # mult ).is_finite() && # mult != 0.0)]
        mult: f64,
    ) {
        let refinement = MultipleOfRefinement {
            factor,
            coerce: false,
        };
        let narrow = MultipleOfRefinement {
            factor: factor * mult,
            coerce: false,
        };

        prop_unwrap!(Thing::from(narrow)
            .try_assign_to(&Thing::from(refinement))
            .success());
    }

    #[proptest]
    fn cast_to_coerce(
        #[filter(# factor != 0.0)] factor: f64,
        #[filter(# other != 0.0)] other: f64,
    ) {
        let refinement = MultipleOfRefinement {
            factor,
            coerce: false,
        };

        let other = MultipleOfRefinement {
            factor,
            coerce: true,
        };

        prop_unwrap!(Thing::from(refinement)
            .try_assign_to(&Thing::from(other))
            .success());
    }
}
