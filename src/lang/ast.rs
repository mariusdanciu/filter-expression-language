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
    pub fn eval<T>(
        &self,
        ctx: &T,
        evaluator: FunctionEvaluator<T>,
    ) -> std::result::Result<bool, String> {
        match self {
            Ast::Func { name, args } => evaluator(ctx, name, args),
            Ast::And { left, right } => {
                Ok(left.eval(ctx, evaluator)? && right.eval(ctx, evaluator)?)
            }
            Ast::Or { left, right } => {
                Ok(left.eval(ctx, evaluator)? || right.eval(ctx, evaluator)?)
            }
        }
    }
}

pub type FunctionEvaluator<T> = fn(&T, &str, &Vec<Primitives>) -> Result<bool, String>;
