use crate::value::Value;
use crate::vm::chunk::Chunk;
use crate::vm::Vm;
use ariadne::{Color, Label, Report, ReportBuilder, ReportKind};
use chumsky::error::Rich;
use derive_more::Display;
use std::fmt::Display;
use std::ops::Range;
use thiserror::Error;

pub const CODE_INTERNAL: u16 = 0;
pub const CODE_RUNTIME: u16 = 1000;
pub const CODE_COMPILE: u16 = 2000;

#[derive(Debug, Error, Display)]
#[display(fmt = "{}", error_type)]
pub struct MsError {
    pub src_id: String,
    pub error_type: MsErrorType,
}

#[derive(Debug, Display)]
pub enum MsErrorType {
    #[display(fmt = "{}", _0)]
    Internal(InternalError),
    #[display(fmt = "{}", _0)]
    Runtime(RuntimeError),
    #[display(fmt = "{}", _0)]
    Compile(CompileError),
}

impl MsError {
    pub fn code(&self) -> u16 {
        match &self.error_type {
            MsErrorType::Internal(item) => item.code(),
            MsErrorType::Runtime(item) => item.code(),
            MsErrorType::Compile(item) => item.code(),
        }
    }

    pub fn report(&self, chunk: Option<&Chunk>, vm: Option<&Vm>) -> Report<(String, Range<usize>)> {
        match &self.error_type {
            MsErrorType::Internal(item) => item.report(&self.src_id, chunk, vm),
            MsErrorType::Runtime(item) => item.report(&self.src_id, chunk, vm),
            MsErrorType::Compile(item) => item.report(&self.src_id, chunk, vm),
        }
    }
}

fn report_template<'a>(
    src_id: &str,
    chunk: Option<&Chunk>,
    vm: Option<&Vm>,
) -> (ReportBuilder<'a, (String, Range<usize>)>, Range<usize>) {
    let span = match (chunk, vm) {
        (Some(chunk), Some(vm)) => chunk.spans()[vm.cursor].into_range(),
        _ => 0..0,
    };
    (
        Report::build(ReportKind::Error, src_id.to_string(), span.start),
        span,
    )
}

fn add_span_info<'a>(
    report: ReportBuilder<'a, (String, Range<usize>)>,
    src_id: &str,
    span: Range<usize>,
    message: &str,
) -> ReportBuilder<'a, (String, Range<usize>)> {
    if span.is_empty() {
        return report;
    }
    let mut label = Label::new((src_id.to_string(), span)).with_color(Color::Red);
    if !message.is_empty() {
        label = label.with_message(message);
    }

    return report.with_label(label);
}

#[derive(Debug, Error)]
#[non_exhaustive]
pub enum InternalError {
    #[error("Operation not implemented")]
    NotImplemented,
    #[error("OpCode at {} is unpatched", .0)]
    UnpatchedOpCode(usize),
}

impl InternalError {
    fn raw_code(&self) -> u16 {
        match self {
            InternalError::NotImplemented => 0,
            InternalError::UnpatchedOpCode(_) => 1,
        }
    }

    fn report(
        &self,
        src_id: &str,
        chunk: Option<&Chunk>,
        vm: Option<&Vm>,
    ) -> Report<(String, Range<usize>)> {
        let (report, span) = report_template(src_id, chunk, vm);
        let report = report.with_help(
            "This is internal error and should never happen. Please report this error to <TODO: GITHUB>",
        ).with_code(self.code());
        match self {
            InternalError::NotImplemented => {
                add_span_info(report.with_message("Not implemented"), src_id, span, "")
            }
            InternalError::UnpatchedOpCode(idx) => add_span_info(
                report.with_message(format!("OpCode at index {idx} is unpatched")),
                src_id,
                span,
                "",
            ),
        }
        .finish()
    }
}

#[derive(Debug, Error)]
#[non_exhaustive]
pub enum RuntimeError {
    #[error("{}", .0)]
    Custom(Value),
}

impl RuntimeError {
    fn raw_code(&self) -> u16 {
        match self {
            RuntimeError::Custom(_) => 0,
        }
    }

    fn report(
        &self,
        src_id: &str,
        chunk: Option<&Chunk>,
        vm: Option<&Vm>,
    ) -> Report<(String, Range<usize>)> {
        let (report, span) = report_template(src_id, chunk, vm);
        let report = report.with_code(self.code());
        match self {
            RuntimeError::Custom(msg) => add_span_info(report.with_message(msg), src_id, span, ""),
        }
        .finish()
    }
}

#[derive(Debug, Error)]
#[non_exhaustive]
pub enum CompileError {
    #[error("Parsing error: {}", .0)]
    Compilation(String, Range<usize>, Vec<(String, Range<usize>)>),
}

impl CompileError {
    fn raw_code(&self) -> u16 {
        match self {
            CompileError::Compilation(_, _, _) => 0,
        }
    }

    fn report(
        &self,
        src_id: &str,
        chunk: Option<&Chunk>,
        vm: Option<&Vm>,
    ) -> Report<(String, Range<usize>)> {
        let (report, span) = report_template(src_id, chunk, vm);
        let report = report.with_code(self.code());
        match self {
            CompileError::Compilation(message, span, contexts) => report
                .with_label(
                    Label::new((src_id.to_string(), span.clone()))
                        .with_message(message)
                        .with_color(Color::Red),
                )
                .with_labels(contexts.iter().map(|(message, span)| {
                    Label::new((src_id.to_string(), span.clone()))
                        .with_message(message)
                        .with_color(Color::Yellow)
                })),
        }
        .finish()
    }

    pub fn from_compilation<T: Display>(error: Rich<T>) -> Self {
        Self::Compilation(
            error.reason().to_string(),
            error.span().into_range(),
            error
                .contexts()
                .into_iter()
                .map(|(msg, span)| (msg.to_string(), span.into_range()))
                .collect(),
        )
    }
}

macro_rules! error_type {
    ($err:ty, $variant:path, $offset:expr) => {
        impl From<$err> for MsErrorType {
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

error_type!(InternalError, MsErrorType::Internal, CODE_INTERNAL);
error_type!(RuntimeError, MsErrorType::Runtime, CODE_RUNTIME);
error_type!(CompileError, MsErrorType::Compile, CODE_COMPILE);
