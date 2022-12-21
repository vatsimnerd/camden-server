use rstar::AABB;
use serde::Serialize;

#[derive(Debug, Serialize, Clone, Copy, PartialEq)]
pub struct Point {
  pub lat: f64,
  pub lng: f64,
}

impl Point {
  pub fn clamp(&self) -> Self {
    Self {
      lat: self.lat.clamp(-90.0, 90.0),
      lng: (self.lng + 180.0).rem_euclid(360.0) - 180.0,
    }
  }

  pub fn envelope(self) -> AABB<Point> {
    AABB::from_point(self)
  }
}

impl rstar::Point for Point {
  type Scalar = f64;
  const DIMENSIONS: usize = 2;

  fn generate(mut generator: impl FnMut(usize) -> Self::Scalar) -> Self {
    let lng = generator(0);
    let lat = generator(1);
    Self { lat, lng }
  }

  fn nth(&self, index: usize) -> Self::Scalar {
    match index {
      0 => self.lng,
      1 => self.lat,
      _ => unreachable!(),
    }
  }

  fn nth_mut(&mut self, index: usize) -> &mut Self::Scalar {
    match index {
      0 => &mut self.lng,
      1 => &mut self.lat,
      _ => unreachable!(),
    }
  }
}

#[derive(Debug, Serialize, Clone, Copy)]
pub struct Rect {
  pub south_west: Point,
  pub north_east: Point,
}

impl Rect {
  pub fn new(min_lng: f64, min_lat: f64, max_lng: f64, max_lat: f64) -> Self {
    Self {
      south_west: Point {
        lng: min_lng,
        lat: min_lat,
      },
      north_east: Point {
        lng: max_lng,
        lat: max_lat,
      },
    }
  }

  fn width(&self) -> f64 {
    (self.north_east.lng + 180.0) - (self.south_west.lng + 180.0)
  }

  fn height(&self) -> f64 {
    self.north_east.lat - self.south_west.lat
  }

  pub fn scale(&self, multiplier: f64) -> Self {
    let ext = multiplier - 1.0;
    let lng_ext = self.width() * ext / 2.0;
    let lat_ext = self.height() * ext / 2.0;
    let south_west = Point {
      lat: self.south_west.lat - lat_ext,
      lng: self.south_west.lng - lng_ext,
    };
    let north_east = Point {
      lat: self.north_east.lat + lat_ext,
      lng: self.north_east.lng + lng_ext,
    };
    Self {
      south_west: south_west.clamp(),
      north_east: north_east.clamp(),
    }
  }

  pub fn envelope(&self) -> AABB<Point> {
    AABB::from_corners(self.south_west, self.north_east)
  }
}
