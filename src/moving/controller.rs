use chrono::{DateTime, Utc};
use serde::Serialize;

#[derive(Debug, Clone, PartialEq, Serialize)]
pub enum Facility {
  Reject = 0,
  ATIS = 1,
  Delivery = 2,
  Ground = 3,
  Tower = 4,
  Approach = 5,
  Radar = 6,
}

impl From<i8> for Facility {
  fn from(v: i8) -> Self {
    match v {
      1 => Facility::ATIS,
      2 => Facility::Delivery,
      3 => Facility::Ground,
      4 => Facility::Tower,
      5 => Facility::Approach,
      6 => Facility::Radar,
      _ => Facility::Reject,
    }
  }
}

#[derive(Debug, Clone, Serialize)]
pub struct Controller {
  pub cid: u32,
  pub name: String,
  pub callsign: String,
  pub freq: u32,
  pub facility: Facility,
  pub rating: i32,
  pub server: String,
  pub visual_range: u32,
  pub atis_code: String,
  pub text_atis: String,
  pub human_readable: Option<String>,
  pub last_updated: DateTime<Utc>,
  pub logon_time: DateTime<Utc>,
}

impl PartialEq for Controller {
  // custom PartialEq for Controller as we don't care about last_updated
  // field as long as the others stay the same
  fn eq(&self, other: &Self) -> bool {
    self.cid == other.cid
      && self.name == other.name
      && self.callsign == other.callsign
      && self.freq == other.freq
      && self.facility == other.facility
      && self.rating == other.rating
      && self.server == other.server
      && self.visual_range == other.visual_range
      && self.atis_code == other.atis_code
      && self.text_atis == other.text_atis
      && self.human_readable == other.human_readable
      && self.logon_time == other.logon_time
  }
}

#[derive(Debug, Clone, Serialize, Default, PartialEq)]
pub struct ControllerSet {
  pub atis: Option<Controller>,
  pub delivery: Option<Controller>,
  pub ground: Option<Controller>,
  pub tower: Option<Controller>,
  pub approach: Option<Controller>,
}

impl ControllerSet {
  pub fn empty() -> Self {
    Self {
      atis: None,
      delivery: None,
      ground: None,
      tower: None,
      approach: None,
    }
  }

  pub fn is_empty(&self) -> bool {
    self.atis.is_none()
      && self.delivery.is_none()
      && self.ground.is_none()
      && self.tower.is_none()
      && self.approach.is_none()
  }
}

impl From<super::exttypes::Controller> for Controller {
  fn from(ctrl: super::exttypes::Controller) -> Self {
    let freq = ctrl.frequency.parse::<f64>().unwrap_or(0.0);
    let freq = freq * 1000.0;
    let freq = freq as u32;
    let facility: Facility = ctrl.facility.into();

    let text_atis = if let Some(ta) = ctrl.text_atis {
      ta.join("\n")
    } else {
      "".to_owned()
    };
    let logon_time = DateTime::parse_from_rfc3339(&ctrl.logon_time)
      .and_then(|dt| Ok(dt.with_timezone(&Utc)))
      .unwrap_or(Utc::now());
    let last_updated = DateTime::parse_from_rfc3339(&ctrl.last_updated)
      .and_then(|dt| Ok(dt.with_timezone(&Utc)))
      .unwrap_or(Utc::now());

    Self {
      cid: ctrl.cid,
      name: ctrl.name,
      callsign: ctrl.callsign,
      freq,
      facility,
      rating: ctrl.rating,
      server: ctrl.server,
      visual_range: ctrl.visual_range,
      atis_code: ctrl.atis_code.unwrap_or("".to_owned()),
      text_atis,
      last_updated,
      logon_time,
      human_readable: None,
    }
  }
}
