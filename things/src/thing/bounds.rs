use std::fmt::{Display, Formatter};
use std::ops::{Bound, RangeBounds};

use as_any::Downcast;
use serde_json::Value;
use thiserror::Error;

use crate::thing::primitives::{inapplicable, validate_number, RefinementCastError};
use crate::thing::{Refinement, Relation};

#[derive(Debug, Clone, Copy)]
pub struct BoundsRefinement {
    pub min: Bound<f64>,
    pub max: Bound<f64>,
    pub coerce: bool,
}

impl BoundsRefinement {
    fn clamp(&self, num: f64) -> f64 {
        match (get_value(&self.min), get_value(&self.max)) {
            (Some(min), Some(max)) => num.clamp(min, max),
            (Some(min), None) => num.max(min),
            (None, Some(max)) => num.min(max),
            (None, None) => num,
        }
    }
}

impl Display for BoundsRefinement {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Bounds {}{}; {}{}",
            (match self.min {
                Bound::Included(_) => "[",
                _ => "(",
            }),
            (match self.min {
                Bound::Included(start) => start.to_string(),
                Bound::Excluded(start) => start.to_string(),
                Bound::Unbounded => "".to_string(),
            }),
            (match self.max {
                Bound::Included(end) => end.to_string(),
                Bound::Excluded(end) => end.to_string(),
                Bound::Unbounded => "".to_string(),
            }),
            (match self.min {
                Bound::Included(_) => "]",
                _ => ")",
            })
        )
    }
}

impl BoundsRefinement {
    pub fn get_min(&self) -> Option<f64> {
        get_value(&self.min)
    }

    pub fn get_max(&self) -> Option<f64> {
        get_value(&self.max)
    }
}

impl RangeBounds<f64> for BoundsRefinement {
    fn start_bound(&self) -> Bound<&f64> {
        self.min.as_ref()
    }

    fn end_bound(&self) -> Bound<&f64> {
        self.max.as_ref()
    }
}

impl Refinement for BoundsRefinement {
    fn apply(&self, value: &mut Value, warnings: &mut Vec<anyhow::Error>) -> anyhow::Result<()> {
        let validated: f64 = match value {
            Value::Number(_) => validate_number(value)?,
            Value::String(str) => str.len() as f64,
            Value::Array(arr) => arr.len() as f64,
            _ => return Err(inapplicable(self, value)),
        };

        if self.contains(&validated) {
            Ok(())
        } else {
            let err = BoundsRefinementError {
                value: value.clone(),
                bounds: *self,
            }
            .into();
            if value.is_number() {
                *value = Value::from(self.clamp(validated));
                warnings.push(err);
                Ok(())
            } else {
                Err(err)
            }
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

    fn clone(&self) -> Box<dyn Refinement> {
        Box::new(Clone::clone(self))
    }

    fn is_subset_of(&self, other: &dyn Refinement) -> Relation {
        if let Some(other) = other.downcast_ref::<BoundsRefinement>() {
            if other.coerce {
                return Relation::Subset;
            }
            // Match min values against each other
            match (get_value(&other.min), get_value(&self.min)) {
                // When other refinement has no min value, we are valid, so just do nothing
                (None, _) => {}
                // When other refinement has min value but this one doesn't, return an error
                (Some(_), None) => {
                    return cast_error(*self, *other);
                }
                // When both refinements have min value, compare them
                (Some(other_min), Some(self_min)) => {
                    // Cast error when own value is lower than other's
                    if self_min < other_min {
                        return cast_error(*self, *other);
                    }
                    // Cast error when own min value is inclusive but other is exclusive with the same value
                    if self_min == other_min && is_exclusive(&other.min) && !is_exclusive(&self.min)
                    {
                        return cast_error(*self, *other);
                    }
                }
            }
            // Match max values against each other
            match (get_value(&other.max), get_value(&self.max)) {
                // When other refinement has no max value, we are valid, so just do nothing
                (None, _) => {}
                // When other refinement has max value but this one doesn't, return an error
                (Some(_), None) => {
                    return cast_error(*self, *other);
                }
                // When both refinements have max value, compare them
                (Some(other_max), Some(self_max)) => {
                    // Cast error when own value is larger than other's
                    if self_max > other_max {
                        return cast_error(*self, *other);
                    }
                    // Cast error when own max is inclusive but other is exclusive with the same value
                    if self_max == other_max && is_exclusive(&other.max) && !is_exclusive(&self.max)
                    {
                        return cast_error(*self, *other);
                    }
                }
            }

            // If both checks passed then all is fine
            Relation::Subset
        } else {
            Relation::Unrelated
        }
    }
}

fn get_value(bound: &Bound<f64>) -> Option<f64> {
    match bound {
        Bound::Included(x) => Some(*x),
        Bound::Excluded(x) => Some(*x),
        Bound::Unbounded => None,
    }
}

fn is_exclusive(bound: &Bound<f64>) -> bool {
    match bound {
        Bound::Excluded(_) => true,
        _ => false,
    }
}

#[derive(Error, Debug)]
pub struct BoundsRefinementError {
    value: Value,
    bounds: BoundsRefinement,
}

impl Display for BoundsRefinementError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match &self.value {
            Value::Number(n) => write!(f, "Number is out of {}: {}", self.bounds, n),
            Value::String(str) => {
                write!(f, "String length is out of {}: {}", self.bounds, str.len())
            }
            Value::Array(arr) => write!(f, "Array length is out of {}: {}", self.bounds, arr.len()),
            _ => panic!("Invalid BoundsRefinementError state"),
        }
    }
}

fn cast_error(got: BoundsRefinement, expected: BoundsRefinement) -> Relation {
    Relation::Conflict(RefinementCastError { got, expected }.into())
}
