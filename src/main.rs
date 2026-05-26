use winnow::Parser;

mod lang;

use crate::lang::parsers::*;

fn main() {
    let input = "( ( path_prefix(\"/api\", \"/api/v1\") or method(\"GET\") ) and has_header(\"X-API-KEY\") ) or has_query(\"/version\", \"1.0.0\")";

    match expr.parse(input) {
        Ok(result) => {
            println!("✓ Parsed successfully:");
            println!("{:#?}", result);
        }
        Err(e) => {
            eprintln!("✗ Parse error:");
            eprintln!("{}", e);
        }
    }

}
