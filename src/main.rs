#[macro_use]
extern crate rocket;

use camden::config::read_config;
use camden::web::error::{catch404, catch500};
use camden::web::{check_query, get_pilot, updates};
use camden::{manager::manager::Manager, web::get_airport};
use log::error;
use simplelog::{ColorChoice, Config, TermLogger, TerminalMode};
use std::sync::Arc;

#[launch]
async fn rocket() -> _ {
  // TODO cmdline flag -c
  let config = read_config(None);

  TermLogger::init(
    config.log.level,
    Config::default(),
    TerminalMode::Stdout,
    ColorChoice::Always,
  )
  .unwrap();

  let m = Manager::new(config.clone()).await;
  let m = Arc::new(m);

  {
    let m = m.clone();
    tokio::spawn(async move {
      let res = m.run().await;
      if let Err(err) = res {
        error!("error running manager: {err:?}");
      }
    });
  }

  rocket::build()
    .manage(m)
    .mount(
      "/api",
      routes![updates, get_airport, get_pilot, check_query],
    )
    .register("/", catchers![catch404, catch500])
}
