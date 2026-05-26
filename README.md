# Filter Expression Language (FEL)

A boolean expression parser built with [winnow](https://github.com/winnow-rs/winnow) parser combinators for Rust. FEL parses expressions with AND/OR operators, function calls, and parentheses into an Abstract Syntax Tree (AST).

## Features

- **Boolean operators**: `and`, `or` with correct precedence (AND binds tighter than OR)
- **Function calls**: Support for functions with multiple arguments
- **Parentheses**: Group expressions to override default precedence
- **Left-associative**: Chains like `a or b or c` parse as `((a or b) or c)`
- **Type-safe AST**: Strongly-typed representation using Rust enums
- **Comprehensive tests**: 10 test cases validating parser correctness

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
fel = { git = "https://github.com/mariusdanciu/filter-expression-language" }
```

## Quick Start

```rust
use winnow::Parser;
use fel::lang::parsers::*;
use fel::lang::ast::{Ast, Primitives};

fn main() {
    let mut input = "path_prefix(\"/api\") and method(\"GET\")";
    
    let result = expr.parse_next(&mut input).unwrap();
    
    println!("{:#?}", result);
}
```

## Grammar

```
expr       ::= and_expr ('or' and_expr)*
and_expr   ::= term ('and' term)*
term       ::= func_call | parens_expr
parens_expr::= '(' expr ')'
func_call  ::= identifier '(' args? ')'
args       ::= primitive (',' primitive)*
primitive  ::= string | int | bool
string     ::= '"' [^"]* '"'
int        ::= [0-9]+
bool       ::= 'true' | 'false'
identifier ::= [a-zA-Z_][a-zA-Z0-9_]*
```

## Operator Precedence

1. Parentheses (highest)
2. `and`
3. `or` (lowest)

### Examples

```rust
// AND binds tighter than OR
"a or b and c"  // parses as: a or (b and c)

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
pub type FunctionEvaluator<T> = fn(&T, &str, &Vec<Primitives>) -> Result<bool, String>;

impl Ast {
    pub fn eval<T>(
        &self,
        ctx: &T,
        evaluator: FunctionEvaluator<T>,
    ) -> Result<bool, String>
}
```

### Complete Example

```rust
use winnow::Parser;
use fel::lang::parsers::*;
use fel::lang::ast::Primitives;

// Define your context type
struct HttpRequest {
    path: String,
    method: String,
    headers: Vec<(String, String)>,
}

fn main() {
    // Parse the expression
    let input = "path_prefix(\"/api\") and method(\"GET\")";
    let ast = expr.parse(input).unwrap();

    // Create a context
    let request = HttpRequest {
        path: "/api/users".to_string(),
        method: "GET".to_string(),
        headers: vec![
            ("X-API-KEY".to_string(), "secret123".to_string()),
        ],
    };

    // Define the evaluator function
    let evaluator = |ctx: &HttpRequest, name: &str, args: &Vec<Primitives>| {
        match name {
            "path_prefix" => {
                if let Some(Primitives::String(prefix)) = args.first() {
                    Ok(ctx.path.starts_with(prefix))
                } else {
                    Err("path_prefix requires a string argument".to_string())
                }
            }
            "method" => {
                if let Some(Primitives::String(method)) = args.first() {
                    Ok(ctx.method == *method)
                } else {
                    Err("method requires a string argument".to_string())
                }
            }
            "has_header" => {
                if let Some(Primitives::String(header_name)) = args.first() {
                    Ok(ctx.headers.iter().any(|(name, _)| name == header_name))
                } else {
                    Err("has_header requires a string argument".to_string())
                }
            }
            _ => Err(format!("Unknown function: {}", name)),
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

It returns `Result<bool, String>`:
- `Ok(true)` or `Ok(false)` for the boolean result
- `Err(message)` for evaluation errors

The AST handles the boolean logic (`and`, `or`) automatically, calling your evaluator only for leaf function nodes.

## Running Tests

```bash
cargo test
```

The test suite includes validation for:
- Simple function calls
- OR expressions
- AND expressions
- Combined AND/OR with precedence
- Parenthesized expressions
- Nested parentheses
- Multiple OR chains (left-associativity)
- Multiple AND chains (left-associativity)
- Complex nested expressions
- Functions with multiple arguments

## Project Structure

```
fel/
├── src/
│   ├── lang/
│   │   ├── ast.rs       # AST data structures
│   │   ├── parsers.rs   # Parser combinators
│   │   └── mod.rs       # Module exports
│   ├── lib.rs           # Library entry point
│   └── main.rs          # Example binary
└── tests/
    └── expression_tests.rs  # Test suite
```

## Use Cases

FEL is designed for scenarios where you need to parse and evaluate filter expressions:

- **API Gateway routing**: Route requests based on path, method, headers
- **Rule engines**: Define business rules using boolean logic
- **Configuration DSLs**: User-friendly syntax for complex conditions
- **Query builders**: Parse filter expressions into executable queries

## License

MIT
