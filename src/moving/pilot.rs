use chrono::{DateTime, Utc};
use serde::Serialize;

use crate::types::Point;

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct Pilot {
  pub cid: u32,
  pub name: String,
  pub callsign: String,
  pub server: String,
  pub pilot_rating: i32,
  pub position: Point,
  pub altitude: i32,
  pub groundspeed: i32,
  pub transponder: String,
  pub heading: i16,
  pub qnh_i_hg: u16,
  pub qnh_mb: u16,
  pub flight_plan: Option<FlightPlan>,
  pub logon_time: DateTime<Utc>,
  pub last_updated: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct FlightPlan {
  pub flight_rules: String,
  pub aircraft: String,
  pub departure: String,
  pub arrival: String,
  pub alternate: String,
  pub cruise_tas: u16,
  pub altitude: u16,
  pub deptime: String,
  pub enroute_time: String,
  pub fuel_time: String,
  pub remarks: String,
  pub route: String,
}

impl From<crate::moving::exttypes::FlightPlan> for FlightPlan {
  fn from(src: crate::moving::exttypes::FlightPlan) -> Self {
    // Use this type converter to normalise FlightPlan data and
    // fix user errors

    let cruise_tas = src.cruise_tas.parse::<u16>().unwrap_or(0);
    let altitude = src.altitude.parse::<u16>().unwrap_or(0);

    Self {
      flight_rules: src.flight_rules,
      aircraft: src.aircraft,
      departure: src.departure,
      arrival: src.arrival,
      alternate: src.alternate,
      cruise_tas,
      altitude,
      deptime: src.deptime,
      enroute_time: src.enroute_time,
      fuel_time: src.fuel_time,
      remarks: src.remarks,
      route: src.route,
    }
  }
}

impl Pilot {
  pub fn track_code(&self) -> String {
    format!(
      "{}:{}:{}",
      self.cid,
      self.callsign,
      self.logon_time.timestamp()
    )
  }
}

impl From<crate::moving::exttypes::Pilot> for Pilot {
  fn from(src: crate::moving::exttypes::Pilot) -> Self {
    let qnh_i_hg = (src.qnh_i_hg * 100.0).round() as u16;

    let logon_time = DateTime::parse_from_rfc3339(&src.logon_time)
      .and_then(|dt| Ok(dt.with_timezone(&Utc)))
      .unwrap_or(Utc::now());
    let last_updated = DateTime::parse_from_rfc3339(&src.last_updated)
      .and_then(|dt| Ok(dt.with_timezone(&Utc)))
      .unwrap_or(Utc::now());

    let flight_plan = src.flight_plan.and_then(|fp| Some(fp.into()));

    Self {
      cid: src.cid,
      name: src.name,
      callsign: src.callsign,
      server: src.server,
      pilot_rating: src.pilot_rating,
      position: Point {
        lat: src.latitude,
        lng: src.longitude,
      },
      altitude: src.altitude,
      groundspeed: src.groundspeed,
      transponder: src.transponder,
      heading: src.heading,
      qnh_i_hg,
      qnh_mb: src.qnh_mb as u16,
      flight_plan,
      logon_time,
      last_updated,
    }
  }
}
