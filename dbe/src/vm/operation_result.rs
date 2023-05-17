use anyhow::Error;

use crate::vm::VmErrors;

#[derive(Debug)]
pub enum OperationResult {
    Ok,
    Warn(Vec<Error>),
    Err(Error),
}

impl<T: Into<OperationResult>> From<Result<Vec<Error>, T>> for OperationResult {
    fn from(value: Result<Vec<Error>, T>) -> Self {
        match value {
            Ok(warns) => warns.into(),
            Err(err) => err.into(),
        }
    }
}

impl From<Vec<Error>> for OperationResult {
    fn from(warns: Vec<Error>) -> Self {
        if warns.is_empty() {
            OperationResult::Ok
        } else {
            OperationResult::Warn(warns)
        }
    }
}

impl From<Error> for OperationResult {
    fn from(err: Error) -> Self {
        return OperationResult::Err(err);
    }
}

impl From<VmErrors> for OperationResult {
    fn from(err: VmErrors) -> Self {
        return OperationResult::Err(err.into());
    }
}

#[macro_export]
macro_rules! op_result {
    ($body:expr) => {
        match $body {
            Ok(data) => data,
            Err(err) => {
                return err.into();
            }
        }
    };
}

pub use op_result;
