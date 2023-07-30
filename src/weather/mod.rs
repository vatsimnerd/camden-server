mod ext_types;

use std::collections::HashMap;

use self::ext_types::{Metar, WindDirection};
use chrono::{DateTime, Duration, Utc};
use log::{debug, error, info};
use reqwest::Client;
use serde::Serialize;

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct WeatherInfo {
  pub temperature: Option<f64>,
  pub dew_point: Option<f64>,
  pub wind_speed: Option<u64>,
  pub wind_gust: Option<u64>,
  pub wind_direction: Option<WindDirection>,
  pub raw: String,
  pub ts: DateTime<Utc>,
}

impl From<Metar> for WeatherInfo {
  fn from(value: Metar) -> Self {
    Self {
      temperature: value.temp,
      dew_point: value.dewp,
      wind_speed: value.wspd,
      wind_gust: value.wgst,
      wind_direction: value.wdir,
      raw: value.raw_ob,
      ts: value.receipt_time,
    }
  }
}

#[derive(Debug)]
struct BlackListItem {
  set_at: DateTime<Utc>,
  duration: Duration,
}

impl BlackListItem {
  pub fn new() -> Self {
    Self {
      set_at: Utc::now(),
      duration: Duration::seconds(3600),
    }
  }

  pub fn double(&self) -> Self {
    Self {
      set_at: Utc::now(),
      duration: self.duration * 2,
    }
  }

  pub fn expired(&self) -> bool {
    let now = Utc::now();
    now > self.set_at + self.duration
  }
}

#[derive(Debug)]
pub struct WeatherManager {
  metar_ttl: Duration,
  cache: HashMap<String, WeatherInfo>,
  blacklist: HashMap<String, BlackListItem>,
  apireq_num: usize,
}

impl WeatherManager {
  pub fn new(metar_ttl: Duration) -> Self {
    Self {
      metar_ttl,
      cache: Default::default(),
      blacklist: Default::default(),
      apireq_num: 0,
    }
  }

  pub fn request_num(&self) -> usize {
    self.apireq_num
  }

  fn has_valid_cache_for(&self, location: &str) -> bool {
    let value = self.cache.get(location);
    if let Some(value) = value {
      let now = Utc::now();
      let delta = now - value.ts;
      delta < self.metar_ttl
    } else {
      false
    }
  }

  fn is_blacklisted(&self, location: &str) -> bool {
    let blitem = self.blacklist.get(location);
    match blitem {
      Some(blitem) => !blitem.expired(),
      None => false,
    }
  }

  pub async fn preload(&mut self, locations: Vec<&str>) {
    let locations: Vec<&str> = locations
      .into_iter()
      .filter(|loc| !self.is_blacklisted(loc) && !self.has_valid_cache_for(loc))
      .collect();

    if locations.is_empty() {
      return;
    }

    let locations = locations.join(",");
    info!("preloading weather for {locations}");

    let path = format!(
      "https://beta.aviationweather.gov/cgi-bin/data/metar.php?ids={locations}&format=json"
    );
    let client = Client::new();

    self.apireq_num += 1;
    let res = client.get(path).send().await;

    if let Err(err) = res {
      error!("error loading wx data: {err}");
      return;
    }

    let res = res.unwrap().json::<Vec<Metar>>().await;
    if let Err(err) = res {
      error!("error parsing wx data: {err}");
      return;
    }

    let metars = res.unwrap();
    for metar in metars {
      let loc = metar.icao_id.clone();
      self.cache.insert(loc, metar.into());
    }
  }

  fn get_cache(&self, location: &str) -> Option<WeatherInfo> {
    debug!("collecting weather for {location} from cache");
    let value = self.cache.get(location).cloned()?;
    let now = Utc::now();
    let delta = now - value.ts;
    if delta > self.metar_ttl {
      None
    } else {
      Some(value)
    }
  }

  async fn get_remote(&mut self, location: &str) -> Option<WeatherInfo> {
    let blitem = self.blacklist.get(location);
    if let Some(blitem) = blitem {
      if !blitem.expired() {
        debug!("location {location} is blacklisted");
        return None;
      }
    }
    info!("collecting weather for {location} from remote api");

    let path =
      format!("https://beta.aviationweather.gov/cgi-bin/data/metar.php?ids={location}&format=json");
    let client = Client::new();

    self.apireq_num += 1;
    let res = client.get(path).send().await;

    if let Err(err) = res {
      error!("error loading {location} wx data: {err}");
      return None;
    }

    let metar = res.unwrap().json::<Vec<Metar>>().await;
    if let Err(err) = metar {
      error!("error parsing {location} wx data: {err}");
      return None;
    }

    let metar = metar.unwrap().get(0).cloned();
    if let Some(metar) = metar {
      Some(metar.into())
    } else {
      error!("got empty array of wx data at {location}");
      let blitem = self.blacklist.get(location);
      let blitem = match blitem {
        Some(blitem) => blitem.double(),
        None => BlackListItem::new(),
      };
      debug!("blacklisting {location} for {}", blitem.duration);
      self.blacklist.insert(location.to_owned(), blitem);
      None
    }
  }

  pub async fn get(&mut self, location: &str) -> Option<WeatherInfo> {
    let wx = self.get_cache(location);
    if let Some(wx) = wx {
      Some(wx)
    } else {
      let wx = self.get_remote(location).await;
      if let Some(wx) = wx {
        self.cache.insert(location.to_owned(), wx.clone());
        Some(wx)
      } else {
        None
      }
    }
  }
}
