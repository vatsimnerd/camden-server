mod calc;
pub mod error;
mod filter;
mod message;
mod types;

use self::{
  error::APIError,
  filter::compile_filter,
  message::{ObjectsSet, Update, UpdateMessage},
  types::{PilotApiResponse, QueryCheckOkResponse},
};
use crate::{
  fixed::types::Airport,
  lee::{make_expr, parser::expression::CompileFunc},
  manager::Manager,
  moving::pilot::Pilot,
  seconds_since,
  types::Point,
};
use chrono::Utc;
use log::{debug, info};
use rocket::{
  get,
  response::stream::{Event, EventStream},
  serde::json::Json,
  Shutdown, State,
};
use rstar::AABB;
use std::{collections::HashMap, sync::Arc, time::Duration};
use tokio::{select, time::interval};
use uuid::Uuid;

// use curl http://localhost:8000/api/updates/-3.0/49.5/5.0/63.0 for testing
#[get("/updates/<min_lng>/<min_lat>/<max_lng>/<max_lat>?<query>")]
pub async fn updates(
  min_lng: f64,
  min_lat: f64,
  max_lng: f64,
  max_lat: f64,
  query: Option<String>,
  manager: &State<Arc<Manager>>,
  mut end: Shutdown,
) -> Result<EventStream![Event + '_], APIError> {
  let client_id = Uuid::new_v4().to_string()[..18].to_owned();
  info!(
    "client {} connected with bbox [{}, {}, {}, {}]",
    client_id, min_lng, min_lat, max_lng, max_lat
  );
  let mut tm = interval(Duration::from_secs(5));

  let env = AABB::from_corners(
    Point {
      lat: min_lat,
      lng: min_lng,
    },
    Point {
      lat: max_lat,
      lng: max_lng,
    },
  );

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

  Ok(EventStream! {
    loop {
      let msg = select! {
        _ = &mut end =>  {
          debug!("shutting down");
          break;
        },
        _ = &mut Box::pin(tm.tick()) => {
          let t = Utc::now();
          let mut pilots = manager.get_pilots(&env).await;
          debug!("[{}] {} pilots loaded in {}s", client_id, pilots.len(), seconds_since(t));

          if let Some(f) = f_expr.as_ref() {
            pilots = pilots.iter().filter(|pilot| f.evaluate(pilot)).cloned().collect();
          }

          let t = Utc::now();
          let (pilots_set, pilots_delete) = calc::calc_pilots(&pilots, &mut pilots_state);
          debug!("[{}] {} pilots diff calculated in {}s, set={}/del={}", client_id, pilots.len(), seconds_since(t), pilots_set.len(), pilots_delete.len());

          let t = Utc::now();
          let airports = manager.get_airports(&env).await;
          debug!("[{}] {} airports loaded in {}s", client_id, airports.len(), seconds_since(t));
          let t = Utc::now();
          let (arpts_set, arpts_delete) = calc::calc_airports(&airports, &mut airports_state);
          debug!("[{}] {} airports diff calculated in {}s, set={}/del={}", client_id, airports.len(), seconds_since(t), arpts_set.len(), arpts_delete.len());

          let t = Utc::now();
          let firs = manager.get_firs(&env).await;
          debug!("[{}] {} firs loaded in {}s", client_id, firs.len(), seconds_since(t));
          let t = Utc::now();
          let (firs_set, firs_delete) = calc::calc_firs(&firs, &mut firs_state);
          debug!("[{}] {} firs diff calculated in {}s, set={}/del={}", client_id, firs.len(), seconds_since(t), firs_set.len(), firs_delete.len());

          UpdateMessage {
            connection_id: client_id.clone(),
            message_type: "update",
            data: Update {
              set: ObjectsSet {
                pilots: pilots_set,
                airports: arpts_set,
                firs: firs_set,
              },
              delete: ObjectsSet {
                pilots: pilots_delete,
                airports: arpts_delete,
                firs: firs_delete,
              }
            }
          }
        }
      };
      if !msg.data.is_empty() {
        yield Event::json(&msg);
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
