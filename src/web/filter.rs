use crate::{
  lee::parser::{condition::PartCondition, error::CompileError, expression::EvaluateFunc},
  moving::pilot::Pilot,
};
use lazy_static::lazy_static;
use std::collections::HashSet;

lazy_static! {
  static ref ALLOWED_FIELDS: HashSet<&'static str> = HashSet::from([
    "callsign",
    "name",
    "alt",
    "gs",
    "lat",
    "lng",
    "aircraft",
    "arrival",
    "departure",
  ]);
  static ref FIELDS_LIST: Vec<&'static str> = ALLOWED_FIELDS.iter().cloned().collect();
}

// Compilation callback
// TODO: add checks for supported condition identifiers
pub fn compile_filter(cond: PartCondition) -> Result<Box<EvaluateFunc<Pilot>>, CompileError> {
  if !ALLOWED_FIELDS.contains(cond.ident.as_str()) {
    Err(CompileError {
      msg: format!(
        "{} is not a valid field to query, valid fields are: [{}]",
        cond.ident,
        FIELDS_LIST.join(", ")
      ),
    })
  } else {
    Ok(Box::new(move |pilot| apply_filter(&cond, pilot)))
  }
}

pub fn apply_filter(cond: &PartCondition, pilot: &Pilot) -> bool {
  match cond.ident.as_str() {
    "callsign" => cond.eval_str(&pilot.callsign),
    "name" => cond.eval_str(&pilot.name),
    "alt" => cond.eval_i64(pilot.altitude as i64),
    "gs" => cond.eval_i64(pilot.groundspeed as i64),
    "lat" => cond.eval_f64(pilot.position.lat),
    "lng" => cond.eval_f64(pilot.position.lng),
    "aircraft" => pilot
      .flight_plan
      .as_ref()
      .map(|fp| cond.eval_str(&fp.aircraft))
      .unwrap_or(false),
    "arrival" => pilot
      .flight_plan
      .as_ref()
      .map(|fp| cond.eval_str(&fp.arrival))
      .unwrap_or(false),
    "departure" => pilot
      .flight_plan
      .as_ref()
      .map(|fp| cond.eval_str(&fp.departure))
      .unwrap_or(false),
    _ => true,
  }
}
