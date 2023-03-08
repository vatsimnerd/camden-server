use serde::Serialize;

use crate::{
  fixed::types::{Airport, FIR},
  moving::pilot::Pilot,
};

#[derive(Debug, Serialize)]
pub struct ObjectsSet {
  #[serde(skip_serializing_if = "Option::is_none")]
  pub pilots: Option<Vec<Pilot>>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub airports: Option<Vec<Airport>>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub firs: Option<Vec<FIR>>,
}

impl ObjectsSet {
  pub fn is_empty(&self) -> bool {
    // if option is none, it's empty, otherwise unwrap it and check if the vec inside is empty
    self.pilots.as_ref().map(|p| p.is_empty()).unwrap_or(true)
      && self.airports.as_ref().map(|a| a.is_empty()).unwrap_or(true)
      && self.firs.as_ref().map(|f| f.is_empty()).unwrap_or(true)
  }
}

#[derive(Debug, Serialize)]
pub struct Update {
  pub set: Option<ObjectsSet>,
  pub delete: Option<ObjectsSet>,
}

impl Update {
  pub fn is_empty(&self) -> bool {
    self.set.as_ref().map(|s| s.is_empty()).unwrap_or(true)
      && self.delete.as_ref().map(|s| s.is_empty()).unwrap_or(true)
  }
}

#[derive(Debug, Serialize)]
pub struct UpdateMessage {
  pub connection_id: String,
  pub message_type: &'static str,
  pub object_type: &'static str,
  pub data: Update,
}

impl UpdateMessage {
  pub fn pilots_set(connection_id: &str, data: Vec<Pilot>) -> Self {
    Self {
      connection_id: connection_id.to_owned(),
      message_type: "update",
      object_type: "pilot",
      data: Update {
        set: Some(ObjectsSet {
          pilots: Some(data),
          airports: None,
          firs: None,
        }),
        delete: None,
      },
    }
  }
  pub fn pilots_delete(connection_id: &str, data: Vec<Pilot>) -> Self {
    Self {
      connection_id: connection_id.to_owned(),
      message_type: "update",
      object_type: "pilot",
      data: Update {
        set: None,
        delete: Some(ObjectsSet {
          pilots: Some(data),
          airports: None,
          firs: None,
        }),
      },
    }
  }
  pub fn airports_set(connection_id: &str, data: Vec<Airport>) -> Self {
    Self {
      connection_id: connection_id.to_owned(),
      message_type: "update",
      object_type: "airport",
      data: Update {
        set: Some(ObjectsSet {
          pilots: None,
          airports: Some(data),
          firs: None,
        }),
        delete: None,
      },
    }
  }
  pub fn airports_delete(connection_id: &str, data: Vec<Airport>) -> Self {
    Self {
      connection_id: connection_id.to_owned(),
      message_type: "update",
      object_type: "airport",
      data: Update {
        set: None,
        delete: Some(ObjectsSet {
          pilots: None,
          airports: Some(data),
          firs: None,
        }),
      },
    }
  }
  pub fn firs_set(connection_id: &str, data: Vec<FIR>) -> Self {
    Self {
      connection_id: connection_id.to_owned(),
      message_type: "update",
      object_type: "fir",
      data: Update {
        set: Some(ObjectsSet {
          pilots: None,
          airports: None,
          firs: Some(data),
        }),
        delete: None,
      },
    }
  }
  pub fn firs_delete(connection_id: &str, data: Vec<FIR>) -> Self {
    Self {
      connection_id: connection_id.to_owned(),
      message_type: "update",
      object_type: "fir",
      data: Update {
        set: None,
        delete: Some(ObjectsSet {
          pilots: None,
          airports: None,
          firs: Some(data),
        }),
      },
    }
  }
}
