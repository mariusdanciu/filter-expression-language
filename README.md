# Filter Expression Language (FEL)

A boolean expression parser built with [winnow](https://github.com/winnow-rs/winnow) parser combinators for Rust. FEL parses expressions with AND/OR operators, function calls, and parentheses into an Abstract Syntax Tree (AST).

## Features

- **Boolean operators**: `and`, `or`, `not` with correct precedence
- **Function calls**: Support for functions with multiple arguments
- **Parentheses**: Group expressions to override default precedence
- **Left-associative**: Chains like `a or b or c` parse as `((a or b) or c)`
- **Type-safe AST**: Strongly-typed representation using Rust enums
- **Stateful parsing**: Context-aware parsing with function validation
- **Comprehensive tests**: 20+ test cases validating parser correctness and evaluation

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
fel = { git = "https://github.com/mariusdanciu/filter-expression-language" }
```

## Quick Start

```rust
use winnow::Parser;
use winnow::stream::Stateful;
use fel::lang::parsers::*;
use fel::lang::ast::{ParserContext, PrimitivesTypes};
use std::collections::HashMap;

fn main() {
    let input_str = "path_prefix(\"/api\") and method(\"GET\")";
    
    // Create parser context with known functions
    let mut ctx = ParserContext {
        known_functions: HashMap::new(),
        original_input: input_str.to_string(),
    };
    ctx.known_functions.insert("path_prefix".to_string(), vec![PrimitivesTypes::String]);
    ctx.known_functions.insert("method".to_string(), vec![PrimitivesTypes::String]);
    
    let input = Stateful { input: input_str, state: &ctx };
    let result = expr.parse(input).unwrap();
    
    println!("{:#?}", result);
}
```

## Grammar

```
expr       ::= or_expr
or_expr    ::= and_expr ('or' and_expr)*
and_expr   ::= term ('and' term)*
term       ::= 'not'? (func_call | parens_expr)
parens_expr::= '(' expr ')'
func_call  ::= identifier '(' args? ')'
args       ::= primitive (',' primitive)*
primitive  ::= string | int | bool
string     ::= '"' [^"]* '"'
int        ::= [0-9]+
bool       ::= 'true' | 'false'
identifier ::= [a-zA-Z_][a-zA-Z0-9_-]*
```

## Operator Precedence

1. Parentheses (highest)
2. `not`
3. `and`
4. `or` (lowest)

### Examples

```rust
// AND binds tighter than OR
"a or b and c"  // parses as: a or (b and c)

// NOT has high precedence
"not a and b"  // parses as: (not a) and b
"not (a or b)"  // parses as: not (a or b)

// Parentheses override precedence
"(a or b) and c"  // parses as: (a or b) and c

// Left-associative chains
"a or b or c"  // parses as: (a or b) or c
"a and b and c"  // parses as: (a and b) and c
```

## AST Structure

The AST is represented using two main enums:

### Ast

```rust
pub enum Ast {
    Func { name: String, args: Vec<Primitives> },
    And { left: Box<Ast>, right: Box<Ast> },
    Or { left: Box<Ast>, right: Box<Ast> },
    Not { expr: Box<Ast> },
}
```

### Primitives

```rust
pub enum Primitives {
    Bool(bool),
    String(String),
    Int(i32),
}
```

### ParserContext

```rust
pub struct ParserContext {
    pub known_functions: HashMap<String, Vec<PrimitivesTypes>>,
    pub original_input: String,
}

pub enum PrimitivesTypes {
    Bool,
    String,
    Int,
}
```

The `ParserContext` enables stateful parsing, allowing the parser to validate function names and argument types during parsing. This is particularly useful for:
- Function name validation
- Type checking at parse time
- Better error messages with original input context

## Example Expressions

### Simple Function Call

```rust
path_prefix("/api")
```

Parses to:
```
Func {
    name: "path_prefix",
    args: [String("/api")]
}
```

### Boolean Expression

```rust
path_prefix("/api") or method("GET")
```

Parses to:
```
Or {
    left: Func { name: "path_prefix", args: [String("/api")] },
    right: Func { name: "method", args: [String("GET")] }
}
```

### Complex Expression

```rust
(path_prefix("/api") or method("GET")) and has_header("X-API-KEY")
```

Parses to:
```
And {
    left: Or {
        left: Func { name: "path_prefix", args: [String("/api")] },
        right: Func { name: "method", args: [String("GET")] }
    },
    right: Func { name: "has_header", args: [String("X-API-KEY")] }
}
```

### Function with Multiple Arguments

```rust
path_prefix("/api", "/v1", "/v2")
```

Parses to:
```
Func {
    name: "path_prefix",
    args: [String("/api"), String("/v1"), String("/v2")]
}
```

## Evaluating Expressions

After parsing, you can evaluate expressions against a context using the `eval` method.

### Evaluation API

```rust
use thiserror::Error;

#[derive(Error, Debug)]
pub enum EvaluatorError {
    #[error("function not found: {0}")]
    FunctionNotFound(String),
    #[error("invalid arguments: {0}")]
    InvalidArguments(String),
    #[error("evaluation error: {0}")]
    EvaluationError(String),
}

pub type FunctionEvaluator<T> = fn(&T, &str, &Vec<Primitives>) -> Result<bool, EvaluatorError>;

impl Ast {
    pub fn eval<T>(
        &self,
        ctx: &T,
        evaluator: FunctionEvaluator<T>,
    ) -> Result<bool, EvaluatorError>
}
```

### Complete Example

```rust
use winnow::Parser;
use winnow::stream::Stateful;
use fel::lang::parsers::*;
use fel::lang::ast::{EvaluatorError, ParserContext, Primitives, PrimitivesTypes};
use std::collections::HashMap;

// Define your context type
struct HttpRequest {
    path: String,
    method: String,
    headers: HashMap<String, String>,
}

fn main() {
    // Parse the expression
    let input_str = "path_prefix(\"/api\") and method(\"GET\")";
    
    // Create parser context
    let mut parser_ctx = ParserContext {
        known_functions: HashMap::new(),
        original_input: input_str.to_string(),
    };
    parser_ctx.known_functions.insert("path_prefix".to_string(), vec![PrimitivesTypes::String]);
    parser_ctx.known_functions.insert("method".to_string(), vec![PrimitivesTypes::String]);
    parser_ctx.known_functions.insert("has_header".to_string(), vec![PrimitivesTypes::String]);
    
    let input = Stateful { input: input_str, state: &parser_ctx };
    let ast = expr.parse(input).unwrap();

    // Create a context
    let mut headers = HashMap::new();
    headers.insert("X-API-KEY".to_string(), "secret123".to_string());
    
    let request = HttpRequest {
        path: "/api/users".to_string(),
        method: "GET".to_string(),
        headers,
    };

    // Define the evaluator function
    let evaluator = |ctx: &HttpRequest, name: &str, args: &Vec<Primitives>| {
        match name {
            "path_prefix" => {
                if let Some(Primitives::String(prefix)) = args.first() {
                    Ok(ctx.path.starts_with(prefix))
                } else {
                    Err(EvaluatorError::InvalidArguments(
                        "path_prefix requires a string argument".to_string()
                    ))
                }
            }
            "method" => {
                if let Some(Primitives::String(method)) = args.first() {
                    Ok(ctx.method == *method)
                } else {
                    Err(EvaluatorError::InvalidArguments(
                        "method requires a string argument".to_string()
                    ))
                }
            }
            "has_header" => {
                if let Some(Primitives::String(header_name)) = args.first() {
                    Ok(ctx.headers.contains_key(header_name))
                } else {
                    Err(EvaluatorError::InvalidArguments(
                        "has_header requires a string argument".to_string()
                    ))
                }
            }
            _ => Err(EvaluatorError::FunctionNotFound(name.to_string())),
        }
    };

    // Evaluate the expression
    match ast.eval(&request, evaluator) {
        Ok(result) => println!("Result: {}", result),  // true
        Err(e) => eprintln!("Error: {}", e),
    }
}
```

### How Evaluation Works

1. **Parse** the expression string into an AST
2. **Define a context type** containing the data your functions need (e.g., HTTP request)
3. **Create an evaluator function** that maps function names to implementations
4. **Call `ast.eval()`** with your context and evaluator

The evaluator function receives:
- `ctx: &T` - your context (e.g., `&HttpRequest`)
- `name: &str` - the function name (e.g., `"path_prefix"`)
- `args: &Vec<Primitives>` - the function arguments

It returns `Result<bool, EvaluatorError>`:
- `Ok(true)` or `Ok(false)` for the boolean result
- `Err(EvaluatorError::FunctionNotFound)` when the function doesn't exist
- `Err(EvaluatorError::InvalidArguments)` when arguments are invalid
- `Err(EvaluatorError::EvaluationError)` for other evaluation errors

The AST handles the boolean logic (`and`, `or`, `not`) automatically, calling your evaluator only for leaf function nodes.

## Running Tests

```bash
cargo test
```

The test suite includes validation for:
- Simple function calls
- OR expressions
- AND expressions
- NOT operator
- Combined AND/OR with precedence
- Parenthesized expressions
- Nested parentheses and NOT
- Multiple OR chains (left-associativity)
- Multiple AND chains (left-associativity)
- Complex nested expressions
- Functions with multiple arguments
- Real-world HTTP request routing scenarios

## Project Structure

```
fel/
├── src/
│   ├── lang/
│   │   ├── ast.rs       # AST data structures, evaluator, error types
│   │   ├── parsers.rs   # Parser combinators
│   │   └── mod.rs       # Module exports
│   ├── lib.rs           # Library entry point
│   └── main.rs          # Example binary
└── tests/
    └── http_request_tests.rs  # Comprehensive test suite with HTTP routing examples
```

## Use Cases

FEL is designed for scenarios where you need to parse and evaluate filter expressions:

- **API Gateway routing**: Route requests based on path, method, headers
- **Rule engines**: Define business rules using boolean logic
- **Configuration DSLs**: User-friendly syntax for complex conditions
- **Query builders**: Parse filter expressions into executable queries

## License

MIT
