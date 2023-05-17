use std::any::{type_name, TypeId};
use std::fmt::{Debug, Display};

use anyhow::{Error, Result};
use as_any::{AsAny, Downcast};
use serde_json::Value;
use thiserror::Error;

use crate::thing::primitives::PrimitiveRefinement;

pub mod bounds;
pub mod multiple_of;
pub mod primitives;

#[cfg(test)]
pub mod tests_utils;

pub trait Refinement: Debug + Display + AsAny {
    fn apply(&self, value: &mut Value, warnings: &mut Vec<anyhow::Error>) -> Result<()>;
    fn strict(&self) -> Box<dyn Refinement>;
    fn clone(&self) -> Box<dyn Refinement>;
    fn is_subset_of(&self, other: &dyn Refinement) -> Relation;
}

#[derive(Debug)]
pub enum Relation {
    /// Refinement is not related to other refinement on any way
    Unrelated,
    /// Refinement is a subset of other refinement
    Subset,
    /// Refinement is conflicting with other refinement
    Conflict(anyhow::Error),
}

#[derive(Error, Debug)]
#[error("Refinement `{}` is not satisfied", .0)]
pub struct MissingRefinementError(String);

#[derive(Debug)]
pub struct Thing(Vec<Box<dyn Refinement>>);

impl Clone for Thing {
    fn clone(&self) -> Self {
        let data = self.0.iter().map(|e| (**e).clone()).collect();
        Thing(data)
    }
}

impl<T: Refinement> From<Vec<T>> for Thing {
    fn from(values: Vec<T>) -> Self {
        let items = values
            .into_iter()
            .map(|item| Box::new(item) as Box<dyn Refinement>)
            .collect();
        Thing(items)
    }
}

impl<T: Refinement> From<T> for Thing {
    fn from(value: T) -> Self {
        Thing::from(vec![value])
    }
}

impl<R: Refinement> FromIterator<R> for Thing {
    fn from_iter<T: IntoIterator<Item = R>>(iter: T) -> Self {
        let items = iter
            .into_iter()
            .map(|item| Box::new(item) as Box<dyn Refinement>)
            .collect();
        Thing(items)
    }
}

impl Thing {
    fn try_assign_to(&self, other: &Thing) -> Result<()> {
        if let Some(err) = other
            .0
            .iter()
            .filter_map(|other| {
                // For each refinement in 'other', iterate over the refinements in 'self'.
                // Use filter_map to transform the iterator, keeping only the refinements
                // with a relationship to the current refinement in 'other'.
                if let Some(rel) = self
                    .0
                    .iter()
                    .filter_map(|refinement| {
                        match refinement.is_subset_of(other.as_ref()) {
                            // Skip unrelated refinements, only keeping successes or errors.
                            Relation::Unrelated => None,
                            rel => Some(rel),
                        }
                    })
                    .next()
                {
                    return if let Relation::Conflict(err) = rel {
                        Some(err)
                    } else {
                        None
                    };
                }
                // If there's no relationship found for the current refinement in 'other',
                // return a MissingRefinementError.
                Some(MissingRefinementError(other.to_string()).into())
            })
            .next()
        {
            return Err(err);
        }
        Ok(())
    }
}

pub trait ThingLike {
    fn get_refinement<T: 'static>(&self) -> Option<&T>;
    fn apply(&self, value: &mut Value) -> Result<Vec<anyhow::Error>>;
}

impl<R: Refinement> ThingLike for R {
    fn get_refinement<T: 'static>(&self) -> Option<&T> {
        return self.downcast_ref();
    }

    fn apply(&self, value: &mut Value) -> Result<Vec<Error>> {
        let mut warnings = Vec::new();
        self.apply(value, &mut warnings)?;
        Ok(warnings)
    }
}

impl ThingLike for Thing {
    fn get_refinement<T: 'static>(&self) -> Option<&T> {
        self.0.iter().find_map(|x| {
            return (**x).downcast_ref::<T>();
        })
    }

    fn apply(&self, value: &mut Value) -> Result<Vec<Error>> {
        let mut warnings = Vec::new();
        for refinement in &self.0 {
            refinement.apply(value, &mut warnings)?;
        }
        Ok(warnings)
    }
}
