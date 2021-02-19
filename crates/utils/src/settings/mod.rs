use crate::{location_info, settings::structs::Settings, LemmyError};
use anyhow::Context;
use deser_hjson::from_str;
use merge::Merge;
use std::{env, fs, io::Error, sync::RwLock};

pub mod defaults;
pub mod structs;

static CONFIG_FILE: &str = "config/config.hjson";

lazy_static! {
  static ref SETTINGS: RwLock<Settings> = RwLock::new(match Settings::init() {
    Ok(c) => c,
    Err(e) => panic!("{}", e),
  });
}

impl Settings {
  /// Reads config from the files and environment.
  /// First, defaults are loaded from CONFIG_FILE_DEFAULTS, then these values can be overwritten
  /// from CONFIG_FILE (optional). Finally, values from the environment (with prefix LEMMY) are
  /// added to the config.
  ///
  /// Note: The env var `LEMMY_DATABASE_URL` is parsed in
  /// `lemmy_db_queries/src/lib.rs::get_database_url_from_env()`
  fn init() -> Result<Self, LemmyError> {
    // Read the config file
    let mut custom_config = from_str::<Settings>(&Self::read_config_file()?)?;

    // Merge with default
    custom_config.merge(Settings::default());

    Ok(custom_config)
  }

  /// Returns the config as a struct.
  pub fn get() -> Self {
    SETTINGS.read().unwrap().to_owned()
  }

  pub fn get_database_url(&self) -> String {
    let conf = self.database.to_owned().unwrap_or_default();
    format!(
      "postgres://{}:{}@{}:{}/{}",
      conf.user, conf.password, conf.host, conf.port, conf.database,
    )
  }

  pub fn get_config_location() -> String {
    env::var("LEMMY_CONFIG_LOCATION").unwrap_or_else(|_| CONFIG_FILE.to_string())
  }

  pub fn read_config_file() -> Result<String, Error> {
    fs::read_to_string(Self::get_config_location())
  }

  pub fn get_allowed_instances(&self) -> Vec<String> {
    let mut allowed_instances: Vec<String> = self
      .federation
      .to_owned()
      .unwrap_or_default()
      .allowed_instances
      .unwrap_or_default()
      .split(',')
      .map(|d| d.trim().to_string())
      .collect();

    allowed_instances.retain(|d| !d.eq(""));
    allowed_instances
  }

  pub fn get_blocked_instances(&self) -> Vec<String> {
    let mut blocked_instances: Vec<String> = self
      .federation
      .to_owned()
      .unwrap_or_default()
      .blocked_instances
      .unwrap_or_default()
      .split(',')
      .map(|d| d.trim().to_string())
      .collect();

    blocked_instances.retain(|d| !d.eq(""));
    blocked_instances
  }

  /// Returns either "http" or "https", depending on tls_enabled setting
  pub fn get_protocol_string(&self) -> &'static str {
    if let Some(tls_enabled) = self.tls_enabled {
      if tls_enabled {
        "https"
      } else {
        "http"
      }
    } else {
      "http"
    }
  }

  /// Returns something like `http://localhost` or `https://lemmy.ml`,
  /// with the correct protocol and hostname.
  pub fn get_protocol_and_hostname(&self) -> String {
    format!(
      "{}://{}",
      self.get_protocol_string(),
      self.hostname.to_owned().unwrap_or_default()
    )
  }

  /// When running the federation test setup in `api_tests/` or `docker/federation`, the `hostname`
  /// variable will be like `lemmy-alpha:8541`. This method removes the port and returns
  /// `lemmy-alpha` instead. It has no effect in production.
  pub fn get_hostname_without_port(&self) -> Result<String, anyhow::Error> {
    Ok(
      self
        .hostname
        .to_owned()
        .unwrap_or_default()
        .split(':')
        .collect::<Vec<&str>>()
        .first()
        .context(location_info!())?
        .to_string(),
    )
  }

  pub fn save_config_file(data: &str) -> Result<String, Error> {
    fs::write(CONFIG_FILE, data)?;

    // Reload the new settings
    // From https://stackoverflow.com/questions/29654927/how-do-i-assign-a-string-to-a-mutable-static-variable/47181804#47181804
    let mut new_settings = SETTINGS.write().unwrap();
    *new_settings = match Settings::init() {
      Ok(c) => c,
      Err(e) => panic!("{}", e),
    };

    Self::read_config_file()
  }
}
