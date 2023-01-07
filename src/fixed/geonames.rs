use super::types::GeonamesCountry;
use crate::{
  config::Config,
  fixed::{
    cached_loader,
    types::{GeonamesShape, GeonamesShapeSet},
  },
  seconds_since,
};
use chrono::Utc;
use csv::StringRecord;
use geojson::{FeatureCollection, GeoJson, Value};
use log::{error, info};
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

pub async fn load_shapes(cfg: &Config) -> Result<Vec<GeonamesShape>, Box<dyn std::error::Error>> {
  let cache_file =
    cached_loader(&cfg.fixed.geonames_shapes_url, &cfg.cache.geonames_shapes).await?;
  let t = Utc::now();
  let mut z = ZipArchive::new(cache_file)?;
  let mut raw_data = String::new();

  let mut file = z.by_name("shapes_simplified_low.json")?;
  file.read_to_string(&mut raw_data)?;

  let geodata = raw_data.parse::<GeoJson>()?;
  info!("geonames geojson parsed in {}s", seconds_since(t));

  let mut shapes = vec![];
  let fc = FeatureCollection::try_from(geodata)?;
  for feature in fc {
    let gss: GeonamesShapeSet = feature.try_into()?;
    match gss {
      GeonamesShapeSet::Single(gs) => shapes.push(gs),
      GeonamesShapeSet::Multi(gsv) => shapes.extend(gsv),
    }
  }
  Ok(shapes)
}

#[cfg(test)]
pub mod tests {

  use super::*;
  use crate::fixed::types::GeonamesShape;

  #[tokio::test]
  async fn test_countries() {
    let cfg: Config = Default::default();
    let res = load_shapes(&cfg).await;
    if let Err(err) = res {
      println!("{}", err)
    }
  }

  #[tokio::test]
  async fn test_rtree() {
    let geo_id = "2960313";
    let cfg: Config = Default::default();
    let cache_file = cached_loader(&cfg.fixed.geonames_shapes_url, &cfg.cache.geonames_shapes)
      .await
      .unwrap();
    let mut z = ZipArchive::new(cache_file).unwrap();
    let mut raw_data = String::new();

    let mut file = z.by_name("shapes_simplified_low.json").unwrap();
    file.read_to_string(&mut raw_data).unwrap();

    let geodata = raw_data.parse::<GeoJson>().unwrap();
    let fc = FeatureCollection::try_from(geodata).unwrap();

    let feature = fc
      .into_iter()
      .find(|feat| {
        let props = feat.properties.as_ref().unwrap();
        let geoname_id = props.get("geoNameId").unwrap();
        geoname_id == geo_id
      })
      .unwrap();

    let geom = feature.geometry.unwrap();
    match geom.value {
      Value::MultiPolygon(mp) => {
        for p in mp.into_iter() {
          let gs = GeonamesShape::from_vec(geo_id, p);
          println!("{gs:?}");
        }
      }
      Value::Polygon(p) => {
        let gs = GeonamesShape::from_vec(geo_id, p);
        println!("{gs:?}");
      }
      v => {
        println!("value is {v:?}");
      }
    };
  }
}
