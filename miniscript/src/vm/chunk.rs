use crate::ast::{
    ast_err, BinaryOp, Body, Comparison, Expr, Path, Span, Spanned, Statement, UnaryOp, Value, AST,
};
use crate::vm::op_code::{BytecodeError, OpCode};
use crate::vm::register::StackIndex;
use rustc_hash::{FxHashMap, FxHashSet};
use std::cmp::Reverse;
use std::collections::BinaryHeap;

#[derive(Debug, Copy, Clone)]
pub struct ConstantIndex(usize);

impl ConstantIndex {
    pub fn raw(&self) -> usize {
        self.0
    }
}

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

pub fn pretty_print(chunk: &Chunk, source: &str) -> String {
    let pairs = chunk
        .code
        .iter()
        .map(|e| e.pretty_print())
        .zip(chunk.spans.iter())
        .collect::<Vec<(String, &Span)>>();

    let id_len = (chunk.code.len() - 1).ilog10() as usize + 1;

    let align = pairs.iter().map(|e| e.0.len()).max().unwrap_or(0);
    pairs
        .into_iter()
        .enumerate()
        .map(|(id, (code, span))| {
            format!(
                "{id:>id_len$}: {code:<align$}  |  {span}",
                span = &source[span.into_range()]
            )
        })
        .collect::<Vec<_>>()
        .join("\n")
}

pub fn compile<'src>(ast: AST<'src>) -> Chunk {
    let body = Body::from(ast);
    let mut ctx = FunctionCompilationContext::<'src>::default();
    compile_body(&body, &mut ctx);
    ctx.emit(OpCode::Return(None), Span::from(0..0));
    ctx.chunk.stack_size = ctx.next_register;
    ctx.chunk
}

struct FunctionCompilationContext<'src> {
    use_locals_map: bool,
    declared_variables: FxHashMap<&'src str, VariableInfo>,
    next_register: usize,
    free_registers: BinaryHeap<Reverse<StackIndex>>,
    patches: FxHashSet<usize>,
    assignment_spans: Vec<usize>,
    chunk: Chunk,
}

#[derive(Debug, Copy, Clone)]
struct VariableInfo {
    register: StackIndex,
    can_be_function: bool,
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
        StackIndex(next)
    }

    fn release_register(&mut self, register: StackIndex) {
        self.free_registers.push(Reverse(register));
    }

    fn release_if_unused(&mut self, register: StackIndex) -> bool {
        if self
            .declared_variables
            .values()
            .any(|x| x.register == register)
        {
            return false;
        }
        self.release_register(register);
        true
    }

    fn actualize_and_release_if_unused(
        &mut self,
        register: Option<StackIndex>,
    ) -> (StackIndex, bool) {
        match register {
            None => {
                let register = self.get_register();
                self.release_register(register);
                (register, true)
            }
            Some(register) => (register, self.release_if_unused(register)),
        }
    }

    fn take_back_register(&mut self, register: StackIndex) -> StackIndex {
        let size = self.free_registers.len();
        self.free_registers.retain(|x| x.0 != register);
        let new_size = self.free_registers.len();
        if size == new_size {
            panic!("Failed to take back the register {}", register);
        }
        register
    }

    fn local_register(&self, ident: &str) -> Option<StackIndex> {
        if self.use_locals_map {
            return None;
        }

        self.declared_variables.get(ident).map(|e| e.register)
    }

    fn new_local(&mut self, ident: &'src str, register: StackIndex) {
        if !self.use_locals_map {
            self.declared_variables.insert(
                ident,
                VariableInfo {
                    register,
                    can_be_function: true,
                },
            );
        }
    }

    fn can_be_function(&self, ident: &str) -> bool {
        if self.use_locals_map {
            return true;
        }

        self.declared_variables
            .get(ident)
            .map(|e| e.can_be_function)
            .unwrap_or(true)
    }

    fn set_can_be_function(&mut self, ident: &'src str, can_be_function: bool) {
        if self.use_locals_map {
            return;
        }

        self.declared_variables
            .entry(ident)
            .and_modify(|e| e.can_be_function = can_be_function);
    }

    fn emit(&mut self, code: OpCode, span: impl Into<Span>) {
        self.chunk.code.push(code);
        self.chunk.spans.push(span.into());
    }

    #[must_use]
    fn emit_reserve(&mut self, span: impl Into<Span>) -> ReservedOpSpace {
        self.emit(OpCode::Error(BytecodeError::UnpatchedOpCode), span.into());
        let id = self.chunk.code.len() - 1;
        self.patches.insert(id);
        ReservedOpSpace(id)
    }

    fn patch(&mut self, op: ReservedOpSpace, code: OpCode) {
        if !self.patches.remove(&op.0) {
            panic!("Patch for position {} is not registered", op.0)
        }
        self.chunk.code[op.0] = code;
    }

    fn get_or_create_constant_index(&mut self, item: &str) -> ConstantIndex {
        if let Some(index) = self.chunk.strings.iter().position(|x| x == item) {
            return ConstantIndex(index);
        }
        self.chunk.strings.push(item.to_owned());
        ConstantIndex(self.chunk.strings.len() - 1)
    }

    fn check_for_trash(&self) {
        // Each register must be either free, or in a local variable
        for i in 0..self.next_register {
            let register = StackIndex(i);
            assert!(
                self.declared_variables
                    .values()
                    .any(|x| x.register == register)
                    || self.free_registers.iter().any(|x| x.0 == register),
                "Register {} is not released",
                i
            )
        }

        for (ident, var) in self.declared_variables.iter() {
            assert!(
                !self.free_registers.iter().any(|e| e.0 == var.register),
                "Variable {} has its register ${} released",
                ident,
                var.register
            )
        }
    }

    fn ops_length(&self) -> usize {
        self.chunk.code.len()
    }

    fn patch_assignment_spans(&mut self, ops_length: usize, span: Span) {
        if self.ops_length() != ops_length {
            *self.chunk.spans.last_mut().unwrap() = span;
        }

        for x in &self.assignment_spans {
            self.chunk.spans[*x] = span;
        }
    }

    fn mark_assignment_span(&mut self) {
        self.assignment_spans.push(self.ops_length() - 1);
    }

    fn clear_assignment_spans(&mut self) {
        self.assignment_spans.clear();
    }
}

impl<'src> Default for FunctionCompilationContext<'src> {
    fn default() -> Self {
        Self {
            use_locals_map: false,
            declared_variables: Default::default(),
            next_register: 0,
            free_registers: Default::default(),
            patches: Default::default(),
            assignment_spans: vec![],
            chunk: Chunk {
                code: vec![],
                spans: vec![],
                strings: vec![],
                stack_size: 0,
            },
        }
    }
}

struct ReservedOpSpace(usize);

fn compile_body<'src>(body: &Body<'src>, ctx: &mut FunctionCompilationContext<'src>) {
    for (statement, span) in body {
        match statement {
            Statement::Expression(expr) => {
                // Expressions statements without side effects are ignored
                if !can_have_side_effects(&expr.0, ctx) {
                    continue;
                }

                let reg = compile_expressions(expr, None, ctx, false);
                ctx.release_register(reg);
            }
            Statement::Assignment(lhs, rhs) => compile_assignment(lhs, rhs, span, ctx),
            Statement::If(chain, else_body) => {
                compile_if(chain, else_body, span, ctx);
            }
            Statement::While(condition, body) => compile_while(condition, body, span, ctx),
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
    span: &Span,
    ctx: &mut FunctionCompilationContext<'src>,
) {
    match &lhs.0 {
        Expr::Path(path) => match path {
            Path::AnyScope(ident) => {
                if ctx.use_locals_map {
                    todo!("Locals map");
                }
                let mut is_new = false;
                let lhs = ctx.local_register(ident).unwrap_or_else(|| {
                    is_new = true;
                    ctx.get_register()
                });
                let ops_length = ctx.ops_length();
                let _ = compile_expressions(rhs, Some(lhs), ctx, false);
                ctx.patch_assignment_spans(ops_length, *span);

                if is_new {
                    ctx.new_local(ident, lhs)
                }

                ctx.set_can_be_function(ident, can_evaluate_to_function(&rhs.0, ctx));
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

fn compile_if<'src>(
    ifs: &[(Spanned<Spanned<Expr<'src>>>, Body<'src>)],
    else_body: &Option<Body<'src>>,
    _span: &Span,
    ctx: &mut FunctionCompilationContext<'src>,
) -> usize {
    if let Some(condition) = ifs.first() {
        let condition_register = compile_expressions(&condition.0 .0, None, ctx, false);
        ctx.release_if_unused(condition_register);
        let reserve = ctx.emit_reserve(condition.0 .1);

        compile_body(&condition.1, ctx);

        let end_jump = if ifs.len() > 1 || else_body.is_some() {
            Some(ctx.emit_reserve(0..0))
        } else {
            None
        };

        let next = ctx.ops_length();
        let end = compile_if(&ifs[1..], else_body, _span, ctx);
        ctx.patch(reserve, OpCode::JumpIfFalse(condition_register, next));
        if let Some(end_jump) = end_jump {
            ctx.patch(end_jump, OpCode::Jump(end));
        }
    } else if let Some(body) = else_body {
        compile_body(body, ctx);
    }

    ctx.ops_length()
}

fn compile_while<'src>(
    (condition, span): &Spanned<Spanned<Expr<'src>>>,
    body: &Body<'src>,
    _span: &Span,
    ctx: &mut FunctionCompilationContext<'src>,
) {
    let start = ctx.ops_length();
    let condition_register = compile_expressions(condition, None, ctx, false);
    let jump = ctx.emit_reserve(*span);
    ctx.release_if_unused(condition_register);
    compile_body(body, ctx);
    ctx.emit(OpCode::Jump(start), 0..0);
    ctx.patch(
        jump,
        OpCode::JumpIfFalse(condition_register, ctx.ops_length()),
    );
}

#[must_use]
fn compile_expressions<'src>(
    (expression, span): &Spanned<Expr<'src>>,
    register: Option<StackIndex>,
    ctx: &mut FunctionCompilationContext<'src>,
    suppress_call: bool,
) -> StackIndex {
    ctx.clear_assignment_spans();
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
        Expr::Comparison(lhs, comparisons) => {
            compile_comparison_chain(lhs, comparisons, *span, register, ctx)
        }
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
            if suppress_call || !ctx.can_be_function(ident) {
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

fn compile_comparison_chain<'src>(
    lhs: &Spanned<Expr<'src>>,
    chain: &Vec<(Comparison, Spanned<Expr<'src>>)>,
    span: Span,
    register: Option<StackIndex>,
    ctx: &mut FunctionCompilationContext<'src>,
) -> StackIndex {
    let (register, released) = ctx.actualize_and_release_if_unused(register);
    if chain.len() == 1 {
        let (comparison, rhs) = &chain[0];
        let lhs_register = compile_expressions(lhs, None, ctx, false);
        let rhs_register = compile_expressions(rhs, None, ctx, false);
        ctx.emit(
            comparison_op(*comparison, lhs_register, rhs_register, register),
            span,
        );

        ctx.release_if_unused(lhs_register);
        ctx.release_if_unused(rhs_register);
    } else {
        // General algorithm is to evaluate all comparisons in order, merging results into an accumulator
        let accumulator = ctx.get_register();
        ctx.emit(OpCode::SetNumber(accumulator, 1.), 0..0);

        // On each iteration, we compare lhs and rhs, and then transform rhs into the next lhs
        let mut lhs_register = compile_expressions(lhs, None, ctx, false);
        let mut previous = lhs;
        for (comparison, rhs) in chain {
            let rhs_register = compile_expressions(rhs, None, ctx, false);
            ctx.release_if_unused(lhs_register);
            let tmp_register = ctx.get_register();
            ctx.emit(
                comparison_op(*comparison, lhs_register, rhs_register, tmp_register),
                previous.1.start..rhs.1.end,
            );
            ctx.emit(
                OpCode::And {
                    lhs: accumulator,
                    rhs: tmp_register,
                    output: accumulator,
                },
                lhs.1.start..rhs.1.end,
            );
            ctx.release_if_unused(tmp_register);
            ctx.release_if_unused(lhs_register);
            lhs_register = rhs_register;
            previous = rhs;
        }
        ctx.release_if_unused(accumulator);

        if register != accumulator {
            ctx.emit(
                OpCode::Copy {
                    source: accumulator,
                    output: register,
                },
                span,
            );
        }
    }
    if released {
        ctx.take_back_register(register);
    }
    register
}

fn comparison_op(comp: Comparison, lhs: StackIndex, rhs: StackIndex, output: StackIndex) -> OpCode {
    match comp {
        Comparison::Eq => OpCode::Equals { lhs, rhs, output },
        Comparison::NotEq => OpCode::NotEquals { lhs, rhs, output },
        Comparison::Gt => OpCode::GreaterThan { lhs, rhs, output },
        Comparison::Lt => OpCode::LessThan { lhs, rhs, output },
        Comparison::GtEq => OpCode::GreaterOrEquals { lhs, rhs, output },
        Comparison::LtEq => OpCode::LessOrEquals { lhs, rhs, output },
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
        return compile_binary_logic(lhs, rhs, op == BinaryOp::And, span, register, ctx);
    }
    let (output, released) = ctx.actualize_and_release_if_unused(register);
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
    if released {
        ctx.take_back_register(output);
    }
    output
}

fn compile_binary_logic<'src>(
    lhs: &Spanned<Expr<'src>>,
    rhs: &Spanned<Expr<'src>>,
    is_and: bool,
    span: Span,
    register: Option<StackIndex>,
    ctx: &mut FunctionCompilationContext<'src>,
) -> StackIndex {
    let (register, released) = ctx.actualize_and_release_if_unused(register);

    let lhs_register = compile_expressions(lhs, None, ctx, false);

    let jump = ctx.emit_reserve(span);

    let rhs_register = compile_expressions(rhs, None, ctx, false);

    if is_and {
        ctx.emit(
            OpCode::FuzzyAnd {
                output: register,
                lhs: lhs_register,
                rhs: rhs_register,
            },
            span,
        );
    } else {
        ctx.emit(
            OpCode::FuzzyOr {
                output: register,
                lhs: lhs_register,
                rhs: rhs_register,
            },
            span,
        );
    }
    ctx.mark_assignment_span();

    let jump_out = ctx.emit_reserve(0..0);

    if is_and {
        ctx.patch(jump, OpCode::JumpIfFalse(lhs_register, ctx.ops_length()));
        ctx.emit(OpCode::SetNumber(register, 0.), span);
    } else {
        ctx.patch(
            jump,
            OpCode::JumpIfAbsOneOrGreater(lhs_register, ctx.ops_length()),
        );
        ctx.emit(OpCode::SetNumber(register, 1.), span);
    }

    ctx.mark_assignment_span();

    ctx.patch(jump_out, OpCode::Jump(ctx.ops_length()));

    ctx.release_if_unused(lhs_register);
    ctx.release_if_unused(rhs_register);

    if released {
        ctx.take_back_register(register);
    }
    register
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

        let register = ctx.actualize(register);
        let arg = compile_expressions(&args[0], Some(register), ctx, false);
        ctx.emit(
            OpCode::Print {
                value: arg,
                output: register,
            },
            span,
        );
        return register;
    }
    todo!("function calls");
}

fn can_have_side_effects(expr: &Expr, ctx: &FunctionCompilationContext) -> bool {
    match expr {
        // Constant values never have side effects
        Expr::Value(_) => false,
        // Variable access can have side effects when locals are a map
        Expr::Path(_) => true,
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
        Expr::Unary(op, expr) => {
            if op != &UnaryOp::AddressOf {
                can_have_side_effects(&(**expr).0, ctx)
            } else {
                match &(**expr).0 {
                    // @ operator suppresses possible function calls, but does not help when locals
                    // has custom indexer logic
                    Expr::Path(_) => ctx.use_locals_map,
                    Expr::Index(expr, _) => can_have_side_effects(&(**expr).0, ctx),
                    expr => can_have_side_effects(expr, ctx),
                }
            }
        }
        // Function calls can have side effects
        Expr::Call(_, _) => true,
        // Indexing can have side effects
        Expr::ExprIndex(_, _) => true,
        Expr::Index(_, _) => true,
        Expr::Error => ast_err!(),
    }
}

fn can_evaluate_to_function(expr: &Expr, ctx: &FunctionCompilationContext) -> bool {
    match expr {
        Expr::Value(_) => false,
        Expr::Path(path) => match path {
            Path::AnyScope(ident) => ctx.can_be_function(ident),
        },
        Expr::List(_) => false,
        Expr::Map(_) => false,
        Expr::FunctionDefinition(_, _) => true,
        Expr::Comparison(_, _) => false,
        Expr::Binary(_, _, _) => false,
        Expr::Unary(_, rhs) => can_evaluate_to_function(&(**rhs).0, ctx),
        Expr::Call(_, _) => true,
        Expr::ExprIndex(_, _) => true,
        Expr::Index(_, _) => true,
        Expr::Error => ast_err!(),
    }
}

fn expr_iter<'a, 'src>(expr: &'a Expr<'src>) -> impl Iterator<Item = &'a Expr<'src>> {
    ExprIter::<'a, 'src>::new(expr, |e| true)
}

fn expr_iter_filtered<'a, 'src, RecursionFilter: Fn(&'a Expr<'src>) -> bool>(
    expr: &'a Expr<'src>,
    filter: RecursionFilter,
) -> impl Iterator<Item = &'a Expr<'src>> {
    ExprIter::<'a, 'src>::new(expr, filter)
}

struct ExprIter<'a, 'src, RecursionFilter: Fn(&'a Expr<'src>) -> bool> {
    stack: Vec<&'a Expr<'src>>,
    filter: RecursionFilter,
}

impl<'a, 'src, RecursionFilter: Fn(&'a Expr<'src>) -> bool> ExprIter<'a, 'src, RecursionFilter> {
    fn new(expr: &'a Expr<'src>, filter: RecursionFilter) -> Self {
        Self {
            stack: vec![expr],
            filter,
        }
    }

    fn add_all<T: Iterator<Item = &'a Spanned<Expr<'src>>>>(&mut self, items: T) {
        for item in items {
            self.stack.push(&item.0);
        }
    }
    fn add(&mut self, item: &'a Spanned<Expr<'src>>) {
        self.stack.push(&item.0);
    }
}

impl<'a, 'src, RecursionFilter: Fn(&'a Expr<'src>) -> bool> Iterator
    for ExprIter<'a, 'src, RecursionFilter>
{
    type Item = &'a Expr<'src>;
    fn next(&mut self) -> Option<Self::Item> {
        let mut state = self.stack.pop()?;
        while !(self.filter)(state) {
            state = self.stack.pop()?;
        }
        match state {
            Expr::List(items) => self.add_all(items.iter()),
            Expr::Map(items) => items.iter().for_each(|(k, v)| {
                self.add(k);
                self.add(v);
            }),
            Expr::Comparison(lhs, rest) => {
                self.stack.push(&(**lhs).0);
                self.add_all(rest.iter().map(|e| &e.1))
            }
            Expr::Binary(lhs, _, rhs) => {
                self.add(lhs);
                self.add(rhs);
            }
            Expr::Unary(_, expr) => self.add(expr),
            Expr::Call(lhs, args) => {
                self.add(lhs);
                self.add_all(args.iter());
            }
            Expr::ExprIndex(expr, index) => {
                self.add(expr);
                self.add(index);
            }
            Expr::Index(expr, _) => self.add(expr),
            Expr::Value(_) | Expr::Path(_) | Expr::FunctionDefinition(_, _) | Expr::Error => {}
        }
        Some(state)
    }
}
