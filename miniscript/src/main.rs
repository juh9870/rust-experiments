use crate::ast::{Spanned, AST};
use crate::parsing::ast_parser::parser;
use crate::parsing::parser::{lexer, Token};
use crate::value::Value;
use crate::vm::chunk::{compile, pretty_print};
use crate::vm::{DefaultRunner, Vm, VmRunner};
use ariadne::{sources, Color, ColorGenerator, Label, Report, ReportKind, Source};
use chumsky::error::Rich;
use chumsky::prelude::Input;
use chumsky::{ParseResult, Parser};
use std::{env, fs};

pub mod ast;
pub mod errors;
pub mod optimizer;
pub mod parsing;
pub mod value;
pub mod vm;

fn main() {
    let filename = env::args().nth(1).expect("Expected file argument");

    println!("{filename}");

    // println!("cwd: {:?}", env::current_dir());

    let src = fs::read_to_string(&filename).expect("Failed to read file");

    let p = lexer();

    let mut result: ParseResult<Vec<Spanned<Token>>, Rich<char>> = p.parse(src.as_str());
    //     let result = p.parse("\n");
    println!("Tokens:\n\n{result:?}\n\n");

    if let Some(tokens) = result.into_output() {
        let filtered = tokens
            .into_iter()
            .filter(|token| !matches!(token.0, Token::Comment(_)))
            .collect::<Vec<_>>();
        let mut a = 1;

        let astp = parser();
        let result = astp.parse((&filtered).spanned((src.len()..src.len()).into()));

        // for err in result.errors() {
        //     let mut colors = ColorGenerator::new();
        //     let a = colors.next();
        //     let b = colors.next();
        //     Report::build(ReportKind::Error, "<eval>", 0)
        //         .with_message(&err.to_string())
        //         .with_label(
        //             Label::new(("<eval>", err.span().into_range())).with_color(a)
        //         ).finish().print(("<eval>", Source::from(src))).unwrap();
        // }

        result
            .errors()
            .map(|e| e.clone().map_token(|c| c.to_string()))
            // .chain(
            //     parse_errs
            //         .into_iter()
            //         .map(|e| e.map_token(|tok| tok.to_string())),
            // )
            .for_each(|e| {
                Report::build(ReportKind::Error, filename.clone(), e.span().start)
                    .with_message(e.to_string())
                    .with_label(
                        Label::new((filename.clone(), e.span().into_range()))
                            .with_message(e.reason().to_string())
                            .with_color(Color::Red),
                    )
                    .with_labels(e.contexts().map(|(label, span)| {
                        Label::new((filename.clone(), span.into_range()))
                            .with_message(format!("while parsing this {}", label))
                            .with_color(Color::Yellow)
                    }))
                    .finish()
                    .print(sources([(filename.clone(), src.clone())]))
                    .unwrap()
            });

        println!("AST:\n\n{result:?}\n\n");

        if !(result.has_errors()) {
            if let Some(body) = result.into_output() {
                let chunk = compile(AST::from_body_unchecked(body));

                println!("{}", pretty_print(&chunk, &src));
                let mut vm = Vm {
                    cursor: 0,
                    stack: vec![Value::Null; chunk.stack_size()],
                };
                DefaultRunner.run(&chunk, &mut vm).unwrap();
            }
        }
    }

    // println!("{}", result.unwrap()[0].0)
}
