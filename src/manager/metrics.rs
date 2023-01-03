use chrono::Utc;

#[derive(Default, Debug)]
pub struct Metrics {
  pub pilots_online: usize,
  pub controllers_online: usize,
  pub track_count: u64,
  pub track_point_count: u64,
  pub vatsim_data_timestamp: i64,
  pub vatsim_data_load_time_sec: f32,
  pub pilots_processing_time_sec: f32,
  pub controllers_processing_time_sec: f32,
  pub db_cleanup_time_sec: f32,
}

impl Metrics {
  pub fn new() -> Self {
    Default::default()
  }

  pub fn render(&self) -> String {
    let t = Utc::now().timestamp();
    let mut metrics = vec![];

    metrics.push(format!(
      r#"vatsim_objects_online{{type="pilot"}} {}"#,
      self.pilots_online
    ));
    metrics.push(format!(
      r#"vatsim_objects_online{{type="controller"}} {}"#,
      self.controllers_online
    ));
    metrics.push(format!(
      r#"database_objects_count{{type="track"}} {}"#,
      self.track_count
    ));
    metrics.push(format!(
      r#"database_objects_count{{type="trackpoint"}} {}"#,
      self.track_point_count
    ));

    let age = t - self.vatsim_data_timestamp;
    metrics.push(format!("vatsim_data_age_sec {age}"));
    metrics.push(format!(
      "vatsim_data_load_time_sec {}",
      self.vatsim_data_load_time_sec
    ));
    metrics.push(format!("db_cleanup_time_sec {}", self.db_cleanup_time_sec));

    metrics.join("\n") + "\n"
  }
}
