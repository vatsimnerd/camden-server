use bcrypt::{hash, verify, BcryptError};
use bson::{oid::ObjectId, DateTime};
use mongodb::{bson::doc, IndexModel};
use serde::{Deserialize, Serialize};

use super::Model;

#[derive(Debug, Serialize, Deserialize)]
pub struct Track {
  pub _id: Option<ObjectId>,
  pub code: String,
  pub created_at: DateTime,
}

impl<'de> Model<'de> for Track {
  fn collection() -> &'static str {
    "tracks"
  }

  fn indexdefs() -> Vec<mongodb::IndexModel> {
    vec![IndexModel::builder()
      .keys(doc! {
        "code": 1
      })
      .build()]
  }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TrackPoint {
  #[serde(skip_serializing)]
  pub _id: Option<ObjectId>,
  #[serde(skip_serializing)]
  pub track_id: ObjectId,

  pub lat: f64,
  pub lng: f64,
  pub alt: i32,
  pub hdg: i16,
  pub gs: i32,
  pub ts: i64,
}

impl<'de> Model<'de> for TrackPoint {
  fn collection() -> &'static str {
    "track_points"
  }

  fn indexdefs() -> Vec<mongodb::IndexModel> {
    vec![IndexModel::builder()
      .keys(doc! {
        "track_id": 1,
        "ts": 1,
      })
      .build()]
  }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct User {
  #[serde(skip_serializing)]
  pub _id: Option<ObjectId>,
  pub ext_id: String,
  pub email: String,
  pub passwd_hash: String,
}

impl<'de> Model<'de> for User {
  fn collection() -> &'static str {
    "users"
  }

  fn indexdefs() -> Vec<IndexModel> {
    vec![IndexModel::builder()
      .keys(doc! {
        "email": 1,
      })
      .build()]
  }
}

impl User {
  pub fn create(email: &str, ext_id: &str, passwd: &str) -> Result<Self, BcryptError> {
    let passwd_hash = hash(passwd, 15)?;
    Ok(Self {
      _id: None,
      ext_id: ext_id.to_owned(),
      email: email.to_owned(),
      passwd_hash,
    })
  }

  pub fn check(&self, passwd: &str) -> bool {
    verify(passwd, &self.passwd_hash).unwrap_or(false)
  }
}
