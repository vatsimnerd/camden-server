mod calc;
pub mod error;
mod filter;
mod message;
mod types;

use self::{
  error::APIError,
  filter::compile_filter,
  message::UpdateMessage,
  types::{PilotApiResponse, QueryCheckOkResponse},
};
use crate::{
  fixed::types::Airport,
  lee::{make_expr, parser::expression::CompileFunc},
  manager::Manager,
  moving::pilot::Pilot,
  seconds_since,
  types::{Point, Rect},
};
use chrono::Utc;
use log::{debug, info};
use rocket::{
  get,
  response::stream::{Event, EventStream},
  serde::json::Json,
  Shutdown, State,
};
use serde::Serialize;
use std::{collections::HashMap, sync::Arc, time::Duration};
use tokio::{select, time::interval};
use uuid::Uuid;

// if zoom is less than this, the map might be wrapped on screen, thus we
// need to show all the objects without checking current user map boundaries
const MIN_ZOOM: f64 = 3.0;

// use curl http://localhost:8000/api/updates/-3.0/49.5/5.0/63.0/5 for testing
#[get("/updates/<min_lng>/<min_lat>/<max_lng>/<max_lat>/<zoom>?<query>&<show_wx>")]
#[allow(clippy::too_many_arguments)]
pub async fn updates(
  min_lng: f64,
  min_lat: f64,
  max_lng: f64,
  max_lat: f64,
  zoom: f64,
  query: Option<String>,
  show_wx: Option<bool>,
  manager: &State<Arc<Manager>>,
  mut end: Shutdown,
) -> Result<EventStream![Event + '_], APIError> {
  let client_id = Uuid::new_v4().to_string()[..18].to_owned();
  info!(
    "client {client_id} connected with bbox [{min_lng}, {min_lat}, {max_lng}, {max_lat}] zoom {zoom}"
  );
  let mut tm = interval(Duration::from_secs(5));

  let show_wx = match show_wx {
    Some(value) => value,
    None => false,
  };

  let rect = Rect {
    south_west: Point {
      lat: min_lat,
      lng: min_lng,
    },
    north_east: Point {
      lat: max_lat,
      lng: max_lng,
    },
  };
  let no_bounds = zoom < MIN_ZOOM;
  if no_bounds {
    info!("client {client_id} no_bounds flag set to true");
  }

  let mut pilots_state = HashMap::new();
  let mut airports_state = HashMap::new();
  let mut firs_state = HashMap::new();
  let f_expr = {
    if let Some(query) = query {
      let mut expr = make_expr::<Pilot>(query.as_str())?;
      let cb: Box<CompileFunc<Pilot>> = Box::new(compile_filter);
      expr.compile(&cb)?;
      Some(expr)
    } else {
      None
    }
  };

  #[allow(clippy::manual_retain)]
  Ok(EventStream! {
    loop {
      let messages = select! {
        _ = &mut end =>  {
          debug!("shutting down");
          break;
        },
        _ = &mut Box::pin(tm.tick()) => {
          let mut messages = vec![];

          let t = Utc::now();
          let mut pilots = if no_bounds {
            manager.get_all_pilots().await
          } else {
            manager.get_pilots(&rect).await
          };
          debug!("[{}] {} pilots loaded in {}s", client_id, pilots.len(), seconds_since(t));

          if let Some(f) = f_expr.as_ref() {
            pilots = pilots.iter().filter(|pilot| f.evaluate(pilot)).cloned().collect();
          }

          let t = Utc::now();
          let (pilots_set, pilots_delete) = calc::calc_pilots(&pilots, &mut pilots_state);
          debug!("[{}] {} pilots diff calculated in {}s, set={}/del={}", client_id, pilots.len(), seconds_since(t), pilots_set.len(), pilots_delete.len());

          if pilots_set.len() > 100 {
            for chunk in pilots_set.chunks(100) {
              messages.push(UpdateMessage::pilots_set(&client_id, chunk.to_vec()));
            }
          } else {
            messages.push(UpdateMessage::pilots_set(&client_id, pilots_set));
          }

          messages.push(UpdateMessage::pilots_delete(&client_id, pilots_delete));

          let t = Utc::now();
          let airports = if no_bounds {
            manager.get_all_airports(show_wx).await
          } else {
            manager.get_airports(&rect, show_wx).await
          };
          debug!("[{}] {} airports loaded in {}s", client_id, airports.len(), seconds_since(t));
          let t = Utc::now();
          let (arpts_set, arpts_delete) = calc::calc_airports(&airports, &mut airports_state);
          debug!("[{}] {} airports diff calculated in {}s, set={}/del={}", client_id, airports.len(), seconds_since(t), arpts_set.len(), arpts_delete.len());

          messages.push(UpdateMessage::airports_set(&client_id, arpts_set));
          messages.push(UpdateMessage::airports_delete(&client_id, arpts_delete));

          let t = Utc::now();
          let firs = if no_bounds {
            manager.get_all_firs().await
          } else {
            manager.get_firs(&rect).await
          };

          debug!("[{}] {} firs loaded in {}s", client_id, firs.len(), seconds_since(t));
          let t = Utc::now();
          let (firs_set, firs_delete) = calc::calc_firs(&firs, &mut firs_state);
          debug!("[{}] {} firs diff calculated in {}s, set={}/del={}", client_id, firs.len(), seconds_since(t), firs_set.len(), firs_delete.len());

          messages.push(UpdateMessage::firs_set(&client_id, firs_set));
          messages.push(UpdateMessage::firs_delete(&client_id, firs_delete));

          messages
        }
      };

      if !messages.is_empty() {
        debug!("[{}] generated {} messages", client_id, messages.len())
      }

      for msg in messages {
        if !msg.data.is_empty() {
          yield Event::json(&msg);
        }
      }
    }
  })
}

#[get("/airports/<code>")]
pub async fn get_airport(code: String, manager: &State<Arc<Manager>>) -> Option<Json<Airport>> {
  manager.find_airport(&code).await.map(Json)
}

#[get("/pilots/<callsign>")]
pub async fn get_pilot(
  callsign: String,
  manager: &State<Arc<Manager>>,
) -> Result<Option<Json<PilotApiResponse>>, APIError> {
  let pilot = manager.get_pilot_by_callsign(&callsign).await;
  if let Some(pilot) = pilot {
    let tps = manager.get_pilot_track(&pilot).await?;
    let mut resp: PilotApiResponse = pilot.into();
    resp.track = tps;
    Ok(Some(Json(resp)))
  } else {
    Ok(None)
  }
}

#[get("/chkquery?<query>")]
pub async fn check_query(query: String) -> Result<Json<QueryCheckOkResponse>, APIError> {
  let mut expr = make_expr::<Pilot>(query.as_str())?;
  let cb: Box<CompileFunc<Pilot>> = Box::new(compile_filter);
  expr.compile(&cb)?;
  Ok(Json(QueryCheckOkResponse { status: "ok" }))
}

#[derive(Serialize)]
pub struct BuildInfo {
  name: String,
  version: String,
  repository: String,
  license: String,
}

#[get("/__build__")]
pub async fn build_info() -> Json<BuildInfo> {
  let pkgname = env!("CARGO_PKG_NAME").to_owned();
  let pkgversion = env!("CARGO_PKG_VERSION").to_owned();
  let repository = env!("CARGO_PKG_REPOSITORY").to_owned();
  let license_file = env!("CARGO_PKG_LICENSE_FILE").to_owned();
  Json(BuildInfo {
    name: pkgname,
    version: pkgversion,
    repository,
    license: license_file,
  })
}

#[get("/metrics")]
pub async fn metrics(manager: &State<Arc<Manager>>) -> String {
  manager.render_metrics().await
}
