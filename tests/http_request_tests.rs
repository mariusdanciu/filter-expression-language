use fel::lang::ast::{EvaluatorError, ParserContext, Primitives, PrimitivesTypes};
use fel::lang::parsers::expr;
use std::collections::HashMap;
use winnow::Parser;
use winnow::stream::Stateful;

#[derive(Debug)]
struct HttpRequest {
    method: String,
    path: String,
    headers: HashMap<String, String>,
    query_params: HashMap<String, String>,
}

fn path_prefix(req: &HttpRequest, args: &Vec<Primitives>) -> Result<bool, EvaluatorError> {
    if let Some(Primitives::String(prefix)) = args.get(0) {
        return Ok(req.path.starts_with(prefix));
    }
    Err(EvaluatorError::InvalidArguments(
        "path_prefix expects a string argument".to_string(),
    ))
}

fn method(req: &HttpRequest, args: &Vec<Primitives>) -> Result<bool, EvaluatorError> {
    if let Some(Primitives::String(m)) = args.get(0) {
        return Ok(&req.method == m);
    }
    Err(EvaluatorError::InvalidArguments(
        "method expects a string argument".to_string(),
    ))
}

fn has_header(req: &HttpRequest, args: &Vec<Primitives>) -> Result<bool, EvaluatorError> {
    if let Some(Primitives::String(header_name)) = args.get(0) {
        return Ok(req.headers.contains_key(header_name));
    }
    Err(EvaluatorError::InvalidArguments(
        "has_header expects a string argument".to_string(),
    ))
}

fn has_query(req: &HttpRequest, args: &Vec<Primitives>) -> Result<bool, EvaluatorError> {
    if let Some(Primitives::String(param_name)) = args.get(0) {
        return Ok(req.query_params.contains_key(param_name));
    }
    Err(EvaluatorError::InvalidArguments(
        "has_query expects a string argument".to_string(),
    ))
}

fn header_equals(req: &HttpRequest, args: &Vec<Primitives>) -> Result<bool, EvaluatorError> {
    if let (Some(Primitives::String(header_name)), Some(Primitives::String(expected_value))) =
        (args.get(0), args.get(1))
    {
        return Ok(req.headers.get(header_name).map_or(false, |v| v == expected_value));
    }
    Err(EvaluatorError::InvalidArguments(
        "header_equals expects two string arguments".to_string(),
    ))
}

fn query_equals(req: &HttpRequest, args: &Vec<Primitives>) -> Result<bool, EvaluatorError> {
    if let (Some(Primitives::String(param_name)), Some(Primitives::String(expected_value))) =
        (args.get(0), args.get(1))
    {
        return Ok(req.query_params.get(param_name).map_or(false, |v| v == expected_value));
    }
    Err(EvaluatorError::InvalidArguments(
        "query_equals expects two string arguments".to_string(),
    ))
}

fn evaluator(
    req: &HttpRequest,
    name: &str,
    args: &Vec<Primitives>,
) -> Result<bool, EvaluatorError> {
    match name {
        "path_prefix" => path_prefix(req, args),
        "method" => method(req, args),
        "has_header" => has_header(req, args),
        "has_query" => has_query(req, args),
        "header_equals" => header_equals(req, args),
        "query_equals" => query_equals(req, args),
        _ => Err(EvaluatorError::FunctionNotFound(name.to_string())),
    }
}

fn parse_and_eval(input: &str, req: &HttpRequest) -> Result<bool, String> {
    let mut ctx = ParserContext {
        known_functions: HashMap::new(),
        original_input: input.to_string(),
    };
    ctx.known_functions
        .insert("path_prefix".to_string(), vec![PrimitivesTypes::String]);
    ctx.known_functions
        .insert("method".to_string(), vec![PrimitivesTypes::String]);
    ctx.known_functions
        .insert("has_header".to_string(), vec![PrimitivesTypes::String]);
    ctx.known_functions
        .insert("has_query".to_string(), vec![PrimitivesTypes::String]);
    ctx.known_functions.insert(
        "header_equals".to_string(),
        vec![PrimitivesTypes::String, PrimitivesTypes::String],
    );
    ctx.known_functions.insert(
        "query_equals".to_string(),
        vec![PrimitivesTypes::String, PrimitivesTypes::String],
    );

    let input_state = Stateful {
        input,
        state: &ctx,
    };

    let ast = expr.parse(input_state).map_err(|e| format!("{:?}", e))?;
    ast.eval(req, evaluator).map_err(|e| e.to_string())
}

#[test]
fn test_simple_method_get() {
    let req = HttpRequest {
        method: "GET".to_string(),
        path: "/api/users".to_string(),
        headers: HashMap::new(),
        query_params: HashMap::new(),
    };

    assert_eq!(parse_and_eval("method(\"GET\")", &req), Ok(true));
    assert_eq!(parse_and_eval("method(\"POST\")", &req), Ok(false));
}

#[test]
fn test_simple_path_prefix() {
    let req = HttpRequest {
        method: "GET".to_string(),
        path: "/api/users".to_string(),
        headers: HashMap::new(),
        query_params: HashMap::new(),
    };

    assert_eq!(parse_and_eval("path_prefix(\"/api\")", &req), Ok(true));
    assert_eq!(parse_and_eval("path_prefix(\"/admin\")", &req), Ok(false));
}

#[test]
fn test_and_operator() {
    let req = HttpRequest {
        method: "GET".to_string(),
        path: "/api/users".to_string(),
        headers: HashMap::new(),
        query_params: HashMap::new(),
    };

    assert_eq!(
        parse_and_eval("method(\"GET\") and path_prefix(\"/api\")", &req),
        Ok(true)
    );
    assert_eq!(
        parse_and_eval("method(\"POST\") and path_prefix(\"/api\")", &req),
        Ok(false)
    );
}

#[test]
fn test_or_operator() {
    let req = HttpRequest {
        method: "GET".to_string(),
        path: "/api/users".to_string(),
        headers: HashMap::new(),
        query_params: HashMap::new(),
    };

    assert_eq!(
        parse_and_eval("method(\"POST\") or method(\"GET\")", &req),
        Ok(true)
    );
    assert_eq!(
        parse_and_eval("method(\"POST\") or method(\"PUT\")", &req),
        Ok(false)
    );
}

#[test]
fn test_not_operator() {
    let req = HttpRequest {
        method: "GET".to_string(),
        path: "/api/users".to_string(),
        headers: HashMap::new(),
        query_params: HashMap::new(),
    };

    assert_eq!(parse_and_eval("not method(\"POST\")", &req), Ok(true));
    assert_eq!(parse_and_eval("not method(\"GET\")", &req), Ok(false));
}

#[test]
fn test_has_header() {
    let mut headers = HashMap::new();
    headers.insert("Authorization".to_string(), "Bearer token123".to_string());
    headers.insert("Content-Type".to_string(), "application/json".to_string());

    let req = HttpRequest {
        method: "POST".to_string(),
        path: "/api/users".to_string(),
        headers,
        query_params: HashMap::new(),
    };

    assert_eq!(parse_and_eval("has_header(\"Authorization\")", &req), Ok(true));
    assert_eq!(parse_and_eval("has_header(\"X-Custom-Header\")", &req), Ok(false));
}

#[test]
fn test_has_query() {
    let mut query_params = HashMap::new();
    query_params.insert("page".to_string(), "1".to_string());
    query_params.insert("limit".to_string(), "10".to_string());

    let req = HttpRequest {
        method: "GET".to_string(),
        path: "/api/users".to_string(),
        headers: HashMap::new(),
        query_params,
    };

    assert_eq!(parse_and_eval("has_query(\"page\")", &req), Ok(true));
    assert_eq!(parse_and_eval("has_query(\"offset\")", &req), Ok(false));
}

#[test]
fn test_header_equals() {
    let mut headers = HashMap::new();
    headers.insert("Content-Type".to_string(), "application/json".to_string());

    let req = HttpRequest {
        method: "POST".to_string(),
        path: "/api/users".to_string(),
        headers,
        query_params: HashMap::new(),
    };

    assert_eq!(
        parse_and_eval("header_equals(\"Content-Type\", \"application/json\")", &req),
        Ok(true)
    );
    assert_eq!(
        parse_and_eval("header_equals(\"Content-Type\", \"text/html\")", &req),
        Ok(false)
    );
}

#[test]
fn test_query_equals() {
    let mut query_params = HashMap::new();
    query_params.insert("status".to_string(), "active".to_string());

    let req = HttpRequest {
        method: "GET".to_string(),
        path: "/api/users".to_string(),
        headers: HashMap::new(),
        query_params,
    };

    assert_eq!(
        parse_and_eval("query_equals(\"status\", \"active\")", &req),
        Ok(true)
    );
    assert_eq!(
        parse_and_eval("query_equals(\"status\", \"inactive\")", &req),
        Ok(false)
    );
}

#[test]
fn test_complex_and_or_combination() {
    let mut headers = HashMap::new();
    headers.insert("Authorization".to_string(), "Bearer token".to_string());

    let req = HttpRequest {
        method: "POST".to_string(),
        path: "/api/admin/users".to_string(),
        headers,
        query_params: HashMap::new(),
    };

    assert_eq!(
        parse_and_eval(
            "path_prefix(\"/api\") and method(\"POST\") or has_header(\"X-Admin\")",
            &req
        ),
        Ok(true)
    );
}

#[test]
fn test_parentheses_for_precedence() {
    let req = HttpRequest {
        method: "GET".to_string(),
        path: "/api/users".to_string(),
        headers: HashMap::new(),
        query_params: HashMap::new(),
    };

    // Without parentheses: GET and /api or /admin = (GET and /api) or /admin = true or false = true
    assert_eq!(
        parse_and_eval("method(\"GET\") and path_prefix(\"/api\") or path_prefix(\"/admin\")", &req),
        Ok(true)
    );

    // With parentheses: GET and (/api or /admin) = GET and true = true
    assert_eq!(
        parse_and_eval("method(\"GET\") and (path_prefix(\"/api\") or path_prefix(\"/admin\"))", &req),
        Ok(true)
    );
}

#[test]
fn test_nested_not_with_and() {
    let req = HttpRequest {
        method: "GET".to_string(),
        path: "/api/users".to_string(),
        headers: HashMap::new(),
        query_params: HashMap::new(),
    };

    assert_eq!(
        parse_and_eval("not method(\"POST\") and path_prefix(\"/api\")", &req),
        Ok(true)
    );
    assert_eq!(
        parse_and_eval("method(\"GET\") and not path_prefix(\"/admin\")", &req),
        Ok(true)
    );
}

#[test]
fn test_not_with_parentheses() {
    let req = HttpRequest {
        method: "GET".to_string(),
        path: "/api/users".to_string(),
        headers: HashMap::new(),
        query_params: HashMap::new(),
    };

    assert_eq!(
        parse_and_eval("not (method(\"POST\") or method(\"PUT\"))", &req),
        Ok(true)
    );
    assert_eq!(
        parse_and_eval("not (method(\"GET\") and path_prefix(\"/api\"))", &req),
        Ok(false)
    );
}

#[test]
fn test_multiple_headers_check() {
    let mut headers = HashMap::new();
    headers.insert("Authorization".to_string(), "Bearer token".to_string());
    headers.insert("Content-Type".to_string(), "application/json".to_string());

    let req = HttpRequest {
        method: "POST".to_string(),
        path: "/api/users".to_string(),
        headers,
        query_params: HashMap::new(),
    };

    assert_eq!(
        parse_and_eval(
            "has_header(\"Authorization\") and has_header(\"Content-Type\")",
            &req
        ),
        Ok(true)
    );
    assert_eq!(
        parse_and_eval(
            "has_header(\"Authorization\") and has_header(\"X-Custom\")",
            &req
        ),
        Ok(false)
    );
}

#[test]
fn test_api_gateway_rule() {
    let mut headers = HashMap::new();
    headers.insert("X-API-Key".to_string(), "secret123".to_string());

    let mut query_params = HashMap::new();
    query_params.insert("version".to_string(), "v2".to_string());

    let req = HttpRequest {
        method: "GET".to_string(),
        path: "/api/v2/resources".to_string(),
        headers,
        query_params,
    };

    assert_eq!(
        parse_and_eval(
            "path_prefix(\"/api\") and method(\"GET\") and has_header(\"X-API-Key\") and has_query(\"version\")",
            &req
        ),
        Ok(true)
    );
}

#[test]
fn test_admin_or_public_access() {
    let req = HttpRequest {
        method: "GET".to_string(),
        path: "/public/docs".to_string(),
        headers: HashMap::new(),
        query_params: HashMap::new(),
    };

    assert_eq!(
        parse_and_eval(
            "path_prefix(\"/admin\") or path_prefix(\"/public\")",
            &req
        ),
        Ok(true)
    );
}

#[test]
fn test_write_operation_with_auth() {
    let mut headers = HashMap::new();
    headers.insert("Authorization".to_string(), "Bearer admin-token".to_string());

    let req = HttpRequest {
        method: "POST".to_string(),
        path: "/api/users".to_string(),
        headers,
        query_params: HashMap::new(),
    };

    assert_eq!(
        parse_and_eval(
            "(method(\"POST\") or method(\"PUT\") or method(\"DELETE\")) and has_header(\"Authorization\")",
            &req
        ),
        Ok(true)
    );
}

#[test]
fn test_cors_preflight() {
    let mut headers = HashMap::new();
    headers.insert("Origin".to_string(), "https://example.com".to_string());

    let req = HttpRequest {
        method: "OPTIONS".to_string(),
        path: "/api/users".to_string(),
        headers,
        query_params: HashMap::new(),
    };

    assert_eq!(
        parse_and_eval("method(\"OPTIONS\") and has_header(\"Origin\")", &req),
        Ok(true)
    );
}

#[test]
fn test_versioned_api_with_query() {
    let mut query_params = HashMap::new();
    query_params.insert("api_version".to_string(), "2".to_string());

    let req = HttpRequest {
        method: "GET".to_string(),
        path: "/api/resources".to_string(),
        headers: HashMap::new(),
        query_params,
    };

    assert_eq!(
        parse_and_eval(
            "path_prefix(\"/api\") and query_equals(\"api_version\", \"2\")",
            &req
        ),
        Ok(true)
    );
}

#[test]
fn test_health_check_endpoint() {
    let req = HttpRequest {
        method: "GET".to_string(),
        path: "/health".to_string(),
        headers: HashMap::new(),
        query_params: HashMap::new(),
    };

    assert_eq!(
        parse_and_eval(
            "method(\"GET\") and (path_prefix(\"/health\") or path_prefix(\"/status\"))",
            &req
        ),
        Ok(true)
    );
}

#[test]
fn test_complex_authorization_rule() {
    let mut headers = HashMap::new();
    headers.insert("Authorization".to_string(), "Bearer token".to_string());
    headers.insert("X-Admin-Role".to_string(), "true".to_string());

    let req = HttpRequest {
        method: "DELETE".to_string(),
        path: "/api/admin/users".to_string(),
        headers,
        query_params: HashMap::new(),
    };

    assert_eq!(
        parse_and_eval(
            "path_prefix(\"/api/admin\") and method(\"DELETE\") and has_header(\"Authorization\") and has_header(\"X-Admin-Role\")",
            &req
        ),
        Ok(true)
    );
}
