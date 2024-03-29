use chrono::{DateTime, TimeZone, Utc};
use serde::{Deserialize, Deserializer, Serialize};

const WHY_IS_IT_EVEN_CUSTOM_FORMAT: &'static str = "%Y-%m-%d %H:%M:%S";

pub fn deserialize_datetime<'de, D>(deserializer: D) -> Result<DateTime<Utc>, D::Error>
where
  D: Deserializer<'de>,
{
  let s = String::deserialize(deserializer)?;
  Utc
    .datetime_from_str(&s, WHY_IS_IT_EVEN_CUSTOM_FORMAT)
    .map_err(serde::de::Error::custom)
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
#[serde(untagged)]
pub enum WindDirection {
  Variable(String),
  Degree(u64),
}

#[derive(Deserialize, Debug, Clone)]
pub struct Metar {
  pub metar_id: u64,
  #[serde(rename(deserialize = "icaoId"))]
  pub icao_id: String,
  #[serde(
    rename(deserialize = "receiptTime"),
    deserialize_with = "deserialize_datetime"
  )]
  pub receipt_time: DateTime<Utc>,
  #[serde(
    rename(deserialize = "reportTime"),
    deserialize_with = "deserialize_datetime"
  )]
  pub report_time: DateTime<Utc>,
  pub temp: Option<f64>,
  pub dewp: Option<f64>,
  pub wdir: Option<WindDirection>,
  pub wspd: Option<u64>,
  pub wgst: Option<u64>,
  #[serde(rename(deserialize = "rawOb"))]
  pub raw_ob: String,
}

#[cfg(test)]
pub mod tests {
  use super::*;

  #[tokio::test]
  async fn test_struct() {
    let res =
      reqwest::get("https://beta.aviationweather.gov/cgi-bin/data/metar.php?ids=EGLL&format=json")
        .await;

    if let Err(err) = res {
      println!("{err}");
      return;
    }

    let resp = res.unwrap();
    let text = resp.text().await.unwrap();

    let res = serde_json::from_str::<Vec<Metar>>(&text);
    match res {
      Ok(data) => println!("{data:?}"),
      Err(err) => println!("{err}"),
    }
  }
}
