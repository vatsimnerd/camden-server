mod data;

use lazy_static::lazy_static;
use serde::Serialize;
use std::collections::HashMap;

#[derive(Debug, Serialize, PartialEq)]
pub enum EngineType {
  Electric,
  Jet,
  Piston,
  Rocket,
  Turboprop,
}

#[derive(Debug, Serialize, PartialEq)]
pub enum AircraftType {
  Amphibian,
  Gyrocopter,
  Helicopter,
  LandPlane,
  SeaPlane,
  Tiltrotor,
}

#[derive(Debug, Serialize, PartialEq)]
pub struct Aircraft {
  pub name: &'static str,
  pub description: &'static str,
  pub wtc: &'static str,
  pub wtg: &'static str,
  pub designator: &'static str,
  pub manufacturer_code: &'static str,
  pub aircraft_type: AircraftType,
  pub engine_count: u8,
  pub engine_type: EngineType,
}

lazy_static! {
  static ref DB: HashMap<&'static str, Vec<&'static Aircraft>> = {
    let mut db: HashMap<&'static str, Vec<&'static Aircraft>> = HashMap::new();
    for atype in data::MODELS {
      let ex = db.get_mut(atype.designator);
      if let Some(ex) = ex {
        ex.push(atype);
      } else {
        db.insert(atype.designator, vec![atype]);
      }
    }
    db
  };
}

pub fn guess_aircraft_types(code: &str) -> Option<Vec<&'static Aircraft>> {
  let mut l = code.len().clamp(0, 4);
  while l > 0 {
    let partial_code = &code[..l];
    let atypes = DB.get(partial_code);
    if atypes.is_some() {
      return atypes.cloned();
    }
    l -= 1;
  }
  None
}
