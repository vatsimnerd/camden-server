use serde::Serialize;

#[derive(Debug, Serialize, Clone, Copy, PartialEq)]
pub struct Point {
  pub lat: f64,
  pub lng: f64,
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
}
