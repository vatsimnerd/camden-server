use rstar::{RTreeObject, AABB};

use crate::{
  fixed::types::{Airport, FIR},
  moving::pilot::Pilot,
  types::{Point, Rect},
};

#[derive(Debug, Clone)]
pub struct PointObject {
  pub id: String,
  point: Point,
}

impl RTreeObject for PointObject {
  type Envelope = AABB<Point>;

  fn envelope(&self) -> Self::Envelope {
    AABB::from_point(self.point)
  }
}

impl From<&Airport> for PointObject {
  fn from(arpt: &Airport) -> Self {
    Self {
      id: arpt.compound_id(),
      point: arpt.position,
    }
  }
}

impl From<&Pilot> for PointObject {
  fn from(pilot: &Pilot) -> Self {
    Self {
      id: pilot.callsign.clone(),
      point: pilot.position,
    }
  }
}

impl PartialEq for PointObject {
  fn eq(&self, other: &Self) -> bool {
    self.id == other.id
  }
}

#[derive(Debug, Clone)]
pub struct RectObject {
  pub id: String,
  rect: Rect,
}

impl RTreeObject for RectObject {
  type Envelope = AABB<Point>;

  fn envelope(&self) -> Self::Envelope {
    AABB::from_corners(self.rect.south_west, self.rect.north_east)
  }
}

impl From<&FIR> for RectObject {
  fn from(fir: &FIR) -> Self {
    Self {
      id: fir.icao.clone(),
      rect: Rect {
        south_west: fir.boundaries.min,
        north_east: fir.boundaries.max,
      },
    }
  }
}

impl PartialEq for RectObject {
  fn eq(&self, other: &Self) -> bool {
    self.id == other.id
  }
}
