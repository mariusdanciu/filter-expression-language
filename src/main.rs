use winnow::Parser;

mod lang;

use crate::lang::ast::EvaluatorError;
use crate::lang::ast::Primitives;
use crate::lang::ast::PrimitivesTypes;
use crate::lang::parsers::*;
use std::collections::HashMap;
use winnow::stream::Stateful;
use crate::lang::ast::ParserContext;

fn path_prefix(_ctx: &Context, args: &Vec<Primitives>) -> Result<bool, EvaluatorError> {
    let first = &args[0];
    if let Primitives::String(value) = first
        && args.len() == 1
    {
        return Ok(value == "/api");
    }

    Err(EvaluatorError::InvalidArguments(
        "'path_prefix' - expected string argument".to_string(),
    ))
}

fn method(_ctx: &Context, args: &Vec<Primitives>) -> Result<bool, EvaluatorError> {
    let first = &args[0];
    if let Primitives::String(value) = first
        && args.len() == 1
    {
        return Ok(value == "GET");
    }

    Err(EvaluatorError::InvalidArguments(
        "'method' - expected string argument".to_string(),
    ))
}

fn has_header(  _ctx: &Context, args: &Vec<Primitives>) -> Result<bool, EvaluatorError> {
    let first = &args[0];
    if let Primitives::String(value) = first
        && args.len() == 1
    {
        return Ok(value == "X-API-KEY");
    }

    Err(EvaluatorError::InvalidArguments(
        "'has_header' - expected string argument".to_string(),
    ))
}

fn has_query(_ctx: &Context, args: &Vec<Primitives>) -> Result<bool, EvaluatorError> {
    let first = &args[0];
    if let Primitives::String(value) = first
        && args.len() == 1
    {
        return Ok(value == "version");
    }

    Err(EvaluatorError::InvalidArguments(
        "'has_query' - expected string argument".to_string(),
    ))
}

fn evaluator(ctx: &Context, name: &str, args: &Vec<Primitives>) -> Result<bool, EvaluatorError> {
    match name {
        "path_prefix" => path_prefix(ctx, args),
        "method" => method(ctx, args),
        "has_header" => has_header(ctx, args),
        "has_query" => has_query(ctx, args),
        _ => Err(EvaluatorError::FunctionNotFound(name.to_string())),
    }
}

struct Context;

fn main() {
    let input_str = "path_prefix(\"/v1\") or method(\"GET\") and has_header(\"X-API-KEY\")";
    println!("Input: {}", input_str);

    let mut ctx = ParserContext {
        known_functions: HashMap::new(),
        original_input: input_str.to_string(),
    };
    ctx.known_functions
        .insert("path_prefix".to_string(), vec![PrimitivesTypes::String]);
    ctx.known_functions
        .insert("method".to_string(), vec![PrimitivesTypes::String]);
    ctx.known_functions
        .insert("has_header".to_string(), vec![PrimitivesTypes::String]);
    ctx.known_functions
        .insert("has_query".to_string(), vec![PrimitivesTypes::String]);

    let mut input = Stateful { input: input_str, state: &ctx };

    match expr.parse(input) {
        Ok(result) => {
            println!("✓ Parsed successfully:");
            println!("{:#?}", result);

            let ctx = &Context {};
            match result.eval(ctx, evaluator) {
                Ok(result) => {
                    println!("✓ Evaluated successfully:");
                    println!("{:#?}", result);
                }
                Err(e) => {
                    eprintln!("✗ Evaluation error:");
                    eprintln!("{}", e);
                }
            }
        }
        Err(e) => {
            eprintln!("✗ Parse error:");
            eprintln!("{}", e);
        }
    }
}
