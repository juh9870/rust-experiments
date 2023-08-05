use std::str::FromStr;

use crate::ast::{Span, Spanned};
use chumsky::prelude::*;
use derive_more::Display;
use strum_macros::{AsRefStr, Display as StrumDisplay, EnumString, EnumVariantNames};

#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq, Display)]
pub enum Token<'src> {
    #[display(fmt = "{}", .0)]
    Keyword(Keyword),
    #[display(fmt = "{}", .0)]
    Number(&'src str),
    #[display(fmt = "{}", .0)]
    String(&'src str),
    #[display(fmt = "{}", .0)]
    Identifier(&'src str),
    #[display(fmt = "=")]
    OpAssign,
    #[display(fmt = "+")]
    OpPlus,
    #[display(fmt = "-")]
    OpMinus,
    #[display(fmt = "*")]
    OpTimes,
    #[display(fmt = "/")]
    OpDivide,
    #[display(fmt = "%")]
    OpMod,
    #[display(fmt = "^")]
    OpPower,
    #[display(fmt = "==")]
    OpEqual,
    #[display(fmt = "!=")]
    OpNotEqual,
    #[display(fmt = ">")]
    OpGreater,
    #[display(fmt = ">=")]
    OpGreatEqual,
    #[display(fmt = "<")]
    OpLesser,
    #[display(fmt = "<=")]
    OpLessEqual,
    #[display(fmt = "+=")]
    OpAssignPlus,
    #[display(fmt = "-=")]
    OpAssignMinus,
    #[display(fmt = "*=")]
    OpAssignTimes,
    #[display(fmt = "/=")]
    OpAssignDivide,
    #[display(fmt = "%=")]
    OpAssignMod,
    #[display(fmt = "^=")]
    OpAssignPower,
    #[display(fmt = "(")]
    LParen,
    #[display(fmt = ")")]
    RParen,
    #[display(fmt = "[")]
    LSquare,
    #[display(fmt = "]")]
    RSquare,
    #[display(fmt = "{{")]
    LCurly,
    #[display(fmt = "}}")]
    RCurly,
    #[display(fmt = "@")]
    AddressOf,
    #[display(fmt = ",")]
    Comma,
    #[display(fmt = ".")]
    Dot,
    #[display(fmt = ":")]
    Colon,
    #[display(fmt = "{}", .0)]
    Comment(&'src str),
    #[display(fmt = "\n")]
    EOL,
    #[display(fmt = ";")]
    Semicolon,

    // Unsupported operators. Parsing them for better error reporting
    #[display(fmt = "++")]
    OpIncrement,
    #[display(fmt = "--")]
    OpDecrement,
}

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, EnumString, EnumVariantNames, AsRefStr, StrumDisplay,
)]
#[strum(serialize_all = "lowercase")]
pub enum Keyword {
    // Control flow
    Break,
    Continue,
    End,
    // If
    If,
    Then,
    Else,
    // For
    For,
    In,
    // Others
    While,
    Function,
    Isa,
    New,
    Null,
    Return,
    And,
    Or,
    Not,
    True,
    False,
    // Reserved but unused
    Repeat,
}

impl<'src> From<Keyword> for Token<'src> {
    fn from(value: Keyword) -> Self {
        value.token()
    }
}

impl Keyword {
    pub const fn token<'src>(&self) -> Token<'src> {
        Token::Keyword(*self)
    }
}

pub const UNICODE_FALLOFF: char = '\u{009F}';

#[must_use]
pub fn ident<
    'a,
    I: chumsky::input::ValueInput<'a> + chumsky::input::StrInput<'a, C>,
    C: text::Char,
    E: extra::ParserExtra<'a, I>,
>() -> impl Parser<'a, I, &'a C::Str, E> + Copy + Clone {
    any()
        // Use try_map over filter to get a better error on failure
        .try_map(|c: C, span| {
            if c.to_char().is_ascii_alphabetic()
                || c.to_char() == '_'
                || c.to_char() > UNICODE_FALLOFF
            {
                Ok(c)
            } else {
                Err(chumsky::error::Error::expected_found(
                    [],
                    Some(chumsky::util::MaybeRef::Val(c)),
                    span,
                ))
            }
        })
        .then(
            any()
                // This error never appears due to `repeated` so can use `filter`
                .filter(|c: &C| {
                    c.to_char().is_ascii_alphanumeric()
                        || c.to_char() == '_'
                        || c.to_char() > UNICODE_FALLOFF
                })
                .repeated(),
        )
        .slice()
}

pub fn lexer<'src>(
) -> impl Parser<'src, &'src str, Vec<Spanned<Token<'src>>>, extra::Err<Rich<'src, char, Span>>> {
    let digits = text::digits(10).slice();

    let frac = just('.').then(digits.or_not());

    let exp = just('e')
        .or(just('E'))
        .then(one_of("+-").or_not())
        .then(digits);

    let number = just('-')
        .or_not()
        .then(digits)
        .then(frac.or_not())
        .then(exp.or_not())
        .map_slice(Token::Number)
        .boxed();

    let string = just('"')
        .ignore_then(
            none_of('"')
                .repeated()
                .separated_by(just("\"\""))
                .map_slice(Token::String),
        )
        .then_ignore(just('"'))
        .boxed();

    let identifier = ident()
        .map_slice(|slice| {
            Keyword::from_str(slice)
                .map(Token::Keyword)
                .unwrap_or_else(|_| Token::Identifier(slice))
        })
        .boxed();

    let operator = choice((
        // Parsing unsupported operators first
        just(Token::OpIncrement.to_string()).to(Token::OpIncrement),
        just(Token::OpDecrement.to_string()).to(Token::OpDecrement),
        // Thenregular operators
        just(Token::OpEqual.to_string()).to(Token::OpEqual),
        just(Token::OpNotEqual.to_string()).to(Token::OpNotEqual),
        just(Token::OpGreatEqual.to_string()).to(Token::OpGreatEqual),
        just(Token::OpLessEqual.to_string()).to(Token::OpLessEqual),
        just(Token::OpAssignPlus.to_string()).to(Token::OpAssignPlus),
        just(Token::OpAssignMinus.to_string()).to(Token::OpAssignMinus),
        just(Token::OpAssignTimes.to_string()).to(Token::OpAssignTimes),
        just(Token::OpAssignDivide.to_string()).to(Token::OpAssignDivide),
        just(Token::OpAssignMod.to_string()).to(Token::OpAssignMod),
        just(Token::OpAssignPower.to_string()).to(Token::OpAssignPower),
        just(Token::OpPlus.to_string()).to(Token::OpPlus),
        just(Token::OpMinus.to_string()).to(Token::OpMinus),
        just(Token::OpTimes.to_string()).to(Token::OpTimes),
        just(Token::OpDivide.to_string()).to(Token::OpDivide),
        just(Token::OpMod.to_string()).to(Token::OpMod),
        just(Token::OpPower.to_string()).to(Token::OpPower),
        just(Token::OpAssign.to_string()).to(Token::OpAssign),
        just(Token::OpLesser.to_string()).to(Token::OpLesser),
        just(Token::OpGreater.to_string()).to(Token::OpGreater),
    ))
    .boxed();

    let parentheses = choice((
        just('(').to(Token::LParen),
        just(')').to(Token::RParen),
        just('[').to(Token::LSquare),
        just(']').to(Token::RSquare),
        just('{').to(Token::LCurly),
        just('}').to(Token::RCurly),
    ))
    .boxed();

    let others = choice((
        just('.').to(Token::Dot),
        just(',').to(Token::Comma),
        just(':').to(Token::Colon),
        just('@').to(Token::AddressOf),
    ))
    .boxed();

    let comment = just("//")
        .ignore_then(
            any()
                .and_is(just('\n').not())
                .repeated()
                .map_slice(Token::Comment),
        )
        .boxed();

    let quiet_line_break = just("\n").to(Token::EOL);
    let loud_line_break = just(";").to(Token::Semicolon);

    let token = number
        .or(string)
        .or(identifier)
        .or(comment)
        .or(quiet_line_break)
        .or(loud_line_break)
        .or(operator)
        .or(parentheses)
        .or(others);

    token
        .map_with_span(|tok, span| (tok, span))
        .padded_by(
            any()
                .filter(|c: &char| c.is_whitespace() && *c != '\n')
                .ignored()
                .repeated(),
        )
        // .recover_with(skip_then_retry_until(any().ignored(), end()))
        .repeated()
        .collect()
        .then_ignore(end())
}
