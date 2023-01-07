pub mod metrics;
pub mod spatial;

use self::{
  metrics::Metrics,
  spatial::{PointObject, RectObject},
};
use crate::{
  config::Config,
  fixed::{
    data::FixedData,
    geonames::{load_countries, load_shapes},
    parser::load_fixed,
    types::{Airport, GeonamesCountry, GeonamesShape, FIR},
  },
  labels,
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
use geo::algorithm::Contains;
use log::{debug, error, info};
use rstar::{RTree, AABB};
use std::collections::{HashMap, HashSet};
use tokio::{sync::RwLock, time::sleep};

const CLEANUP_EVERY_X_ITER: u8 = 5;

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

  gn_countries: RwLock<HashMap<String, GeonamesCountry>>,
  gn_shapes: RwLock<RTree<GeonamesShape>>,

  metrics: RwLock<Metrics>,
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
      let tracks = tracks.write().await;

      let res = tracks.indexes().await;
      if let Err(err) = res {
        error!("error creating track indices: {}", err);
      }

      info!("cleaning up tracks");
      let t = Utc::now();
      let res = tracks.cleanup().await;
      if let Err(err) = res {
        error!("error cleaning up: {}", err);
      } else {
        let process_time = seconds_since(t);
        info!("boot-time db cleanup took {process_time}s");
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
      metrics: RwLock::new(Metrics::new()),
      gn_countries: RwLock::new(HashMap::new()),
      gn_shapes: RwLock::new(RTree::new()),
    }
  }

  pub fn config(&self) -> &Config {
    &self.cfg
  }

  pub async fn render_metrics(&self) -> String {
    self.metrics.read().await.render()
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

  async fn setup_geonames_data(&self) -> Result<(), Box<dyn std::error::Error>> {
    let t = Utc::now();
    let geonames_countries = load_countries(&self.cfg).await?;
    {
      let mut gn = self.gn_countries.write().await;
      for (k, v) in geonames_countries.into_iter() {
        gn.insert(k, v);
      }
    }
    debug!("geonames countries processed in {}s", seconds_since(t));

    let t = Utc::now();
    let geonames_shapes = load_shapes(&self.cfg).await?;
    {
      // consider bulk loading for better startup performance
      let mut tree = self.gn_shapes.write().await;
      for shape in geonames_shapes {
        tree.insert(shape);
      }
    }
    debug!("geonames shapes processed in {}s", seconds_since(t));
    Ok(())
  }

  pub async fn search_country(&self, position: Point) -> Option<GeonamesCountry> {
    let pcoord: geo_types::Point = position.into();
    let envelope = AABB::from_point(pcoord);
    let tree = self.gn_shapes.read().await;
    let mut res = tree.locate_in_envelope_intersecting(&envelope);
    let geo_id = res
      .find(|gs| gs.poly.contains(&pcoord))
      .map(|gs| &gs.ref_id);
    if let Some(geo_id) = geo_id {
      self.gn_countries.read().await.get(geo_id).cloned()
    } else {
      None
    }
  }

  pub async fn run(&self) -> Result<(), Box<dyn std::error::Error>> {
    self.setup_fixed_data().await?;
    self.setup_geonames_data().await?;

    let mut pilots_callsigns = HashSet::new();
    let mut controllers: HashMap<String, Controller> = HashMap::new();
    let mut data_updated_at = 0;
    let mut cleanup = CLEANUP_EVERY_X_ITER;

    loop {
      info!("loading vatsim data");
      let t = Utc::now();
      let data = load_vatsim_data(&self.cfg).await;
      let process_time = seconds_since(t);
      self
        .metrics
        .write()
        .await
        .vatsim_data_load_time_sec
        .set_single(process_time);
      info!("vatsim data loaded in {}s", process_time);
      if let Some(data) = data {
        let ts = data.general.updated_at.timestamp();
        if ts > data_updated_at {
          data_updated_at = ts;
          self.metrics.write().await.vatsim_data_timestamp = ts;
          // region:pilots_processing
          let mut fresh_pilots_callsigns = HashSet::new();

          info!("processing pilots");
          let t = Utc::now();
          let pcount = data.pilots.len();

          let mut pilots_grouped = HashMap::new();
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

              let country = self.search_country(pilot.position).await;
              if let Some(country) = country {
                let counter = pilots_grouped.entry(country.geoname_id).or_insert(0);
                *counter += 1;
              }

              // We have to keep point objects in both hashmap and rtree
              // because rtree doesn't support searching by id:
              //
              // We need to search point objects by id for removing a pilot
              // from RTree "by id". We search for a point object in the HashMap
              // then we pass it to .remove() method of the tree where it's
              // being searched by coords and then checked with PartialEq,
              // so it's OK that the HashMap and RTree contain copies of the object.
              // See remove_pilot() method for details
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

          let process_time = seconds_since(t);
          {
            let mut metrics = self.metrics.write().await;
            let countries = self.gn_countries.read().await;
            metrics
              .processing_time_sec
              .set(labels!("object_type" = "pilot"), process_time);

            for (geo_id, count) in pilots_grouped {
              let country = countries.get(&geo_id).unwrap();
              metrics.vatsim_objects_online.set(
                labels!(
                  "object_type" = "pilot",
                  "country_code" = &country.iso,
                  "continent_code" = &country.continent
                ),
                count,
              );
            }
          }
          info!("{} pilots processed in {}s", pcount, process_time);
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

          let process_time = seconds_since(t);
          {
            let mut metrics = self.metrics.write().await;
            metrics
              .processing_time_sec
              .set(labels!("object_type" = "controller"), process_time);
            metrics
              .vatsim_objects_online
              .set(labels!("object_type" = "controller"), controllers.len());
          }
          info!("{} controllers processed in {}s", ccount, process_time);
          // endregion:controllers_processing
        }

        if let Some(tracks) = &self.tracks {
          let t = Utc::now();
          let res = tracks.read().await.counters().await;
          let process_time = seconds_since(t);
          match res {
            Ok((tc, tpc)) => {
              let mut metrics = self.metrics.write().await;
              metrics
                .database_objects_count
                .set(labels!("object_type" = "track"), tc);
              metrics
                .database_objects_count
                .set(labels!("object_type" = "trackpoint"), tpc);
              metrics
                .database_objects_count_fetch_time_sec
                .set_single(process_time);
            }
            Err(err) => {
              error!("error getting db counters: {err}");
            }
          }

          cleanup -= 1;
          if cleanup == 0 {
            let t = Utc::now();
            let res = tracks.write().await.cleanup().await;
            match res {
              Err(err) => error!("error cleaning up db: {err}"),
              Ok(_) => {
                let process_time = seconds_since(t);
                info!("db cleanup took {process_time}s");
                cleanup = CLEANUP_EVERY_X_ITER;
              }
            }
          } else {
            debug!("{cleanup} iterations to db cleanup");
          }
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
