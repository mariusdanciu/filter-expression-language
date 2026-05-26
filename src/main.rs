use winnow::Parser;

mod lang;

use crate::lang::ast::Primitives;
use crate::lang::parsers::*;

fn path_prefix(args: &Vec<Primitives>) -> Result<bool, String> {
    let first = &args[0];
    if let Primitives::String(value) = first
        && args.len() == 1
    {
        return Ok(value == "/api");
    }

    Err("'path_prefix' - expected string argument".to_string())
}

fn method(args: &Vec<Primitives>) -> Result<bool, String> {
    let first = &args[0];
    if let Primitives::String(value) = first
        && args.len() == 1
    {
        return Ok(value == "GET");
    }

    Err("'method' - expected string argument".to_string())
}

fn has_header(args: &Vec<Primitives>) -> Result<bool, String> {
    let first = &args[0];
    if let Primitives::String(value) = first
        && args.len() == 1
    {
        return Ok(value == "X-API-KEY");
    }

    Err("'has_header' - expected string argument".to_string())
}

fn has_query(args: &Vec<Primitives>) -> Result<bool, String> {
    let first = &args[0];
    if let Primitives::String(value) = first
        && args.len() == 1
    {
        return Ok(value == "/version");
    }

    Err("'has_query' - expected string argument".to_string())
}

fn evaluator(name: &str, args: &Vec<Primitives>) -> Result<bool, String> {
    match name {
        "path_prefix" => path_prefix(args),
        "method" => method(args),
        "has_header" => has_header(args),
        "has_query" => has_query(args),
        _ => Err(format!("Unknown function: {}", name)),
    }
}

fn main() {
    let input =
        "( ( path_prefix(\"/v1\", \"/v2\") or method(\"POST\") ) and has_header(\"X-API-KEY\") )";

    match expr.parse(input) {
        Ok(result) => {
            println!("✓ Parsed successfully:");
            println!("{:#?}", result);

            match result.eval(evaluator) {
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
