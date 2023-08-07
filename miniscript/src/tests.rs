use crate::compile;
use crate::vm::chunk::pretty_print;
use ariadne::{sources, Report};
use std::io::BufWriter;
use std::ops::Range;

fn report_to_string(report: Report<(String, Range<usize>)>, code: &str) -> String {
    let mut buf = BufWriter::new(Vec::new());
    report
        .write(sources([("<eval>".to_owned(), code.to_string())]), &mut buf)
        .unwrap();
    let buf = buf.into_inner().unwrap();
    let buf = strip_ansi_escapes::strip(&buf).unwrap();
    String::from_utf8(buf).unwrap()
}

fn review_code(code: &str) -> String {
    compile("<eval>", code)
        .map(|result| format!("{code}\n------\n{}", pretty_print(&result, code)))
        .unwrap_or_else(|err| {
            err.into_iter()
                .map(|err| report_to_string(err.report(None, None), code))
                .collect::<Vec<String>>()
                .join("\n\n")
        })
}

macro_rules! review {
    ($src:expr) => {
        let result = review_code($src);
        insta::assert_display_snapshot!(result);
    };
}

#[test]
fn test_binary() {
    review!("1 + 2");
}

#[test]
fn test_print() {
    review!("print 1 + 2");
}

#[test]
fn test_single_comparison() {
    review!("print 1 == 2");
}

#[test]
fn test_chained_comparison() {
    review!("print 1 == 2 > 1 < 3");
}

#[test]
fn test_fail() {
    review!("if a print 5");
}
