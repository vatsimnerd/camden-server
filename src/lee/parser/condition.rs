use super::{
  error::CompileError,
  expression::{CompileFunc, EvaluateFunc},
};
use regex::Regex;
use std::str::FromStr;

#[derive(Debug, Clone)]
pub enum Operator {
  Matches,
  NotMatches,
  Equals,
  NotEquals,
  Less,
  LessOrEqual,
  Greater,
  GreaterOrEqual,
}

impl Operator {
  pub fn literal(&self) -> &'static str {
    match self {
      Operator::Matches => "=~",
      Operator::NotMatches => "!~",
      Operator::Equals => "==",
      Operator::NotEquals => "!=",
      Operator::Less => "<",
      Operator::LessOrEqual => "<=",
      Operator::Greater => ">",
      Operator::GreaterOrEqual => ">=",
    }
  }
}

#[derive(Clone)]
pub enum Value {
  Integer(i64),
  Float(f64),
  String(String),
}

impl Value {
  pub fn value_type(&self) -> &'static str {
    match self {
      Value::Integer(_) => "integer",
      Value::Float(_) => "float",
      Value::String(_) => "string",
    }
  }

  pub fn as_string(&self) -> String {
    match self {
      Value::Integer(v) => format!("int({})", v),
      Value::Float(v) => format!("float({})", v),
      Value::String(v) => format!("string({})", v),
    }
  }
}

pub struct PartCondition {
  pub ident: String,
  pub operator: Operator,
  pub value: Value,
}

impl PartCondition {
  pub fn eval_i64(&self, ext_val: i64) -> bool {
    match self.value {
      Value::Integer(v) => match self.operator {
        Operator::Equals => ext_val == v,
        Operator::NotEquals => ext_val != v,
        Operator::Less => ext_val < v,
        Operator::LessOrEqual => ext_val <= v,
        Operator::Greater => ext_val > v,
        Operator::GreaterOrEqual => ext_val >= v,
        _ => false,
      },
      Value::Float(v) => {
        let ext_val = ext_val as f64;
        match self.operator {
          Operator::Equals => ext_val == v,
          Operator::NotEquals => ext_val != v,
          Operator::Less => ext_val < v,
          Operator::LessOrEqual => ext_val <= v,
          Operator::Greater => ext_val > v,
          Operator::GreaterOrEqual => ext_val >= v,
          _ => false,
        }
      }
      Value::String(_) => false,
    }
  }

  pub fn eval_f64(&self, ext_val: f64) -> bool {
    match self.value {
      Value::Integer(v) => {
        let v = v as f64;
        match self.operator {
          Operator::Equals => ext_val == v,
          Operator::NotEquals => ext_val != v,
          Operator::Less => ext_val < v,
          Operator::LessOrEqual => ext_val <= v,
          Operator::Greater => ext_val > v,
          Operator::GreaterOrEqual => ext_val >= v,
          _ => false,
        }
      }
      Value::Float(v) => match self.operator {
        Operator::Equals => ext_val == v,
        Operator::NotEquals => ext_val != v,
        Operator::Less => ext_val < v,
        Operator::LessOrEqual => ext_val <= v,
        Operator::Greater => ext_val > v,
        Operator::GreaterOrEqual => ext_val >= v,
        _ => false,
      },
      Value::String(_) => false,
    }
  }

  pub fn eval_str(&self, ext_val: &str) -> bool {
    match &self.value {
      Value::Integer(_) => false,
      Value::Float(_) => false,
      Value::String(v) => match self.operator {
        Operator::Matches => {
          let re = Regex::from_str(v);
          if let Ok(re) = re {
            re.is_match(ext_val)
          } else {
            false
          }
        }
        Operator::NotMatches => {
          // TODO: evaluation errors
          let re = Regex::from_str(v);
          if let Ok(re) = re {
            !re.is_match(ext_val)
          } else {
            true
          }
        }
        Operator::Equals => ext_val == v,
        Operator::NotEquals => ext_val != v,
        _ => false,
      },
    }
  }

  pub fn repr(&self) -> String {
    format!(
      "PartCondition<({} {} {})>",
      self.ident,
      self.operator.literal(),
      self.value.as_string()
    )
  }
}

pub struct Condition<T> {
  pub ident: String,
  pub operator: Operator,
  pub value: Value,
  pub callback: Option<Box<EvaluateFunc<T>>>,
}

impl<T> Condition<T> {
  pub fn repr(&self) -> String {
    format!(
      "Condition<({} {} {})>",
      self.ident,
      self.operator.literal(),
      self.value.as_string()
    )
  }

  pub fn compile(&mut self, cb: &Box<CompileFunc<T>>) -> Result<(), CompileError> {
    let evalfunc = cb(self.part_clone())?;
    self.callback = Some(evalfunc);
    Ok(())
  }

  pub fn evaluate(&self, model: &T) -> bool {
    if let Some(cb) = &self.callback {
      cb(model)
    } else {
      false
    }
  }

  pub fn part_clone(&self) -> PartCondition {
    PartCondition {
      ident: self.ident.clone(),
      operator: self.operator.clone(),
      value: self.value.clone(),
    }
  }
}
