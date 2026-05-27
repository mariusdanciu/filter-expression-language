use winnow::Parser;
use fel::lang::parsers::*;
use fel::lang::ast::{Ast, Primitives};

#[test]
fn test_simple_function_call() {
    let mut input = "path_prefix(\"/api\")";
    let result = expr.parse_next(&mut input).expect("Failed to parse simple function call");

    match *result {
        Ast::Func { name, args } => {
            assert_eq!(name, "path_prefix");
            assert_eq!(args.len(), 1);
            match &args[0] { Primitives::String(s) => assert_eq!(s, "/api"), _ => panic!("Expected String") }
        }
        _ => panic!("Expected Func, got {:?}", result),
    }
}

#[test]
fn test_or_expression() {
    let mut input = "path_prefix(\"/api\") or method(\"GET\")";
    let result = expr.parse_next(&mut input).expect("Failed to parse OR expression");

    match *result {
        Ast::Or { left, right } => {
            match left.as_ref() {
                Ast::Func { name, args } => {
                    assert_eq!(name, "path_prefix");
                    match &args[0] { Primitives::String(s) => assert_eq!(s, "/api"), _ => panic!("Expected String") }
                }
                _ => panic!("Expected Func on left side"),
            }
            match right.as_ref() {
                Ast::Func { name, args } => {
                    assert_eq!(name, "method");
                    match &args[0] { Primitives::String(s) => assert_eq!(s, "GET"), _ => panic!("Expected String") }
                }
                _ => panic!("Expected Func on right side"),
            }
        }
        _ => panic!("Expected Or, got {:?}", result),
    }
}

#[test]
fn test_and_expression() {
    let mut input = "method(\"POST\") and has_header(\"Content-Type\")";
    let result = expr.parse_next(&mut input).expect("Failed to parse AND expression");

    match *result {
        Ast::And { left, right } => {
            match left.as_ref() {
                Ast::Func { name, args } => {
                    assert_eq!(name, "method");
                    match &args[0] { Primitives::String(s) => assert_eq!(s, "POST"), _ => panic!("Expected String") }
                }
                _ => panic!("Expected Func on left side"),
            }
            match right.as_ref() {
                Ast::Func { name, args } => {
                    assert_eq!(name, "has_header");
                    match &args[0] { Primitives::String(s) => assert_eq!(s, "Content-Type"), _ => panic!("Expected String") }
                }
                _ => panic!("Expected Func on right side"),
            }
        }
        _ => panic!("Expected And, got {:?}", result),
    }
}

#[test]
fn test_combined_and_or() {
    let mut input = "path_prefix(\"/api\") or method(\"GET\") and has_header(\"X-API-KEY\")";
    let result = expr.parse_next(&mut input).expect("Failed to parse combined AND/OR");

    // Should be: OR(path_prefix, AND(method, has_header))
    match *result {
        Ast::Or { left, right } => {
            // Left: path_prefix
            match left.as_ref() {
                Ast::Func { name, .. } => assert_eq!(name, "path_prefix"),
                _ => panic!("Expected Func on left of OR"),
            }
            // Right: AND expression
            match right.as_ref() {
                Ast::And { left: and_left, right: and_right } => {
                    match and_left.as_ref() {
                        Ast::Func { name, .. } => assert_eq!(name, "method"),
                        _ => panic!("Expected Func on left of AND"),
                    }
                    match and_right.as_ref() {
                        Ast::Func { name, .. } => assert_eq!(name, "has_header"),
                        _ => panic!("Expected Func on right of AND"),
                    }
                }
                _ => panic!("Expected And on right of OR"),
            }
        }
        _ => panic!("Expected Or at root"),
    }
}

#[test]
fn test_parenthesized_expression() {
    let mut input = "(path_prefix(\"/api\") or method(\"GET\")) and has_header(\"X-API-KEY\")";
    let result = expr.parse_next(&mut input).expect("Failed to parse parenthesized expression");

    // Should be: AND(OR(path_prefix, method), has_header)
    match *result {
        Ast::And { left, right } => {
            // Left: OR expression
            match left.as_ref() {
                Ast::Or { left: or_left, right: or_right } => {
                    match or_left.as_ref() {
                        Ast::Func { name, .. } => assert_eq!(name, "path_prefix"),
                        _ => panic!("Expected Func on left of OR"),
                    }
                    match or_right.as_ref() {
                        Ast::Func { name, .. } => assert_eq!(name, "method"),
                        _ => panic!("Expected Func on right of OR"),
                    }
                }
                _ => panic!("Expected Or on left of AND"),
            }
            // Right: has_header
            match right.as_ref() {
                Ast::Func { name, .. } => assert_eq!(name, "has_header"),
                _ => panic!("Expected Func on right of AND"),
            }
        }
        _ => panic!("Expected And at root"),
    }
}

#[test]
fn test_nested_parentheses() {
    let mut input = "((path_prefix(\"/api\") or method(\"GET\")) and has_header(\"X-API-KEY\"))";
    let result = expr.parse_next(&mut input).expect("Failed to parse nested parentheses");

    // Structure should be same as test_parenthesized_expression
    // AND(OR(path_prefix, method), has_header)
    match *result {
        Ast::And { left, right } => {
            assert!(matches!(left.as_ref(), Ast::Or { .. }), "Left should be Or");
            assert!(matches!(right.as_ref(), Ast::Func { .. }), "Right should be Func");
        }
        _ => panic!("Expected And at root"),
    }
}

#[test]
fn test_multiple_or_chain() {
    let mut input = "method(\"GET\") or method(\"POST\") or method(\"PUT\")";
    let result = expr.parse_next(&mut input).expect("Failed to parse multiple OR chain");

    // Should be: OR(GET, OR(POST, PUT)) - right-associative
    match *result {
        Ast::Or { left, right } => {
            // Left: GET
            match left.as_ref() {
                Ast::Func { args, .. } => match &args[0] { Primitives::String(s) => assert_eq!(s, "GET"), _ => panic!("Expected String") },
                _ => panic!("Expected Func on left"),
            }
            // Right: nested OR
            match right.as_ref() {
                Ast::Or { left: inner_left, right: inner_right } => {
                    match inner_left.as_ref() {
                        Ast::Func { args, .. } => match &args[0] { Primitives::String(s) => assert_eq!(s, "POST"), _ => panic!("Expected String") },
                        _ => panic!("Expected Func"),
                    }
                    match inner_right.as_ref() {
                        Ast::Func { args, .. } => match &args[0] { Primitives::String(s) => assert_eq!(s, "PUT"), _ => panic!("Expected String") },
                        _ => panic!("Expected Func"),
                    }
                }
                _ => panic!("Expected nested Or on right"),
            }
        }
        _ => panic!("Expected Or at root"),
    }
}

#[test]
fn test_multiple_and_chain() {
    let mut input = "has_header(\"Auth\") and has_header(\"Content-Type\") and has_header(\"Accept\")";
    let result = expr.parse_next(&mut input).expect("Failed to parse multiple AND chain");

    // Should be: AND(Auth, AND(Content-Type, Accept)) - right-associative
    match *result {
        Ast::And { left, right } => {
            // Left: Auth
            match left.as_ref() {
                Ast::Func { args, .. } => match &args[0] { Primitives::String(s) => assert_eq!(s, "Auth"), _ => panic!("Expected String") },
                _ => panic!("Expected Func on left"),
            }
            // Right: nested AND
            match right.as_ref() {
                Ast::And { left: inner_left, right: inner_right } => {
                    match inner_left.as_ref() {
                        Ast::Func { args, .. } => match &args[0] { Primitives::String(s) => assert_eq!(s, "Content-Type"), _ => panic!("Expected String") },
                        _ => panic!("Expected Func"),
                    }
                    match inner_right.as_ref() {
                        Ast::Func { args, .. } => match &args[0] { Primitives::String(s) => assert_eq!(s, "Accept"), _ => panic!("Expected String") },
                        _ => panic!("Expected Func"),
                    }
                }
                _ => panic!("Expected nested And on right"),
            }
        }
        _ => panic!("Expected And at root"),
    }
}

#[test]
fn test_complex_nested_expression() {
    let mut input = "((path_prefix(\"/api\", \"/v1\") or method(\"GET\")) and has_header(\"X-API-KEY\")) or has_query(\"/version\", \"1.0.0\")";
    let result = expr.parse_next(&mut input).expect("Failed to parse complex expression");

    // Should be: OR(AND(OR(path_prefix, method), has_header), has_query)
    match *result {
        Ast::Or { left, right } => {
            // Left: AND expression
            match left.as_ref() {
                Ast::And { left: and_left, right: and_right } => {
                    // Left of AND: OR expression
                    match and_left.as_ref() {
                        Ast::Or { left: or_left, right: or_right } => {
                            match or_left.as_ref() {
                                Ast::Func { name, args } => {
                                    assert_eq!(name, "path_prefix");
                                    assert_eq!(args.len(), 2);
                                    match &args[0] { Primitives::String(s) => assert_eq!(s, "/api"), _ => panic!("Expected String") }
                                    match &args[1] { Primitives::String(s) => assert_eq!(s, "/v1"), _ => panic!("Expected String") }
                                }
                                _ => panic!("Expected Func"),
                            }
                            match or_right.as_ref() {
                                Ast::Func { name, args } => {
                                    assert_eq!(name, "method");
                                    match &args[0] { Primitives::String(s) => assert_eq!(s, "GET"), _ => panic!("Expected String") }
                                }
                                _ => panic!("Expected Func"),
                            }
                        }
                        _ => panic!("Expected Or"),
                    }
                    // Right of AND: has_header
                    match and_right.as_ref() {
                        Ast::Func { name, args } => {
                            assert_eq!(name, "has_header");
                            match &args[0] { Primitives::String(s) => assert_eq!(s, "X-API-KEY"), _ => panic!("Expected String") }
                        }
                        _ => panic!("Expected Func"),
                    }
                }
                _ => panic!("Expected And on left of outer Or"),
            }
            // Right: has_query
            match right.as_ref() {
                Ast::Func { name, args } => {
                    assert_eq!(name, "has_query");
                    assert_eq!(args.len(), 2);
                    match &args[0] { Primitives::String(s) => assert_eq!(s, "/version"), _ => panic!("Expected String") }
                    match &args[1] { Primitives::String(s) => assert_eq!(s, "1.0.0"), _ => panic!("Expected String") }
                }
                _ => panic!("Expected Func on right of outer Or"),
            }
        }
        _ => panic!("Expected Or at root"),
    }
}

#[test]
fn test_function_with_multiple_args() {
    let mut input = "path_prefix(\"/api\", \"/v1\", \"/v2\", \"/admin\")";
    let result = expr.parse_next(&mut input).expect("Failed to parse function with multiple args");

    match *result {
        Ast::Func { name, args } => {
            assert_eq!(name, "path_prefix");
            assert_eq!(args.len(), 4);
            match &args[0] { Primitives::String(s) => assert_eq!(s, "/api"), _ => panic!("Expected String") }
            match &args[1] { Primitives::String(s) => assert_eq!(s, "/v1"), _ => panic!("Expected String") }
            match &args[2] { Primitives::String(s) => assert_eq!(s, "/v2"), _ => panic!("Expected String") }
            match &args[3] { Primitives::String(s) => assert_eq!(s, "/admin"), _ => panic!("Expected String") }
        }
        _ => panic!("Expected Func, got {:?}", result),
    }
}
