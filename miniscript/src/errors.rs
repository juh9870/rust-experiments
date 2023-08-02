use thiserror::Error;

pub const CODE_INTERNAL: u16 = 0;
pub const CODE_RUNTIME: u16 = 1000;
pub const CODE_COMPILE: u16 = 2000;

#[derive(Debug, Error)]
pub enum MsError {
    #[error("{}", .0)]
    Internal(InternalError),
    #[error("{}", .0)]
    Runtime(RuntimeError),
    #[error("{}", .0)]
    Compile(CompileError),
}

impl MsError {
    pub fn code(&self) -> u16 {
        match self {
            MsError::Internal(item) => item.code(),
            MsError::Runtime(item) => item.code(),
            MsError::Compile(item) => item.code(),
        }
    }
}

#[derive(Debug, Error)]
#[non_exhaustive]
pub enum InternalError {
    #[error("Operation not implemented")]
    NotImplemented,
}

impl InternalError {
    fn raw_code(&self) -> u16 {
        match self {
            InternalError::NotImplemented => 0,
        }
    }
}

#[derive(Debug, Error)]
#[non_exhaustive]
pub enum RuntimeError {}

impl RuntimeError {
    fn raw_code(&self) -> u16 {
        // match self {}
        unreachable!()
    }
}

#[derive(Debug, Error)]
#[non_exhaustive]
pub enum CompileError {}

impl CompileError {
    fn raw_code(&self) -> u16 {
        // match self {}
        unreachable!()
    }
}

macro_rules! error_type {
    ($err:ty, $variant:path, $offset:expr) => {
        impl From<$err> for MsError {
            fn from(value: $err) -> Self {
                $variant(value)
            }
        }

        impl $err {
            pub fn code(&self) -> u16 {
                return self.raw_code() + $offset;
            }
        }
    };
}

error_type!(InternalError, MsError::Internal, CODE_INTERNAL);
error_type!(RuntimeError, MsError::Runtime, CODE_RUNTIME);
error_type!(CompileError, MsError::Compile, CODE_COMPILE);
