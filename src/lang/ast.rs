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
    pub fn eval(&self, evaluator: FunctionEvaluator) -> std::result::Result<bool, String> {
        match self {
            Ast::Func { name, args } => evaluator(name, args),
            Ast::And { left, right } => Ok(left.eval(evaluator)? && right.eval(evaluator)?),
            Ast::Or { left, right } => Ok(left.eval(evaluator)? || right.eval(evaluator)?),
        }
    }
}

pub type FunctionEvaluator = fn(&str, &Vec<Primitives>) -> Result<bool, String>;
