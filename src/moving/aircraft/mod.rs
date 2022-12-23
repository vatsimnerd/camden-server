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
  static ref DB: HashMap<&'static str, &'static Aircraft> = {
    let mut db: HashMap<&'static str, Vec<&'static Aircraft>> = HashMap::new();
    for atype in data::MODELS {
      let ex = db.get_mut(atype.designator);
      if let Some(ex) = ex {
        ex.push(atype);
      } else {
        db.insert(atype.designator, vec![atype]);
      }
    }
    db.into_iter()
      .map(|(key, options)| (key, pick_best_at(&options)))
      .collect()
  };
}

fn pick_best_at(options: &Vec<&'static Aircraft>) -> &'static Aircraft {
  if options.len() == 1 {
    options[0]
  } else {
    // TODO
    options[0]
  }
}

pub fn guess_aircraft_types(code: &str) -> Option<&'static Aircraft> {
  let mut l = code.len().clamp(0, 4);
  while l > 0 {
    let partial_code = &code[..l];
    let atype = DB.get(partial_code);
    if let Some(atype) = atype {
      return Some(*atype);
    }
    l -= 1;
  }
  None
}

#[cfg(test)]
pub mod tests {
  use super::guess_aircraft_types;

  #[test]
  fn test_atype() {
    let atype = guess_aircraft_types("B738");
    assert!(atype.is_some());
    let atype = atype.unwrap();
    assert_eq!(atype.name, "737-800");
  }
}
