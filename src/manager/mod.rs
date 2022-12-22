pub mod spatial;

use self::spatial::{PointObject, RectObject};
use crate::{
  config::Config,
  fixed::{
    data::FixedData,
    parser::load_fixed,
    types::{Airport, FIR},
  },
  moving::{
    controller::{Controller, Facility},
    load_vatsim_data,
    pilot::Pilot,
  },
  seconds_since,
  track::{TrackPoint, TrackStore},
  types::Point,
};
use chrono::Utc;
use log::{error, info};
use rstar::{RTree, AABB};
use std::collections::{HashMap, HashSet};
use tokio::{sync::RwLock, time::sleep};

#[derive(Debug)]
pub struct Manager {
  cfg: Config,
  fixed: RwLock<FixedData>,
  pilots: RwLock<HashMap<String, Pilot>>,
  pilots2d: RwLock<RTree<PointObject>>,
  pilots_po: RwLock<HashMap<String, PointObject>>,

  airports2d: RwLock<RTree<PointObject>>,
  firs2d: RwLock<RTree<RectObject>>,
  tracks: Option<RwLock<TrackStore>>,
}

impl Manager {
  pub async fn new(cfg: Config) -> Self {
    info!("setting vatsim data manager up");

    let res = TrackStore::new(&cfg).await;

    if let Err(err) = &res {
      error!("error creating track store: {}", err)
    }

    let tracks = res.ok().map(RwLock::new);

    if let Some(tracks) = &tracks {
      info!("creating track indices");
      let res = tracks.write().await.indexes().await;
      if let Err(err) = res {
        error!("error creating track indices: {}", err)
      }
    }

    Self {
      cfg,
      fixed: RwLock::new(FixedData::empty()),
      pilots: RwLock::new(HashMap::new()),
      pilots2d: RwLock::new(RTree::new()),
      pilots_po: RwLock::new(HashMap::new()),
      airports2d: RwLock::new(RTree::new()),
      firs2d: RwLock::new(RTree::new()),
      tracks,
    }
  }

  pub fn config(&self) -> &Config {
    &self.cfg
  }

  pub async fn get_pilots(&self, env: &AABB<Point>) -> Vec<Pilot> {
    let pilots2d = self.pilots2d.read().await;
    let pilots_idx = self.pilots.read().await;
    let mut pilots = vec![];

    for po in pilots2d.locate_in_envelope(env) {
      let pilot = pilots_idx.get(&po.id);
      if let Some(pilot) = pilot {
        pilots.push(pilot.clone());
      }
    }
    pilots
  }

  pub async fn get_airports(&self, env: &AABB<Point>) -> Vec<Airport> {
    let airports2d = self.airports2d.read().await;
    let fixed = self.fixed.read().await;
    let mut airports = vec![];

    for po in airports2d.locate_in_envelope(env) {
      let airport = fixed.find_airport_compound(&po.id);
      if let Some(airport) = airport {
        if !airport.controllers.is_empty() {
          airports.push(airport)
        }
      }
    }
    airports
  }

  pub async fn get_firs(&self, env: &AABB<Point>) -> Vec<FIR> {
    let firs2d = self.firs2d.read().await;
    let fixed = self.fixed.read().await;
    let mut firs = HashMap::new();

    for po in firs2d.locate_in_envelope_intersecting(env) {
      let fir_list = fixed.find_firs(&po.id);
      for fir in fir_list.into_iter().filter(|f| !f.is_empty()) {
        firs.insert(fir.icao.clone(), fir);
      }
    }
    firs.into_values().collect()
  }

  pub async fn find_airport(&self, code: &str) -> Option<Airport> {
    self.fixed.read().await.find_airport(code)
  }

  async fn setup_fixed_data(&self) -> Result<(), Box<dyn std::error::Error>> {
    info!("loading fixed data");
    let fixed = load_fixed(&self.cfg).await?; // TODO retries
    for arpt in fixed.airports() {
      self.airports2d.write().await.insert(arpt.into());
    }
    for fir in fixed.firs() {
      self.firs2d.write().await.insert(fir.into())
    }
    self.fixed.write().await.fill(fixed);
    info!("fixed data configured");
    Ok(())
  }

  async fn remove_pilot(&self, callsign: &str) -> bool {
    let po = { self.pilots_po.write().await.remove(callsign) };
    if let Some(po) = po {
      self.pilots2d.write().await.remove(&po);
      self.pilots.write().await.remove(callsign);
      true
    } else {
      false
    }
  }

  pub async fn run(&self) -> Result<(), Box<dyn std::error::Error>> {
    self.setup_fixed_data().await?;

    let mut pilots_callsigns = HashSet::new();
    let mut controllers: HashMap<String, Controller> = HashMap::new();
    let mut data_updated_at = 0;
    loop {
      info!("loading vatsim data");
      let t = Utc::now();
      let data = load_vatsim_data(&self.cfg).await;
      info!("vatsim data loaded in {}s", seconds_since(t));
      if let Some(data) = data {
        let ts = data.general.updated_at.timestamp();
        if ts > data_updated_at {
          data_updated_at = ts;
          // region:pilots_processing
          let mut fresh_pilots_callsigns = HashSet::new();

          info!("processing pilots");
          let t = Utc::now();
          let pcount = data.pilots.len();

          {
            for pilot in data.pilots.into_iter() {
              // avoid duplication in rtree
              self.remove_pilot(&pilot.callsign).await;

              // collecting pilots callsigns to find those disappeared since
              // the previous iteration
              fresh_pilots_callsigns.insert(pilot.callsign.clone());

              let po: PointObject = (&pilot).into();

              let mut pilots2d = self.pilots2d.write().await;
              let mut pilots_po = self.pilots_po.write().await;
              let mut pilots = self.pilots.write().await;

              // tracking first, to avoid additional cloning while inserting into hashmap later
              if let Some(tracks) = &self.tracks {
                let res = tracks.write().await.store(&pilot).await;
                if let Err(err) = res {
                  error!("error storing pilot track: {}", err);
                }
              }

              // we have to keep point objects in both hashmap and rtree
              // because rtree doesn't support searching by id
              pilots2d.insert(po.clone());
              pilots_po.insert(pilot.callsign.clone(), po);
              pilots.insert(pilot.callsign.clone(), pilot);
            }
          }

          // for each callsign not met this iteration let's remove it from the indexes
          for cs in pilots_callsigns.difference(&fresh_pilots_callsigns) {
            self.remove_pilot(cs).await;
          }

          // setup this iteration as "previous"
          pilots_callsigns = fresh_pilots_callsigns;

          info!("{} pilots processed in {}s", pcount, seconds_since(t));
          // endregion:pilots_processing

          // region:controllers_processing
          info!("processing controllers");
          let t = Utc::now();
          let mut fresh_controllers = HashMap::new();
          let mut ccount = 0;
          for ctrl in data.controllers.into_iter() {
            match &ctrl.facility {
              Facility::Reject => {
                continue;
              }
              Facility::Radar => {
                fresh_controllers.insert(ctrl.callsign.clone(), ctrl.clone());
                self.fixed.write().await.set_fir_controller(ctrl)
              }
              _ => {
                fresh_controllers.insert(ctrl.callsign.clone(), ctrl.clone());
                self.fixed.write().await.set_airport_controller(ctrl);
              }
            }
            ccount += 1;
          }

          for (cs, ctrl) in controllers.iter() {
            if !fresh_controllers.contains_key(cs) {
              match ctrl.facility {
                Facility::Radar => self.fixed.write().await.reset_fir_controller(ctrl),
                _ => {
                  self.fixed.write().await.reset_airport_controller(ctrl);
                }
              }
            }
          }
          controllers = fresh_controllers;

          info!("{} controllers processed in {}s", ccount, seconds_since(t));
          // endregion:controllers_processing
        }
        sleep(self.cfg.api.poll_period).await;
      }
    }
  }

  pub async fn get_pilot_by_callsign(&self, callsign: &str) -> Option<Pilot> {
    self.pilots.read().await.get(callsign).cloned()
  }

  pub async fn get_pilot_track(
    &self,
    pilot: &Pilot,
  ) -> Result<Option<Vec<TrackPoint>>, mongodb::error::Error> {
    if let Some(tracks) = &self.tracks {
      tracks.read().await.get_track_points(pilot).await
    } else {
      Ok(None)
    }
  }
}
