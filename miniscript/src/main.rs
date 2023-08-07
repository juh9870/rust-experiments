use crate::ast::{Spanned, AST};
use crate::errors::{CompileError, MsError, MsErrorType};
use crate::parsing::ast_parser::ast_parser;
use crate::parsing::parser::{lexer, Token};
use crate::value::Value;
use crate::vm::chunk::{compile_chunk, pretty_print, Chunk};
use crate::vm::{DefaultRunner, Vm, VmRunner};
use ariadne::{sources, Color, ColorGenerator, Label, Report, ReportKind, Source};
use chumsky::error::Rich;
use chumsky::prelude::Input;
use chumsky::{ParseResult, Parser};
use std::fmt::Display;
use std::{env, fs, process};

pub mod ast;
pub mod errors;
pub mod parsing;
#[cfg(test)]
pub mod tests;
pub mod value;
pub mod vm;

fn format_errors<T: Display>(src_id: &str, errors: Vec<Rich<T>>) -> Vec<MsError> {
    errors
        .into_iter()
        .map(CompileError::from_compilation)
        .map(|err| MsError {
            error_type: err.into(),
            src_id: src_id.to_string(),
        })
        .collect()
}

pub fn compile(src_id: &str, src: &str) -> Result<Chunk, Vec<MsError>> {
    let lexer = lexer();
    let (tokens, errors) = lexer.parse(src).into_output_errors();

    if !errors.is_empty() {
        return Err(format_errors(src_id, errors));
    }

    let tokens = tokens.expect("Tokens output is none, but no errors were emitted either");
    let stripped = tokens
        .into_iter()
        .filter(|token| !matches!(token.0, Token::Comment(_)))
        .collect::<Vec<_>>();
    let spanned = stripped.spanned((src.len()..src.len()).into());

    let ast_parser = ast_parser();

    let (ast, errors) = ast_parser.parse(spanned).into_output_errors();

    if !errors.is_empty() {
        return Err(format_errors(src_id, errors));
    }

    let ast = ast.expect("AST output is none, but no errors were emitted either");

    let ast = AST::from_body(ast, src_id.to_string()).map_err(|err| vec![err])?;

    let chunk = compile_chunk(ast);

    Ok(chunk)
}

fn main() {
    let filename = env::args().nth(1).expect("Expected file argument");

    println!("{filename}");

    // println!("cwd: {:?}", env::current_dir());

    let src = fs::read_to_string(&filename).expect("Failed to read file");

    let chunk = compile(&filename, &src).unwrap_or_else(|errors| {
        for err in errors {
            err.report(None, None)
                .print(sources([(filename.clone(), src.clone())]))
                .expect("Failed to print error message");
        }
        process::exit(1);
    });

    println!("{}", pretty_print(&chunk, &src));
    let mut vm = Vm {
        cursor: 0,
        stack: vec![Value::Null; chunk.stack_size()],
    };
    DefaultRunner.run(&chunk, &mut vm).unwrap_or_else(|err| {
        err.report(Some(&chunk), Some(&vm))
            .print(sources([(filename.clone(), src.clone())]))
            .expect("Failed to print error message");
        process::exit(1);
    });

    // println!("{}", result.unwrap()[0].0)
}
