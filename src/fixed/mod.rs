/// Fixed data provider
/// This includes vatspy-data-project's items like Countries, Airports,
/// FIRs and UIRs as well as ourairports' data on runways
mod boundaries;
pub mod data;
pub mod errors;
pub mod geonames;
pub mod ourairports;
pub mod parser;
pub mod types;

use std::{error::Error, fs::File, io::Write, path::Path};

use chrono::Utc;
use log::info;

use crate::seconds_since;

async fn cached_loader(url: &str, cache_filename: &str) -> Result<File, Box<dyn Error>> {
  let path = Path::new(&cache_filename);
  if !path.is_file() {
    info!("fetching {url} from web");
    let t = Utc::now();
    let data = reqwest::get(url).await?.bytes().await?;
    let mut cache_file = File::create(path)?;
    cache_file.write_all(&data)?;
    info!(
      "data loaded from web in {}s and stored in {cache_filename}",
      seconds_since(t)
    );
  } else {
    info!("{cache_filename} found, skipping fetching")
  }

  let f = File::open(path)?;
  Ok(f)
}
