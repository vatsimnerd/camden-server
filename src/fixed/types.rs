use crate::{
  atis::runways::{detect_arrivals, detect_departures, normalize_atis_text},
  moving::controller::{Controller, ControllerSet},
  types::Point,
};
use serde::Serialize;
use std::collections::HashMap;

use super::ourairports::Runway;

#[derive(Debug, Clone)]
pub struct Country {
  pub name: String,
  pub prefix: String,
  pub control_name: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Airport {
  pub icao: String,
  pub iata: String,
  pub name: String,
  pub position: Point,
  pub fir_id: String,
  pub is_pseudo: bool,
  pub controllers: ControllerSet,
  pub runways: HashMap<String, Runway>,
}

impl Airport {
  pub fn compound_id(&self) -> String {
    format!("{}:{}", self.icao, self.iata)
  }

  pub fn reset_active_runways(&mut self) {
    for (_, rwy) in self.runways.iter_mut() {
      rwy.active_lnd = false;
      rwy.active_to = false;
    }
  }

  pub fn set_active_runways(&mut self) {
    self.reset_active_runways();
    if let Some(atis) = &self.controllers.atis {
      let norm_atis = normalize_atis_text(&atis.text_atis, true);
      let arrivals = detect_arrivals(&norm_atis);
      let departures = detect_departures(&norm_atis);
      for ident in arrivals.iter() {
        let rwy = self.runways.get_mut(ident);
        if let Some(rwy) = rwy {
          rwy.active_lnd = true
        }
      }
      for ident in departures.iter() {
        let rwy = self.runways.get_mut(ident);
        if let Some(rwy) = rwy {
          rwy.active_to = true
        }
      }
    }
  }
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct FIR {
  pub icao: String,
  pub name: String,
  pub prefix: String,
  pub boundaries: Boundaries,
  pub controllers: HashMap<String, Controller>,
}

impl FIR {
  pub fn is_empty(&self) -> bool {
    self.controllers.len() == 0
  }
}

#[derive(Debug, Clone)]
pub struct UIR {
  pub icao: String,
  pub name: String,
  pub fir_ids: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct Boundaries {
  pub id: String,
  pub region: String,
  pub division: String,
  pub is_oceanic: bool,
  pub min: Point,
  pub max: Point,
  pub center: Point,
  pub points: Vec<Vec<Point>>,
}

impl PartialEq for Boundaries {
  // simplify partial eq as boundaries don't change within a single app run
  fn eq(&self, other: &Self) -> bool {
    self.id == other.id
  }
}
