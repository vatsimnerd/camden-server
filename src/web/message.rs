use serde::Serialize;

use crate::{
  fixed::types::{Airport, FIR},
  moving::pilot::Pilot,
};

#[derive(Debug, Serialize)]
pub struct ObjectsSet {
  pub pilots: Vec<Pilot>,
  pub airports: Vec<Airport>,
  pub firs: Vec<FIR>,
}

impl ObjectsSet {
  pub fn is_empty(&self) -> bool {
    self.pilots.is_empty() && self.airports.is_empty() && self.firs.is_empty()
  }
}

#[derive(Debug, Serialize)]
pub struct Update {
  pub set: ObjectsSet,
  pub delete: ObjectsSet,
}

impl Update {
  pub fn is_empty(&self) -> bool {
    self.set.is_empty() && self.delete.is_empty()
  }
}

#[derive(Debug, Serialize)]
pub struct UpdateMessage {
  pub connection_id: String,
  pub message_type: &'static str,
  pub data: Update,
}
