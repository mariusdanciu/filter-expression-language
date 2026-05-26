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
