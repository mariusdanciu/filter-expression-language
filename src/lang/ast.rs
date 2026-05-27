use std::fmt::Debug;
use thiserror::Error;

#[derive(Error,Debug)]
pub enum EvaluatorError {
    #[error("function not found: {0}")]
    FunctionNotFound(String),
    #[error("invalid arguments: {0}")]
    InvalidArguments(String),
    #[error("evaluation error: {0}")]
    EvaluationError(String),
}

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
    Not { expr: Box<Ast> },
}

impl Ast {
    pub fn eval<T>(
        &self,
        ctx: &T,
        evaluator: FunctionEvaluator<T>,
    ) -> std::result::Result<bool, EvaluatorError> {
        match self {
            Ast::Func { name, args } => evaluator(ctx, name, args),
            Ast::And { left, right } => {
                Ok(left.eval(ctx, evaluator)? && right.eval(ctx, evaluator)?)
            }
            Ast::Or { left, right } => {
                Ok(left.eval(ctx, evaluator)? || right.eval(ctx, evaluator)?)
            }
            Ast::Not { expr } => {
                Ok(!expr.eval(ctx, evaluator)?)
            }
        }
    }
}

pub type FunctionEvaluator<T> = fn(&T, &str, &Vec<Primitives>) -> Result<bool, EvaluatorError>;
