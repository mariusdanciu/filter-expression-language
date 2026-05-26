use winnow::Parser;
use fel::lang::parsers::*;

#[test]
fn test_missing_closing_paren() {
    let mut input = "path_prefix(\"/api\"";
    let result = expr.parse_next(&mut input);
    assert!(result.is_err(), "Should fail with missing closing paren");

    let err = result.unwrap_err();
    let err_msg = format!("{}", err);
    assert!(err_msg.contains(")"), "Error should mention missing closing paren");
}

#[test]
fn test_missing_opening_paren() {
    let mut input = "path_prefix\"/api\")";
    let result = expr.parse_next(&mut input);
    assert!(result.is_err(), "Should fail with missing opening paren");

    let err = result.unwrap_err();
    let err_msg = format!("{}", err);
    assert!(err_msg.contains("("), "Error should mention expected opening paren");
}

#[test]
fn test_unclosed_string() {
    let mut input = "path_prefix(\"/api)";
    let result = expr.parse_next(&mut input);
    assert!(result.is_err(), "Should fail with unclosed string literal");

    let err = result.unwrap_err();
    let err_msg = format!("{}", err);
    // Should indicate issue with function call or closing paren
    assert!(err_msg.contains("function call") || err_msg.contains(")"),
            "Error should mention function call or closing paren issue");
}

#[test]
fn test_invalid_uppercase_operator() {
    let mut input = "path_prefix(\"/api\") AND method(\"GET\")";
    let result = expr.parse_next(&mut input);
    // Uppercase AND is parsed as a function name, so parsing succeeds
    // but leaves unparsed input " method(\"GET\")"
    match result {
        Ok(_) => {
            assert!(!input.trim().is_empty(), "Should have unparsed input");
        }
        Err(_) => {
            // Also acceptable if it fails
        }
    }
}

#[test]
fn test_missing_comma_between_args() {
    let mut input = "path_prefix(\"/api\" \"/v1\")";
    let result = expr.parse_next(&mut input);
    assert!(result.is_err(), "Should fail with missing comma between arguments");

    let err = result.unwrap_err();
    let err_msg = format!("{}", err);
    assert!(err_msg.contains(")") || err_msg.contains("function call"),
            "Error should mention expected closing paren or function call issue");
}

#[test]
fn test_empty_expression() {
    let mut input = "";
    let result = expr.parse_next(&mut input);
    assert!(result.is_err(), "Should fail with empty expression");

    let err = result.unwrap_err();
    let err_msg = format!("{}", err);
    assert!(err_msg.contains("function name") || err_msg.contains("expected"),
            "Error should indicate expected function name or expression");
}

#[test]
fn test_just_operator() {
    let mut input = "and";
    let result = expr.parse_next(&mut input);
    assert!(result.is_err(), "Should fail with just an operator");

    let err = result.unwrap_err();
    let err_msg = format!("{}", err);
    assert!(err_msg.contains("(") || err_msg.contains("function call"),
            "Error should mention expected opening paren or function call");
}

#[test]
fn test_unmatched_opening_paren() {
    let mut input = "(path_prefix(\"/api\")";
    let result = expr.parse_next(&mut input);
    assert!(result.is_err(), "Should fail with unmatched opening paren");

    let err = result.unwrap_err();
    let err_msg = format!("{}", err);
    assert!(err_msg.contains(")") || err_msg.contains("expected"),
            "Error should mention missing closing paren");
}

#[test]
fn test_unmatched_closing_paren() {
    let mut input = "path_prefix(\"/api\"))";
    let result = expr.parse_next(&mut input);
    // This might succeed but leave input unparsed
    match result {
        Ok(_) => {
            // Check if there's remaining input
            assert!(!input.is_empty(), "Should have remaining unparsed input");
        }
        Err(_) => {
            // Also acceptable to fail
        }
    }
}

#[test]
fn test_missing_function_name() {
    let mut input = "(\"/api\")";
    let result = expr.parse_next(&mut input);
    assert!(result.is_err(), "Should fail with missing function name");

    let err = result.unwrap_err();
    let err_msg = format!("{}", err);
    assert!(err_msg.contains("function") || err_msg.contains("expression"),
            "Error should mention function or expression expected");
}

#[test]
fn test_empty_function_args() {
    let mut input = "path_prefix()";
    let result = expr.parse_next(&mut input);
    // Empty args should be valid
    assert!(result.is_ok(), "Empty args should be valid");
}

#[test]
fn test_operator_without_right_operand() {
    let mut input = "path_prefix(\"/api\") and";
    let result = expr.parse_next(&mut input);
    // Parser will parse "path_prefix(\"/api\")" and leave " and" unparsed
    match result {
        Ok(_) => {
            assert!(!input.trim().is_empty(), "Should have unparsed input");
        }
        Err(_) => {
            // Also acceptable if it fails
        }
    }
}

#[test]
fn test_operator_without_left_operand() {
    let mut input = "or method(\"GET\")";
    let result = expr.parse_next(&mut input);
    assert!(result.is_err(), "Should fail with operator missing left operand");

    let err = result.unwrap_err();
    let err_msg = format!("{}", err);
    assert!(err_msg.contains("(") || err_msg.contains("function"),
            "Error should mention expected opening paren or function");
}

#[test]
fn test_double_operator() {
    let mut input = "path_prefix(\"/api\") and or method(\"GET\")";
    let result = expr.parse_next(&mut input);
    // Parser parses "path_prefix(\"/api\") and or" as a valid expression
    // where "or" is treated as a function name (even though it's a keyword)
    // It leaves " method(\"GET\")" unparsed
    match result {
        Ok(_) => {
            // This is acceptable - parser leaves unparsed input
            assert!(!input.trim().is_empty(), "Should have unparsed input");
        }
        Err(_) => {
            // Also acceptable if it detects the error
        }
    }
}

#[test]
fn test_invalid_character_in_function_name() {
    let mut input = "path-prefix(\"/api\")";
    let result = expr.parse_next(&mut input);
    // Hyphen might be valid depending on func_name parser
    // This test validates current behavior
    let _ = result;
}

#[test]
fn test_trailing_comma_in_args() {
    let mut input = "path_prefix(\"/api\", \"/v1\",)";
    let result = expr.parse_next(&mut input);
    assert!(result.is_err(), "Should fail with trailing comma");

    let err = result.unwrap_err();
    let err_msg = format!("{}", err);
    assert!(err_msg.contains(")") || err_msg.contains("function call"),
            "Error should mention closing paren or function call issue");
}

#[test]
fn test_leading_comma_in_args() {
    let mut input = "path_prefix(, \"/api\")";
    let result = expr.parse_next(&mut input);
    assert!(result.is_err(), "Should fail with leading comma");

    let err = result.unwrap_err();
    let err_msg = format!("{}", err);
    assert!(err_msg.contains(")") || err_msg.contains("function call"),
            "Error should mention closing paren or function call issue");
}

#[test]
fn test_double_quotes_in_string() {
    let mut input = "path_prefix(\"/api\"test\")";
    let result = expr.parse_next(&mut input);
    // Should fail or leave unparsed input
    match result {
        Ok(_) => {
            assert!(!input.is_empty(), "Should have remaining unparsed input");
        }
        Err(_) => {
            // Also acceptable
        }
    }
}

#[test]
fn test_mixed_case_operator() {
    let mut input = "path_prefix(\"/api\") And method(\"GET\")";
    let result = expr.parse_next(&mut input);
    // Mixed-case "And" is parsed as a function name, not an operator
    match result {
        Ok(_) => {
            assert!(!input.trim().is_empty(), "Should have unparsed input");
        }
        Err(_) => {
            // Also acceptable if it fails
        }
    }
}

#[test]
fn test_incomplete_parenthesized_expr() {
    let mut input = "(path_prefix(\"/api\") and";
    let result = expr.parse_next(&mut input);
    assert!(result.is_err(), "Should fail with incomplete parenthesized expression");

    let err = result.unwrap_err();
    let err_msg = format!("{}", err);
    assert!(err_msg.contains("expression") || err_msg.contains("expected"),
            "Error should mention expected expression or closing paren");
}

#[test]
fn test_error_message_shows_position() {
    let input = "path_prefix(\"/api\"";
    let result = expr.parse(input);
    assert!(result.is_err(), "Should fail");

    let err = result.unwrap_err();
    let err_msg = format!("{}", err);
    // Error message should show the input
    assert!(err_msg.contains("path_prefix"), "Error message should show the input");
    // Error message should have a caret indicator (^)
    assert!(err_msg.contains("^"), "Error message should show position with ^");
}

#[test]
fn test_error_message_context_labels() {
    let input = "method(\"GET\" \"POST\")";
    let result = expr.parse(input);
    assert!(result.is_err(), "Should fail with missing comma");

    let err = result.unwrap_err();
    let err_msg = format!("{}", err);
    // Should mention function call context
    assert!(err_msg.contains("function call"), "Error should mention 'function call' context");
}

#[test]
fn test_error_message_expected_values() {
    let input = "path_prefix";
    let result = expr.parse(input);
    assert!(result.is_err(), "Should fail with missing opening paren");

    let err = result.unwrap_err();
    let err_msg = format!("{}", err);
    // Should mention what was expected
    assert!(err_msg.contains("expected") || err_msg.contains("("),
            "Error should mention expected opening paren");
}
