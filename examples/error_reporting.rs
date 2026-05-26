use winnow::Parser;
use fel::lang::parsers::expr;

fn main() {
    let examples = vec![
        ("Valid", "path_prefix(\"/api\") and method(\"GET\")"),
        ("Missing closing paren", "path_prefix(\"/api\""),
        ("Missing opening paren", "path_prefix\"/api\")"),
        ("Unclosed string", "path_prefix(\"/api)"),
        ("Invalid operator", "path_prefix(\"/api\" AND method(\"GET\")"),
        ("Missing comma", "path_prefix(\"/api\" \"/v1\")"),
        ("Empty expression", ""),
        ("Just operator", "and"),
        ("Unmatched parens", "(path_prefix(\"/api\")"),
    ];

    for (label, input) in examples {
        println!("\n{}: {}", label, input);
        println!("{}", "=".repeat(60));

        match expr.parse(input) {
            Ok(result) => {
                println!("✓ Parsed successfully");
            }
            Err(e) => {
                eprintln!("✗ Parse error:");
                eprintln!("{}", e);
            }
        }
    }
}
