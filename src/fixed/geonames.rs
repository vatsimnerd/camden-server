use crate::{config::Config, fixed::cached_loader, seconds_since};
use chrono::Utc;
use log::info;
use std::fs::File;

fn parse_countries(file: File) -> Result<(), Box<dyn std::error::Error>> {
  let mut rdr = csv::ReaderBuilder::new()
    .has_headers(true)
    .delimiter(b'\t')
    .flexible(true)
    .comment(Some(b'#'))
    .from_reader(file);

  for record in rdr.records() {
    let record = record?;
    println!("{:?}", record);
  }
  Ok(())
}

pub async fn load_countries(cfg: &Config) -> Result<(), Box<dyn std::error::Error>> {
  let cache_file = cached_loader(
    &cfg.fixed.geonames_countries_url,
    &cfg.cache.geonames_countries,
  )
  .await?;

  let t = Utc::now();
  let res = parse_countries(cache_file)?;
  info!("runways data parsed in {}s", seconds_since(t));
  Ok(res)
}

#[cfg(test)]
pub mod tests {

  use super::*;

  #[tokio::test]
  async fn test_countries() {
    let cfg: Config = Default::default();
    let res = load_countries(&cfg).await;
    assert!(res.is_ok())
  }
}
