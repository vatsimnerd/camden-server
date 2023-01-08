use chrono::{DateTime, Utc};

pub mod atis;
pub mod config;
pub mod fixed;
pub mod lee;
pub mod manager;
pub mod moving;
pub mod persistent;
pub mod types;
pub mod util;
pub mod web;

pub fn seconds_since(t: DateTime<Utc>) -> f32 {
  let t2 = Utc::now();
  let d = (t2 - t).to_std();
  if let Ok(d) = d {
    d.as_secs_f32()
  } else {
    0.0
  }
}
