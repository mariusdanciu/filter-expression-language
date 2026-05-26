use std::fmt::Debug;

#[derive(Debug)]
pub enum Primitives {
    Bool(bool),
    String(String),
    Int(i32),
}

#[derive(Debug)]
pub enum Ast {
    Func { name: String, args: Vec<Primitives> },
    And { left: Box<Ast>, right: Box<Ast> },
    Or { left: Box<Ast>, right: Box<Ast> },
}

impl Ast {
    fn eval(&self) -> std::result::Result<bool, String> {
        match self {
            Ast::Func { name, args } => Ok(true),
            Ast::And { left, right } => Ok(left.eval()? && right.eval()?),
            Ast::Or { left, right } => Ok(left.eval()? || right.eval()?),
        }
    }
}
