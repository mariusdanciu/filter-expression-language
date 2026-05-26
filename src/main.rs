use winnow::Parser;

mod lang;

use crate::lang::ast::Primitives;
use crate::lang::parsers::*;

fn path_prefix(ctx: &Context, args: &Vec<Primitives>) -> Result<bool, String> {
    let first = &args[0];
    if let Primitives::String(value) = first
        && args.len() == 1
    {
        return Ok(value == "/api");
    }

    Err("'path_prefix' - expected string argument".to_string())
}

fn method(ctx: &Context, args: &Vec<Primitives>) -> Result<bool, String> {
    let first = &args[0];
    if let Primitives::String(value) = first
        && args.len() == 1
    {
        return Ok(value == "GET");
    }

    Err("'method' - expected string argument".to_string())
}

fn has_header(ctx: &Context, args: &Vec<Primitives>) -> Result<bool, String> {
    let first = &args[0];
    if let Primitives::String(value) = first
        && args.len() == 1
    {
        return Ok(value == "X-API-KEY");
    }

    Err("'has_header' - expected string argument".to_string())
}

fn has_query(ctx: &Context, args: &Vec<Primitives>) -> Result<bool, String> {
    let first = &args[0];
    if let Primitives::String(value) = first
        && args.len() == 1
    {
        return Ok(value == "version");
    }

    Err("'has_query' - expected string argument".to_string())
}

fn evaluator(ctx: &Context, name: &str, args: &Vec<Primitives>) -> Result<bool, String> {
    match name {
        "path_prefix" => path_prefix(ctx, args),
        "method" => method(ctx, args),
        "has_header" => has_header(ctx, args),
        "has_query" => has_query(ctx, args),
        _ => Err(format!("Unknown function: {}", name)),
    }
}

struct Context;

fn main() {
    let input =
        "( ( path_prefix(\"/v1\") or method(\"GET\") ) and has_header(\"X-API-KEY\") )";

    match expr.parse(input) {
        Ok(result) => {
            println!("✓ Parsed successfully:");
            println!("{:#?}", result);

            let ctx = &Context{};
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
