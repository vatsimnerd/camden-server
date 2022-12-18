use chrono::{DateTime, Utc};
use serde::Serialize;

use crate::{
  moving::pilot::{FlightPlan, Pilot},
  track::TrackPoint,
  types::Point,
};

#[derive(Serialize)]
pub struct PilotApiResponse {
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
  pub track: Option<Vec<TrackPoint>>,
}

impl From<Pilot> for PilotApiResponse {
  fn from(p: Pilot) -> Self {
    Self {
      cid: p.cid,
      name: p.name,
      callsign: p.callsign,
      server: p.server,
      pilot_rating: p.pilot_rating,
      position: p.position,
      altitude: p.altitude,
      groundspeed: p.groundspeed,
      transponder: p.transponder,
      heading: p.heading,
      qnh_i_hg: p.qnh_i_hg,
      qnh_mb: p.qnh_mb,
      flight_plan: p.flight_plan,
      logon_time: p.logon_time,
      last_updated: p.last_updated,
      track: None,
    }
  }
}

#[derive(Debug, Serialize)]
pub struct QueryCheckOkResponse {
  pub status: &'static str,
}
