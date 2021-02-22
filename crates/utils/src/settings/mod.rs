use crate::{
  location_info,
  settings::{
    environment::parse_from_env,
    merge::Merge,
    structs::Settings,
    structs_opt::SettingsOpt,
  },
  LemmyError,
};
use anyhow::{anyhow, Context};
use deser_hjson::from_str;
use std::{env, fs, sync::RwLock};

pub mod defaults;
mod environment;
mod merge;
pub mod structs;
mod structs_opt;

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
    let mut config = Settings::default();

    // Read the config file
    if let Some(config_file) = &Self::read_config_file() {
      config = config.merge(from_str::<SettingsOpt>(config_file)?);
    }

    // Read env vars
    config = config.merge(parse_from_env());

    if config.hostname == Settings::default().hostname {
      return Err(anyhow!("Hostname variable is not set!").into());
    }

    Ok(config)
  }

  /// Returns the config as a struct.
  pub fn get() -> Self {
    SETTINGS.read().unwrap().to_owned()
  }

  pub fn get_database_url(&self) -> String {
    let conf = self.database.to_owned();
    format!(
      "postgres://{}:{}@{}:{}/{}",
      conf.user, conf.password, conf.host, conf.port, conf.database,
    )
  }

  pub fn get_config_location() -> String {
    env::var("LEMMY_CONFIG_LOCATION").unwrap_or_else(|_| CONFIG_FILE.to_string())
  }

  pub fn read_config_file() -> Option<String> {
    fs::read_to_string(Self::get_config_location()).ok()
  }

  pub fn get_allowed_instances(&self) -> Vec<String> {
    let mut allowed_instances: Vec<String> = self
      .federation
      .to_owned()
      .allowed_instances
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
      .blocked_instances
      .split(',')
      .map(|d| d.trim().to_string())
      .collect();

    blocked_instances.retain(|d| !d.eq(""));
    blocked_instances
  }

  /// Returns either "http" or "https", depending on tls_enabled setting
  pub fn get_protocol_string(&self) -> &'static str {
    if self.tls_enabled {
      "https"
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
      self.hostname.to_owned()
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
        .split(':')
        .collect::<Vec<&str>>()
        .first()
        .context(location_info!())?
        .to_string(),
    )
  }

  pub fn save_config_file(data: &str) -> Result<String, LemmyError> {
    fs::write(CONFIG_FILE, data)?;

    // Reload the new settings
    // From https://stackoverflow.com/questions/29654927/how-do-i-assign-a-string-to-a-mutable-static-variable/47181804#47181804
    let mut new_settings = SETTINGS.write().unwrap();
    *new_settings = match Settings::init() {
      Ok(c) => c,
      Err(e) => panic!("{}", e),
    };

    Self::read_config_file().ok_or(anyhow!("Failed to read config").into())
  }
}
