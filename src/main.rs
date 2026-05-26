
use winnow::Parser;

mod lang;

use crate::lang::parsers::*;


fn main() {
    let mut input =
        "( ( path_prefix(\"/api\", \"/api/v1\") or method(\"GET\") ) and has_header(\"X-API-KEY\") ) or has_query(\"/version\", \"1.0.0\")";

    let output = expr.parse_next(&mut input);

    match output {
        Ok(result) => {
            println!("{:?}", result);
        }
        Err(e) => println!("Error: {:?}", e),
    }
}
