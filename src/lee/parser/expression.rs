use super::{
  condition::{Condition, PartCondition},
  error::CompileError,
};

#[derive(Debug)]
pub enum CombineOperator {
  And,
  Or,
}

pub type EvaluateFunc<T> = dyn Fn(&T) -> bool + Send + Sync;
pub type CompileFunc<T> = dyn Fn(PartCondition) -> Result<Box<EvaluateFunc<T>>, CompileError>;

pub enum LeftExpression<T> {
  Condition(Condition<T>),
  Expression(Expression<T>),
}

pub struct Expression<T> {
  pub left: Box<LeftExpression<T>>,
  pub operator: Option<CombineOperator>,
  pub right: Option<Box<Expression<T>>>,
}

impl<T> Expression<T> {
  pub fn compile(&mut self, cb: &Box<CompileFunc<T>>) -> Result<(), CompileError> {
    match self.left.as_mut() {
      LeftExpression::Condition(c) => {
        c.compile(cb)?;
      }
      LeftExpression::Expression(e) => {
        e.compile(cb)?;
      }
    };
    if let Some(right) = self.right.as_mut() {
      right.compile(cb)?;
    }
    Ok(())
  }

  pub fn evaluate(&self, model: &T) -> bool {
    let left_result = match self.left.as_ref() {
      LeftExpression::Condition(c) => c.evaluate(model),
      LeftExpression::Expression(e) => e.evaluate(model),
    };

    if self.operator.is_none() {
      left_result
    } else {
      let right = self.right.as_ref().unwrap();
      match self.operator.as_ref().unwrap() {
        CombineOperator::And => left_result && right.evaluate(model),
        CombineOperator::Or => left_result || right.evaluate(model),
      }
    }
  }
}
