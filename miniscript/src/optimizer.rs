use crate::ast::{Body, Spanned, Statement, AST};

pub fn optimize(ast: AST) -> AST {
    let body = Body::from(ast);
    let output = vec![];
    for statement in body {}
    return AST::from_body(output).expect("Optimization resulted in invalid AST");
}

struct OptimizationContext {
    use_locals_map: bool,
}

fn optimize_statement((statement, span): &Spanned<Statement>, output: &mut Body) {
    match statement {
        Statement::Assignment(_, _) => {}
        Statement::Expression(_) => {}
        Statement::If(_, _) => {}
        Statement::While(_, _) => {}
        Statement::For(_, _, _) => {}
        Statement::Break => {}
        Statement::Continue => {}
        Statement::Return(_) => {}
        Statement::Error => {}
    }
}
