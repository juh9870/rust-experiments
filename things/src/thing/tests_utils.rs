use std::any::type_name;
use std::error::Error;

use serde_json::Value;

use crate::thing::{Refinement, Relation, Thing};

macro_rules! prop_unwrap {
    ($expression:expr) => {
        match $expression {
            Ok(data) => data,
            Err(err) => {
                proptest::prop_assert!(false, "{err}");
                panic!("Bad macro");
            }
        }
    };
}
macro_rules! prop_assert_type {
    ($expression:expr, Number) => {{
        let value = $expression;
        if let Some(data) = value.as_f64() {
            Ok(data)
        } else {
            Err(anyhow::anyhow!("Invalid value type: {}", value))
        }
    }};
    ($expression:expr, $expected_type:ty) => {
        if let $expected_type(data) = $expression {
            Ok(data)
        } else {
            Err(anyhow::anyhow!("Invalid value type: {}", value))
        }
    };
}
//
// macro_rules! discard {
//     ($expression:expr) => {
//         if $expression {
//             return TestResult::discard();
//         }
//     };
// }

// pub(crate) use discard;
pub(crate) use prop_assert_type;
pub(crate) use prop_unwrap;

pub(crate) trait TestAssertionResult<T> {
    fn assert(self) -> T;
}

impl<T> TestAssertionResult<T> for anyhow::Result<T> {
    fn assert(self) -> T {
        match self {
            Ok(data) => data,
            Err(err) => panic!("{err}"),
        }
    }
}

pub(crate) trait TestConversionResult<R> {
    fn success(self) -> anyhow::Result<R>;
    fn error<T: Error + Send + Sync + 'static>(self) -> anyhow::Result<T>;
}

pub(crate) trait TestConversionResultWithWarnings<R> {
    fn with_warnings(self, warnings: usize) -> anyhow::Result<R>;
}

impl<R> TestConversionResultWithWarnings<Vec<R>> for anyhow::Result<Vec<R>> {
    fn with_warnings(self, warnings: usize) -> anyhow::Result<Vec<R>> {
        match self {
            Ok(warns) => {
                if warns.len() == warnings {
                    Ok(warns)
                } else {
                    Err(anyhow::anyhow!(
                        "Got {} warnings where {warnings} warnings were expected",
                        warns.len()
                    ))
                }
            }
            Err(err) => Err(anyhow::anyhow!("{err}")),
        }
    }
}

impl<R, E> TestConversionResultWithWarnings<(E, Vec<R>)> for anyhow::Result<(E, Vec<R>)> {
    fn with_warnings(self, warnings: usize) -> anyhow::Result<(E, Vec<R>)> {
        match self {
            Ok(warns) => {
                if warns.1.len() == warnings {
                    Ok(warns)
                } else {
                    Err(anyhow::anyhow!(
                        "Got {} warnings where {warnings} warnings were expected",
                        warns.1.len()
                    ))
                }
            }
            Err(err) => Err(anyhow::anyhow!("{err}")),
        }
    }
}

impl<R> TestConversionResult<R> for anyhow::Result<R> {
    fn success(self) -> anyhow::Result<R> {
        match self {
            Ok(warns) => Ok(warns),
            Err(err) => Err(anyhow::anyhow!("{err}")),
        }
    }

    fn error<T: Error + Send + Sync + 'static>(self) -> anyhow::Result<T> {
        match self {
            Ok(_) => Err(anyhow::anyhow!("Error was expected")),
            Err(anyhow_err) => is_err_of_type(anyhow_err),
        }
    }
}

pub(crate) trait TestRefinement {
    fn check<T: Into<Value>>(self, value: T) -> anyhow::Result<(Value, Vec<anyhow::Error>)>;
}

impl<R: Refinement> TestRefinement for R {
    fn check<T: Into<Value>>(self, value: T) -> anyhow::Result<(Value, Vec<anyhow::Error>)> {
        let thing = Thing::from(self);
        let mut value = value.into();
        let warnings = thing.apply(&mut value)?;
        Ok((value, warnings))
    }
}

fn is_err_of_type<T: Error + Send + Sync + 'static>(
    anyhow_err: anyhow::Error,
) -> Result<T, anyhow::Error> {
    let dbg = format!("{:?}", anyhow_err);
    if let Ok(err) = anyhow_err.downcast::<T>() {
        Ok(err)
    } else {
        Err(anyhow::anyhow!(
            "Got error {} where error of type {} was expected",
            dbg,
            type_name::<T>()
        ))
    }
}

pub fn do_assert(condition: bool, err: &str) -> anyhow::Result<()> {
    if condition {
        Ok(())
    } else {
        Err(anyhow::anyhow!("{err}"))
    }
}

pub fn subset(a: &dyn Refinement, of: &dyn Refinement) -> anyhow::Result<()> {
    match a.is_subset_of(of) {
        Relation::Subset => Ok(()),
        relation => Err(anyhow::anyhow!(
            "Subset relation was expected between {} and {}, but got {:?}",
            a,
            of,
            relation
        )),
    }
}

pub fn unrelated(a: &dyn Refinement, of: &dyn Refinement) -> anyhow::Result<()> {
    match a.is_subset_of(of) {
        Relation::Unrelated => Ok(()),
        relation => Err(anyhow::anyhow!(
            "Unrelated relation was expected between {} and {}, but got {:?}",
            a,
            of,
            relation
        )),
    }
}

impl Relation {}
