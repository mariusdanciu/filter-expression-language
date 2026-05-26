use winnow::Parser;
use winnow::ascii::dec_int;
use winnow::ascii::space0;
use winnow::combinator::*;
use winnow::error::{AddContext, ParserError, StrContext, StrContextValue};
use winnow::token::*;

use crate::lang::ast::*;

type Result<T> = winnow::Result<T>;

pub fn token<'s, E>(name: &'s str) -> impl Parser<&'s str, &'s str, E>
where
    E: ParserError<&'s str> + AddContext<&'s str, StrContext>,
{
    delimited(
        space0,        // 1. Consume 0 or more spaces on the left
        literal(name), // 2. Parse the core literal token
        space0,        // 3. Consume 0 or more spaces on the right
    )
}

fn int_literal<'s>(input: &mut &'s str) -> Result<Primitives> {
    dec_int.parse_next(input).map(Primitives::Int)
}

fn bool_literal<'s>(input: &mut &'s str) -> Result<Primitives> {
    alt((
        token("true").map(|_| Primitives::Bool(true)),
        token("false").map(|_| Primitives::Bool(false)),
    ))
    .parse_next(input)
}

fn string_literal<'s>(input: &mut &'s str) -> Result<Primitives> {
    delimited(
        '"',                 // Opening delimiter
        take_till(0.., '"'), // Consume everything until the closing quote
        '"',                 // Closing delimiter
    )
    .map(|s: &'s str| Primitives::String(s.to_string()))
    .context(StrContext::Label("string literal"))
    .context(StrContext::Expected(StrContextValue::Description("quoted string like \"value\"")))
    .parse_next(input)
}

fn local_literal<'s>(input: &mut &'s str) -> Result<Primitives> {
    alt((int_literal, bool_literal, string_literal)).parse_next(input)
}

fn seq_literals<'s>(input: &mut &'s str) -> Result<Vec<Primitives>> {
    separated(
        0..,           // Expect 0 or more matching items (use 1.. for mandatory)
        local_literal, // What we are looking for
        token(","),    // The separator: comma with optional spaces around it
    )
    .parse_next(input)
}

pub fn func_name<'s>(input: &mut &'s str) -> Result<&'s str> {
    take_while(1.., ('a'..='z', 'A'..='Z', '0'..='9', '-', '_')).parse_next(input)
}

pub fn func_call<'s>(input: &mut &str) -> Result<Box<Ast>> {
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

pub fn parens_expr<'s>(input: &mut &'s str) -> Result<Box<Ast>> {
    (
        token("(").context(StrContext::Expected(StrContextValue::CharLiteral('('))),
        bool_expr,
        token(")").context(StrContext::Expected(StrContextValue::CharLiteral(')'))),
    )
        .map(|(_, expr, _)| expr)
        .context(StrContext::Label("parenthesized expression"))
        .parse_next(input)
}

pub fn and_expr<'s>(input: &mut &'s str) -> Result<Box<Ast>> {
    let term_1 = alt((parens_expr, func_call))
        .context(StrContext::Expected(StrContextValue::Description("function call or parenthesized expression")));
    let term_2 = alt((parens_expr, func_call))
        .context(StrContext::Expected(StrContextValue::Description("function call or parenthesized expression")));

    let (first, rest): (Box<Ast>, Vec<_>) =
        (term_1, repeat(0.., (token("and"), term_2)))
            .context(StrContext::Label("AND expression"))
            .parse_next(input)?;

    let result = rest.into_iter().fold(first, |acc, (_, right)| {
        Box::new(Ast::And { left: acc, right })
    });
    Ok(result)
}

pub fn bool_expr<'s>(input: &mut &'s str) -> Result<Box<Ast>> {
    let (first, rest): (Box<Ast>, Vec<_>) =
        (and_expr, repeat(0.., (token("or"), and_expr)))
            .context(StrContext::Label("OR expression"))
            .parse_next(input)?;

    let result = rest.into_iter().fold(first, |acc, (_, right)| {
        Box::new(Ast::Or { left: acc, right })
    });
    Ok(result)
}

pub fn expr<'s>(input: &mut &'s str) -> Result<Box<Ast>> {
    alt((bool_expr, func_call))
        .context(StrContext::Label("expression"))
        .context(StrContext::Expected(StrContextValue::Description("boolean expression or function call")))
        .parse_next(input)
}
