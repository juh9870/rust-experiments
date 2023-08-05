use crate::errors::MsError;
use chumsky::span::SimpleSpan;

pub struct AST<'src>(Body<'src>);

impl<'src> AST<'src> {
    pub fn body(&self) -> &Body<'src> {
        return &self.0;
    }
    pub fn from_body(body: Body<'src>) -> Result<Self, MsError> {
        Ok(Self(body))
    }

    pub fn from_body_unchecked(body: Body<'src>) -> Self {
        Self(body)
    }
}

impl<'src> From<AST<'src>> for Body<'src> {
    fn from(value: AST<'src>) -> Self {
        value.0
    }
}

pub struct DirtyAST<'src>(pub Body<'src>);

pub type Span = SimpleSpan<usize>;
pub type Spanned<T> = (T, Span);

#[derive(Debug, Clone)]
pub enum Statement<'src> {
    Assignment(Spanned<Expr<'src>>, Spanned<Expr<'src>>),
    Expression(Spanned<Expr<'src>>),
    If(
        /* conditions and their bodies for else-if chain */
        Vec<(Spanned<Spanned<Expr<'src>>>, Body<'src>)>,
        /* else */ Option<Body<'src>>,
    ),
    While(Spanned<Spanned<Expr<'src>>>, Body<'src>),
    For(Spanned<&'src str>, Spanned<Expr<'src>>, Body<'src>),
    Break,
    Continue,
    Return(Option<Spanned<Expr<'src>>>),
    Error,
}

pub type Body<'src> = Vec<Spanned<Statement<'src>>>;

#[derive(Debug, Clone)]
pub enum Expr<'src> {
    /// Literal value
    Value(Value<'src>),
    /// Resolving a provided variable path
    Path(Path<'src>),
    /// List expression
    List(Vec<Spanned<Self>>),
    /// Map expressions in (key, value) form
    Map(Vec<(Spanned<Self>, Spanned<Self>)>),
    // Function definition, in (arguments, body) form
    FunctionDefinition(Vec<FunctionArgument<'src>>, Body<'src>),
    // Comparison expression
    Comparison(Box<Spanned<Self>>, Vec<(Comparison, Spanned<Self>)>),
    // Binary expression
    Binary(Box<Spanned<Self>>, BinaryOp, Box<Spanned<Self>>),
    // Unary operator
    Unary(UnaryOp, Box<Spanned<Self>>),
    // Function call
    Call(Box<Spanned<Self>>, Vec<Spanned<Self>>),
    // Indexing with expression
    ExprIndex(Box<Spanned<Self>>, Box<Spanned<Self>>),
    // Indexing operator
    Index(Box<Spanned<Self>>, &'src str),
    /// Error during AST parsing
    Error,
}

impl<'src> Expr<'src> {
    pub fn is_valid_assignment_target(&self) -> bool {
        match self {
            Self::Path(_) => true,
            Self::Index(_, _) => true,
            Self::ExprIndex(_, _) => true,
            _ => false,
        }
    }

    pub fn is_constant(&self) -> bool {
        match self {
            Self::Value(_) => true,
            _ => false,
        }
    }
}

#[derive(Debug, Clone)]
pub enum Value<'src> {
    /// Null literal
    Null,
    /// f64-stored Numeric literal
    Num(f64),
    /// String literal
    String(&'src str),
    /// Boolean literal
    Boolean(bool),
}

/// Enum of all binary operators, listed in order from lowest to highest precedence
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum BinaryOp {
    Or,
    And,
    // < Unary Not
    // < Unary isa
    // < Comparison
    Add,
    Sub,
    Mul,
    Div,
    // < Unary negation
    // < Unary `new`
    // < Unary `address-of`
    Pow,
    // < Indexing dot operator
}

/// Enum of all comparison operators.
/// All comparison operators have the same precedence
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum Comparison {
    Eq,
    NotEq,
    Gt,
    Lt,
    GtEq,
    LtEq,
}

/// Enum of all unary operators, listed in order from lowest to highest precedence
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum UnaryOp {
    Not,
    Isa,
    Neg,
    New,
    AddressOf,
    Error,
}

#[derive(Debug, Clone)]
pub struct FunctionArgument<'src> {
    pub name: Spanned<&'src str>,
    pub default_value: Option<Spanned<Expr<'src>>>,
}

#[derive(Debug, Clone)]
pub enum Path<'src> {
    /// Path to a variable that might be in any visible scope
    AnyScope(&'src str),
}

macro_rules! ast_err {
    () => {
        panic!("AST has an error node!")
    };
}

pub(crate) use ast_err;

impl<'src> Expr<'src> {}
