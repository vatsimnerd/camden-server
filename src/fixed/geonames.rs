use super::types::GeonamesCountry;
use crate::{config::Config, fixed::cached_loader, seconds_since};
use chrono::Utc;
use csv::StringRecord;
use geojson::{FeatureCollection, GeoJson, Value};
use log::info;
use rstar::RTree;
use std::{collections::HashMap, fs::File, io::Read};
use zip::ZipArchive;

fn parse_countries(
  file: File,
) -> Result<HashMap<u32, GeonamesCountry>, Box<dyn std::error::Error>> {
  let mut rdr = csv::ReaderBuilder::new()
    .has_headers(false)
    .delimiter(b'\t')
    .flexible(true)
    .comment(Some(b'#'))
    .from_reader(file);
  let headers = StringRecord::from(vec![
    "iso",
    "iso3",
    "iso_numeric",
    "fips",
    "name",
    "capital",
    "area",
    "population",
    "continent",
    "tld",
    "currency_code",
    "currency_name",
    "phone",
    "postal_code_format",
    "postal_code_regex",
    "languages",
    "geoname_id",
    "neighbours",
    "equivalent_fips_code",
  ]);
  rdr.set_headers(headers);
  let mut countries = HashMap::new();

  for res in rdr.deserialize() {
    if let Err(err) = res {
      println!("{err}, {:?}", err.position());
    } else {
      let country: GeonamesCountry = res.unwrap();
      countries.insert(country.geoname_id, country);
    }
  }
  Ok(countries)
}

pub async fn load_countries(
  cfg: &Config,
) -> Result<HashMap<u32, GeonamesCountry>, Box<dyn std::error::Error>> {
  let cache_file = cached_loader(
    &cfg.fixed.geonames_countries_url,
    &cfg.cache.geonames_countries,
  )
  .await?;

  let t = Utc::now();
  let countries = parse_countries(cache_file)?;
  info!("geonames countries parsed in {}s", seconds_since(t));
  Ok(countries)
}

pub async fn load_shapes(cfg: &Config) -> Result<GeoJson, Box<dyn std::error::Error>> {
  let cache_file =
    cached_loader(&cfg.fixed.geonames_shapes_url, &cfg.cache.geonames_shapes).await?;
  let t = Utc::now();
  let mut z = ZipArchive::new(cache_file)?;
  let mut raw_data = String::new();

  let mut file = z.by_name("shapes_simplified_low.json")?;
  file.read_to_string(&mut raw_data)?;

  let geodata = raw_data.parse::<GeoJson>()?;
  info!("geonames shapes parsed in {}s", seconds_since(t));

  let r: RTree<[f64; 2]> = RTree::new();

  let fc = FeatureCollection::try_from(geodata)?;
  for feature in fc {
    let geom = feature.geometry.as_ref();
    if let Some(geom) = geom {
      match &geom.value {
        Value::MultiPolygon(mp) => geo::MultiPolygon::from(geom),
        _ => println!("not a multipolygon"),
      }
    }
    println!("{:?}", feature);
  }
  Ok(raw_data.parse::<GeoJson>()?)
}

#[cfg(test)]
pub mod tests {

  use super::*;

  #[tokio::test]
  async fn test_countries() {
    let cfg: Config = Default::default();
    let res = load_shapes(&cfg).await;
    if let Err(err) = res {
      println!("{}", err)
    }
  }
}
