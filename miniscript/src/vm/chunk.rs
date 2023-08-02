use crate::ast::{ast_err, BinaryOp, Body, Expr, Path, Span, Spanned, Statement, Value, AST};
use crate::vm::op_code::OpCode;
use crate::vm::register::StackIndex;
use rustc_hash::FxHashMap;
use std::cmp::Reverse;
use std::collections::BinaryHeap;

#[derive(Debug, Copy, Clone)]
pub struct ConstantIndex(usize);

#[derive(Debug, Clone)]
pub struct Chunk {
    code: Vec<OpCode>,
    spans: Vec<Span>,
    strings: Vec<String>,
    stack_size: usize,
}

impl Chunk {
    pub fn get_constant(&self, index: &ConstantIndex) -> &str {
        &self.strings[index.0]
    }

    pub fn code(&self) -> &Vec<OpCode> {
        &self.code
    }

    pub fn spans(&self) -> &Vec<Span> {
        &self.spans
    }

    pub fn stack_size(&self) -> usize {
        self.stack_size
    }
}

pub fn compile<'src>(ast: AST<'src>) -> Chunk {
    let body = Body::from(ast);
    let mut ctx = FunctionCompilationContext::<'src>::default();
    compile_body(&body, &mut ctx);
    ctx.chunk.stack_size = ctx.next_register;
    ctx.chunk
}

struct FunctionCompilationContext<'src> {
    use_locals_map: bool,
    declared_variables: FxHashMap<&'src str, StackIndex>,
    next_register: usize,
    free_registers: BinaryHeap<Reverse<StackIndex>>,
    chunk: Chunk,
}

impl<'src> FunctionCompilationContext<'src> {
    fn actualize(&mut self, register: Option<StackIndex>) -> StackIndex {
        match register {
            None => self.get_register(),
            Some(reg) => reg,
        }
    }

    fn get_register(&mut self) -> StackIndex {
        if let Some(register) = self.free_registers.pop() {
            return register.0;
        }
        let next = self.next_register;
        self.next_register += 1;
        return StackIndex(next);
    }

    fn release_register(&mut self, register: StackIndex) {
        self.free_registers.push(Reverse(register));
    }

    fn release_if_unused(&mut self, register: StackIndex) {
        if self.declared_variables.values().any(|x| *x == register) {
            return;
        }
        self.release_register(register);
    }

    fn local_register(&self, ident: &str) -> Option<StackIndex> {
        if self.use_locals_map {
            return None;
        }

        self.declared_variables.get(ident).copied()
    }

    fn new_local(&mut self, ident: &'src str) -> StackIndex {
        let register = self.get_register();
        self.declared_variables.insert(ident, register);
        register
    }

    fn emit(&mut self, code: OpCode, span: Span) {
        self.chunk.code.push(code);
        self.chunk.spans.push(span);
    }

    fn get_or_create_constant_index(&mut self, item: &str) -> ConstantIndex {
        if let Some(index) = self.chunk.strings.iter().position(|x| x == item) {
            return ConstantIndex(index);
        }
        self.chunk.strings.push(item.to_owned());
        return ConstantIndex(self.chunk.strings.len() - 1);
    }

    fn check_for_trash(&self) {
        // Each register must be either free, or in a local variable
        for i in 0..self.next_register {
            let register = StackIndex(i);
            assert!(
                self.declared_variables.values().any(|x| *x == register)
                    || self.free_registers.iter().any(|x| x.0 == register),
                "Register {} is not released",
                i
            )
        }
    }
}

impl<'src> Default for FunctionCompilationContext<'src> {
    fn default() -> Self {
        Self {
            use_locals_map: false,
            declared_variables: Default::default(),
            next_register: 0,
            free_registers: Default::default(),
            chunk: Chunk {
                code: vec![],
                spans: vec![],
                strings: vec![],
                stack_size: 0,
            },
        }
    }
}

fn compile_body<'src>(body: &Body<'src>, ctx: &mut FunctionCompilationContext<'src>) {
    for (statement, span) in body {
        match statement {
            Statement::Expression(expr) => {
                // Expressions statements without side effects are ignored
                if !can_have_side_effects(&expr.0, ctx) {
                    continue;
                }

                let reg = compile_expressions(expr, None, ctx, false);
                ctx.release_if_unused(reg)
            }
            Statement::Assignment(lhs, rhs) => compile_assignment(lhs, rhs, span, ctx),
            Statement::If(_, _) => todo!(),
            Statement::While(_, _) => todo!(),
            Statement::For(_, _, _) => todo!(),
            Statement::Break => todo!(),
            Statement::Continue => todo!(),
            Statement::Return(_) => todo!(),
            Statement::Error => todo!(),
        }
        ctx.check_for_trash();
    }
}

fn compile_assignment<'src>(
    lhs: &Spanned<Expr<'src>>,
    rhs: &Spanned<Expr<'src>>,
    _span: &Span,
    ctx: &mut FunctionCompilationContext<'src>,
) {
    match &lhs.0 {
        Expr::Path(path) => match path {
            Path::AnyScope(ident) => {
                let lhs = ctx
                    .local_register(ident)
                    .unwrap_or_else(|| ctx.new_local(ident));
                let _ = compile_expressions(rhs, Some(lhs), ctx, false);
            }
        },
        Expr::Index(_, _) => {
            todo!();
        }
        Expr::ExprIndex(_, _) => {
            todo!();
        }
        _ => {
            unreachable!("Invalid assignment target");
        }
    };
}

#[must_use]
fn compile_expressions<'src>(
    (expression, span): &Spanned<Expr<'src>>,
    register: Option<StackIndex>,
    ctx: &mut FunctionCompilationContext<'src>,
    suppress_call: bool,
) -> StackIndex {
    let new_register = match &expression {
        Expr::Value(val) => {
            let register = ctx.actualize(register);
            match val {
                Value::Null => ctx.emit(OpCode::SetNull(register), *span),
                Value::Num(number) => ctx.emit(OpCode::SetNumber(register, *number), *span),
                Value::Boolean(condition) => ctx.emit(
                    OpCode::SetNumber(register, (*condition) as i32 as f64),
                    *span,
                ),
                Value::String(string) => {
                    let index = ctx.get_or_create_constant_index(string);
                    ctx.emit(OpCode::SetString(register, index), *span)
                }
            }
            register
        }
        Expr::Path(path) => compile_path(path, *span, register, ctx, suppress_call),
        Expr::List(_) => todo!(),
        Expr::Map(_) => todo!(),
        Expr::FunctionDefinition(_, _) => todo!(),
        Expr::Comparison(_, _) => todo!(),
        Expr::Binary(lhs, op, rhs) => compile_binary_op(lhs, op, rhs, *span, register, ctx),
        Expr::Unary(_, _) => todo!(),
        Expr::Call(expr, arguments) => compile_function_call(expr, arguments, *span, register, ctx),
        Expr::ExprIndex(_, _) => todo!(),
        Expr::Index(_, _) => todo!(),
        Expr::Error => ast_err!(),
    };
    if let Some(register) = register {
        assert_eq!(register, new_register, "Register mismatch");
    }
    new_register
}

fn compile_path<'src>(
    path: &Path<'src>,
    span: Span,
    register: Option<StackIndex>,
    ctx: &mut FunctionCompilationContext<'src>,
    suppress_call: bool,
) -> StackIndex {
    match path {
        Path::AnyScope(ident) => {
            if suppress_call {
                match (ctx.local_register(ident), register) {
                    // No target register provided and local variable exists
                    (Some(index), None) => {
                        index // Direct access
                    }
                    // Target is provided, and local variable exists
                    (Some(index), Some(output)) => {
                        ctx.emit(
                            OpCode::Copy {
                                source: index,
                                output,
                            },
                            span,
                        );
                        output
                    }
                    // Local variable does not exist
                    (None, _) => {
                        let register = ctx.actualize(register);
                        ctx.emit(OpCode::ReadVariable(register, (*ident).to_owned()), span);
                        register
                    }
                }
            } else {
                let register = ctx.actualize(register);
                let input = ctx.local_register(ident).unwrap_or_else(|| {
                    ctx.emit(OpCode::ReadVariable(register, (*ident).to_owned()), span);
                    register
                });
                // Calls found value with no arguments
                ctx.emit(
                    OpCode::Call0 {
                        function: input,
                        output: register,
                    },
                    span,
                );
                register
            }
        }
    }
}

fn compile_binary_op<'src>(
    lhs: &Spanned<Expr<'src>>,
    op: &BinaryOp,
    rhs: &Spanned<Expr<'src>>,
    span: Span,
    register: Option<StackIndex>,
    ctx: &mut FunctionCompilationContext<'src>,
) -> StackIndex {
    let op = *op;
    if op == BinaryOp::Or || op == BinaryOp::And {
        todo!("short circuit binary operators")
    }
    let output = ctx.actualize(register);
    // If the right side can have side effects, we create a new register for lhs
    // Note that when locals map is not used, lhs variable can't be changed by side effects
    // of rhs
    let lhs_reg = if ctx.use_locals_map && can_have_side_effects(&rhs.0, ctx) {
        Some(ctx.get_register())
    } else {
        None
    };
    let lhs = compile_expressions(lhs, lhs_reg, ctx, false);
    let rhs = compile_expressions(rhs, None, ctx, false);
    let op = match op {
        BinaryOp::Or | BinaryOp::And => unreachable!(),
        BinaryOp::Add => OpCode::Add { output, lhs, rhs },
        BinaryOp::Sub => OpCode::Subtract { output, lhs, rhs },
        BinaryOp::Mul => OpCode::Multiply { output, lhs, rhs },
        BinaryOp::Div => OpCode::Divide { output, lhs, rhs },
        BinaryOp::Pow => OpCode::Pow { output, lhs, rhs },
    };
    ctx.emit(op, span);
    ctx.release_if_unused(lhs);
    ctx.release_if_unused(rhs);
    output
}

fn compile_function_call<'src>(
    callee: &Spanned<Expr<'src>>,
    args: &Vec<Spanned<Expr<'src>>>,
    span: Span,
    register: Option<StackIndex>,
    ctx: &mut FunctionCompilationContext<'src>,
) -> StackIndex {
    if let Expr::Path(Path::AnyScope("print")) = callee.0 {
        if args.len() != 1 {
            unimplemented!("Print only support one argument at the moment")
        }

        let arg = compile_expressions(&args[0], None, ctx, false);
        ctx.emit(OpCode::Print(arg), span);
        ctx.release_if_unused(arg);
        let register = ctx.actualize(register);
        ctx.emit(OpCode::SetNull(register), span);
        return register;
    }
    todo!("function calls");
}

fn can_have_side_effects(expr: &Expr, ctx: &FunctionCompilationContext) -> bool {
    match expr {
        // Constant values never have side effects
        Expr::Value(_) => false,
        // Variable access can have side effects when locals are a map
        Expr::Path(_) => ctx.use_locals_map,
        // Lists only have side effects if one of the elements also have side effects
        Expr::List(items) => items.iter().any(|x| can_have_side_effects(&x.0, ctx)),
        // Same for maps, but also checking keys
        Expr::Map(items) => items
            .iter()
            .any(|x| can_have_side_effects(&x.0 .0, ctx) || can_have_side_effects(&x.1 .0, ctx)),
        // Function definitions never have side effects
        Expr::FunctionDefinition(_, _) => false,
        // All operations have side effects if one of the operands have side effects
        Expr::Comparison(a, chain) => {
            can_have_side_effects(&(**a).0, ctx)
                || chain.iter().any(|x| can_have_side_effects(&x.1 .0, ctx))
        }
        Expr::Binary(lhs, _, rhs) => {
            can_have_side_effects(&(**lhs).0, ctx) || can_have_side_effects(&(**rhs).0, ctx)
        }
        Expr::Unary(_, expr) => can_have_side_effects(&(**expr).0, ctx),
        // Function calls can have side effects
        Expr::Call(_, _) => true,
        // Indexing can have side effects
        Expr::ExprIndex(_, _) => true,
        Expr::Index(_, _) => true,
        Expr::Error => ast_err!(),
    }
}
