use winnow::Parser;
use fel::lang::ast::{EvaluatorError, Primitives};
use fel::lang::parsers::*;

// Test context type
struct HttpRequest {
    path: String,
    method: String,
    headers: Vec<String>,
    query_params: Vec<(String, String)>,
}

// Test evaluator function
fn http_evaluator(
    ctx: &HttpRequest,
    name: &str,
    args: &Vec<Primitives>,
) -> Result<bool, EvaluatorError> {
    match name {
        "path_prefix" => {
            if let Some(Primitives::String(prefix)) = args.first() {
                Ok(ctx.path.starts_with(prefix))
            } else {
                Err(EvaluatorError::InvalidArguments(
                    "path_prefix requires a string argument".to_string(),
                ))
            }
        }
        "method" => {
            if let Some(Primitives::String(method)) = args.first() {
                Ok(&ctx.method == method)
            } else {
                Err(EvaluatorError::InvalidArguments(
                    "method requires a string argument".to_string(),
                ))
            }
        }
        "has_header" => {
            if let Some(Primitives::String(header)) = args.first() {
                Ok(ctx.headers.contains(header))
            } else {
                Err(EvaluatorError::InvalidArguments(
                    "has_header requires a string argument".to_string(),
                ))
            }
        }
        "has_query" => {
            if let Some(Primitives::String(key)) = args.first() {
                Ok(ctx.query_params.iter().any(|(k, _)| k == key))
            } else {
                Err(EvaluatorError::InvalidArguments(
                    "has_query requires a string argument".to_string(),
                ))
            }
        }
        _ => Err(EvaluatorError::FunctionNotFound(name.to_string())),
    }
}

// Helper function to create a test request
fn create_request(path: &str, method: &str, headers: Vec<&str>) -> HttpRequest {
    HttpRequest {
        path: path.to_string(),
        method: method.to_string(),
        headers: headers.iter().map(|s| s.to_string()).collect(),
        query_params: vec![],
    }
}

#[test]
fn test_eval_simple_function_true() {
    let input = "path_prefix(\"/api\")";
    let ast = expr.parse(input).unwrap();
    let request = create_request("/api/users", "GET", vec![]);

    let result = ast.eval(&request, http_evaluator).unwrap();
    assert!(result, "path_prefix should match");
}

#[test]
fn test_eval_simple_function_false() {
    let input = "path_prefix(\"/admin\")";
    let ast = expr.parse(input).unwrap();
    let request = create_request("/api/users", "GET", vec![]);

    let result = ast.eval(&request, http_evaluator).unwrap();
    assert!(!result, "path_prefix should not match");
}

#[test]
fn test_eval_method_match() {
    let input = "method(\"GET\")";
    let ast = expr.parse(input).unwrap();
    let request = create_request("/api/users", "GET", vec![]);

    let result = ast.eval(&request, http_evaluator).unwrap();
    assert!(result, "method should match GET");
}

#[test]
fn test_eval_method_no_match() {
    let input = "method(\"POST\")";
    let ast = expr.parse(input).unwrap();
    let request = create_request("/api/users", "GET", vec![]);

    let result = ast.eval(&request, http_evaluator).unwrap();
    assert!(!result, "method should not match POST");
}

#[test]
fn test_eval_and_both_true() {
    let input = "path_prefix(\"/api\") and method(\"GET\")";
    let ast = expr.parse(input).unwrap();
    let request = create_request("/api/users", "GET", vec![]);

    let result = ast.eval(&request, http_evaluator).unwrap();
    assert!(result, "both conditions should be true");
}

#[test]
fn test_eval_and_left_false() {
    let input = "path_prefix(\"/admin\") and method(\"GET\")";
    let ast = expr.parse(input).unwrap();
    let request = create_request("/api/users", "GET", vec![]);

    let result = ast.eval(&request, http_evaluator).unwrap();
    assert!(!result, "left condition is false");
}

#[test]
fn test_eval_and_right_false() {
    let input = "path_prefix(\"/api\") and method(\"POST\")";
    let ast = expr.parse(input).unwrap();
    let request = create_request("/api/users", "GET", vec![]);

    let result = ast.eval(&request, http_evaluator).unwrap();
    assert!(!result, "right condition is false");
}

#[test]
fn test_eval_or_both_true() {
    let input = "path_prefix(\"/api\") or method(\"GET\")";
    let ast = expr.parse(input).unwrap();
    let request = create_request("/api/users", "GET", vec![]);

    let result = ast.eval(&request, http_evaluator).unwrap();
    assert!(result, "both conditions are true");
}

#[test]
fn test_eval_or_left_true() {
    let input = "path_prefix(\"/api\") or method(\"POST\")";
    let ast = expr.parse(input).unwrap();
    let request = create_request("/api/users", "GET", vec![]);

    let result = ast.eval(&request, http_evaluator).unwrap();
    assert!(result, "left condition is true");
}

#[test]
fn test_eval_or_right_true() {
    let input = "path_prefix(\"/admin\") or method(\"GET\")";
    let ast = expr.parse(input).unwrap();
    let request = create_request("/api/users", "GET", vec![]);

    let result = ast.eval(&request, http_evaluator).unwrap();
    assert!(result, "right condition is true");
}

#[test]
fn test_eval_or_both_false() {
    let input = "path_prefix(\"/admin\") or method(\"POST\")";
    let ast = expr.parse(input).unwrap();
    let request = create_request("/api/users", "GET", vec![]);

    let result = ast.eval(&request, http_evaluator).unwrap();
    assert!(!result, "both conditions are false");
}

#[test]
fn test_eval_complex_expression() {
    let input = "(path_prefix(\"/api\") or method(\"POST\")) and has_header(\"X-API-KEY\")";
    let ast = expr.parse(input).unwrap();
    let request = create_request("/api/users", "GET", vec!["X-API-KEY", "Content-Type"]);

    let result = ast.eval(&request, http_evaluator).unwrap();
    assert!(result, "complex expression should evaluate to true");
}

#[test]
fn test_eval_complex_expression_false() {
    let input = "(path_prefix(\"/api\") or method(\"POST\")) and has_header(\"X-API-KEY\")";
    let ast = expr.parse(input).unwrap();
    let request = create_request("/api/users", "GET", vec!["Content-Type"]);

    let result = ast.eval(&request, http_evaluator).unwrap();
    assert!(!result, "complex expression should evaluate to false (missing header)");
}

#[test]
fn test_eval_nested_and_or() {
    let input = "path_prefix(\"/api\") and (method(\"GET\") or method(\"POST\"))";
    let ast = expr.parse(input).unwrap();
    let request = create_request("/api/users", "POST", vec![]);

    let result = ast.eval(&request, http_evaluator).unwrap();
    assert!(result, "nested AND/OR should evaluate to true");
}

#[test]
fn test_eval_multiple_or_chain() {
    let input = "method(\"GET\") or method(\"POST\") or method(\"PUT\")";
    let ast = expr.parse(input).unwrap();
    let request = create_request("/api/users", "PUT", vec![]);

    let result = ast.eval(&request, http_evaluator).unwrap();
    assert!(result, "multiple OR chain should match PUT");
}

#[test]
fn test_eval_multiple_and_chain() {
    let input = "path_prefix(\"/api\") and method(\"GET\") and has_header(\"X-API-KEY\")";
    let ast = expr.parse(input).unwrap();
    let request = create_request("/api/users", "GET", vec!["X-API-KEY"]);

    let result = ast.eval(&request, http_evaluator).unwrap();
    assert!(result, "multiple AND chain should all match");
}

#[test]
fn test_eval_unknown_function_error() {
    let input = "unknown_func(\"test\")";
    let ast = expr.parse(input).unwrap();
    let request = create_request("/api/users", "GET", vec![]);

    let result = ast.eval(&request, http_evaluator);
    assert!(result.is_err(), "should error on unknown function");

    match result.unwrap_err() {
        EvaluatorError::FunctionNotFound(name) => {
            assert_eq!(name, "unknown_func");
        }
        _ => panic!("Expected FunctionNotFound error"),
    }
}

#[test]
fn test_eval_with_headers() {
    let input = "has_header(\"Authorization\") and has_header(\"Content-Type\")";
    let ast = expr.parse(input).unwrap();
    let request = create_request(
        "/api/users",
        "POST",
        vec!["Authorization", "Content-Type"],
    );

    let result = ast.eval(&request, http_evaluator).unwrap();
    assert!(result, "both headers should be present");
}

#[test]
fn test_eval_missing_header() {
    let input = "has_header(\"Authorization\")";
    let ast = expr.parse(input).unwrap();
    let request = create_request("/api/users", "GET", vec!["Content-Type"]);

    let result = ast.eval(&request, http_evaluator).unwrap();
    assert!(!result, "Authorization header should be missing");
}

#[test]
fn test_eval_operator_precedence() {
    // path_prefix OR (method AND has_header)
    let input = "path_prefix(\"/admin\") or method(\"POST\") and has_header(\"X-API-KEY\")";
    let ast = expr.parse(input).unwrap();

    // Case 1: path_prefix matches, so should be true regardless of right side
    let request1 = create_request("/admin/users", "GET", vec![]);
    let result1 = ast.eval(&request1, http_evaluator).unwrap();
    assert!(result1, "left side of OR is true");

    // Case 2: path_prefix doesn't match, but method AND header both match
    let request2 = create_request("/api/users", "POST", vec!["X-API-KEY"]);
    let result2 = ast.eval(&request2, http_evaluator).unwrap();
    assert!(result2, "right side AND is true");

    // Case 3: path_prefix doesn't match, method matches but header missing
    let request3 = create_request("/api/users", "POST", vec![]);
    let result3 = ast.eval(&request3, http_evaluator).unwrap();
    assert!(!result3, "right side AND has one false");
}

#[test]
fn test_eval_with_query_params() {
    let input = "has_query(\"version\")";
    let ast = expr.parse(input).unwrap();

    let mut request = create_request("/api/users", "GET", vec![]);
    request.query_params.push(("version".to_string(), "1.0".to_string()));

    let result = ast.eval(&request, http_evaluator).unwrap();
    assert!(result, "query param should be found");
}
