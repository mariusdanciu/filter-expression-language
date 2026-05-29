use winnow::Parser;
use winnow::ascii::dec_int;
use winnow::ascii::space0;
use winnow::combinator::*;
use winnow::error::ContextError;
use winnow::error::{StrContext, StrContextValue};
use winnow::stream::Stateful;
use winnow::token::*;

use crate::lang::ast::*;

type Input<'s> = Stateful<&'s str, &'s ParserContext>;
type Result<T> = winnow::Result<T, ContextError>;

pub fn token<'s>(name: &'s str) -> impl Parser<Input<'s>, &'s str, ContextError> {
    delimited(
        space0,        // 1. Consume 0 or more spaces on the left
        literal(name), // 2. Parse the core literal token
        space0,        // 3. Consume 0 or more spaces on the right
    )
}

fn int_literal<'s>(input: &mut Input<'s>) -> Result<Primitives> {
    dec_int.parse_next(input).map(Primitives::Int)
}

fn bool_literal<'s>(input: &mut Input<'s>) -> Result<Primitives> {
    alt((
        token("true").map(|_| Primitives::Bool(true)),
        token("false").map(|_| Primitives::Bool(false)),
    ))
    .parse_next(input)
}

fn string_literal<'s>(input: &mut Input<'s>) -> Result<Primitives> {
    delimited(
        '"',                 // Opening delimiter
        take_till(0.., '"'), // Consume everything until the closing quote
        '"',                 // Closing delimiter
    )
    .map(|s: &'s str| Primitives::String(s.to_string()))
    .context(StrContext::Label("string literal"))
    .context(StrContext::Expected(StrContextValue::Description(
        "quoted string like \"value\"",
    )))
    .parse_next(input)
}

fn local_literal<'s>(input: &mut Input<'s>) -> Result<Primitives> {
    alt((int_literal, bool_literal, string_literal)).parse_next(input)
}

fn seq_literals<'s>(input: &mut Input<'s>) -> Result<Vec<Primitives>> {
    separated(
        0..,           // Expect 0 or more matching items (use 1.. for mandatory)
        local_literal, // What we are looking for
        token(","),    // The separator: comma with optional spaces around it
    )
    .parse_next(input)
}

pub fn func_name<'s>(input: &mut Input<'s>) -> Result<&'s str> {
    take_while(1.., ('a'..='z', 'A'..='Z', '0'..='9', '-', '_')).parse_next(input)
}

pub fn func_call<'s>(input: &mut Input<'s>) -> Result<Box<Ast>> {
    (
        func_name.context(StrContext::Label("function name")),
        token("(").context(StrContext::Expected(StrContextValue::CharLiteral('('))),
        seq_literals,
        token(")").context(StrContext::Expected(StrContextValue::CharLiteral(')'))),
    )
        .map(|(name, _, string, _)| {
            Box::new(Ast::Func {
                name: name.to_string(),
                args: string.into_iter().collect(),
            })
        })
        .context(StrContext::Label("function call"))
        .parse_next(input)
}

// Base term: function call or parenthesized expression
pub fn term<'s>(input: &mut Input<'s>) -> Result<Box<Ast>> {
    (opt(token("not")), alt((parens_expr, func_call)))
        .map(|(not, expr)| match not {
            Some(_) => Box::new(Ast::Not { expr }),
            None => expr,
        })
        .parse_next(input)
}

pub fn parens_expr<'s>(input: &mut Input<'s>) -> Result<Box<Ast>> {
    (
        token("("),
        expr,
        token(")"),
    )
     .map(|(_, expr, _)| expr)
        .parse_next(input)
}

// AND: parse term (token("and") term)*
pub fn and_expr<'s>(input: &mut Input<'s>) -> Result<Box<Ast>> {
    let (first, rest): (Box<Ast>, Vec<(&str, Box<Ast>)>) =
        (term, repeat(0.., (token("and"), term))).parse_next(input)?;

    Ok(rest.into_iter().fold(first, |left, (_, right)| {
        Box::new(Ast::And { left, right })
    }))
}

// OR: parse and_expr (token("or") and_expr)*
pub fn or_expr<'s>(input: &mut Input<'s>) -> Result<Box<Ast>> {
    let (first, rest): (Box<Ast>, Vec<(&str, Box<Ast>)>) =
        (and_expr, repeat(0.., (token("or"), and_expr))).parse_next(input)?;

    Ok(rest.into_iter().fold(first, |left, (_, right)| {
        Box::new(Ast::Or { left, right })
    }))
}

pub fn expr<'s>(input: &mut Input<'s>) -> Result<Box<Ast>> {
    or_expr.parse_next(input)
}
