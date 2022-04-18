use crate::{location_info, settings::structs::Settings, LemmyError};
use anyhow::{anyhow, Context};
use deser_hjson::from_str;
use once_cell::sync::Lazy;
use regex::{Regex, RegexBuilder};
use std::{env, fs, io::Error, sync::RwLock};

pub mod structs;

static DEFAULT_CONFIG_FILE: &str = "config/config.hjson";

static SETTINGS: Lazy<RwLock<Settings>> =
  Lazy::new(|| RwLock::new(Settings::init().expect("Failed to load settings file")));
static WEBFINGER_REGEX: Lazy<Regex> = Lazy::new(|| {
  Regex::new(&format!(
    "^acct:([a-zA-Z0-9_]{{3,}})@{}$",
    Settings::get().hostname
  ))
  .expect("compile webfinger regex")
});

impl Settings {
  /// Reads config from configuration file.
  ///
  /// Note: The env var `LEMMY_DATABASE_URL` is parsed in
  /// `lemmy_db_schema/src/lib.rs::get_database_url_from_env()`
  /// Warning: Only call this once.
  pub fn init() -> Result<Self, LemmyError> {
    // Read the config file
    let config = from_str::<Settings>(&Self::read_config_file()?)?;

    if config.hostname == "unset" {
      return Err(anyhow!("Hostname variable is not set!").into());
    }

    Ok(config)
  }

  /// Returns the config as a struct.
  pub fn get() -> Self {
    SETTINGS.read().expect("read config").to_owned()
  }

  pub fn get_database_url(&self) -> String {
    let conf = &self.database;
    format!(
      "postgres://{}:{}@{}:{}/{}",
      conf.user, conf.password, conf.host, conf.port, conf.database,
    )
  }

  pub fn get_config_location() -> String {
    env::var("LEMMY_CONFIG_LOCATION").unwrap_or_else(|_| DEFAULT_CONFIG_FILE.to_string())
  }

  pub fn read_config_file() -> Result<String, Error> {
    fs::read_to_string(Self::get_config_location())
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
    format!("{}://{}", self.get_protocol_string(), self.hostname)
  }

  /// When running the federation test setup in `api_tests/` or `docker/federation`, the `hostname`
  /// variable will be like `lemmy-alpha:8541`. This method removes the port and returns
  /// `lemmy-alpha` instead. It has no effect in production.
  pub fn get_hostname_without_port(&self) -> Result<String, anyhow::Error> {
    Ok(
      self
        .hostname
        .split(':')
        .collect::<Vec<&str>>()
        .first()
        .context(location_info!())?
        .to_string(),
    )
  }

  pub fn save_config_file(data: &str) -> Result<String, LemmyError> {
    // check that the config is valid
    from_str::<Settings>(data)?;

    fs::write(Settings::get_config_location(), data)?;

    // Reload the new settings
    // From https://stackoverflow.com/questions/29654927/how-do-i-assign-a-string-to-a-mutable-static-variable/47181804#47181804
    let mut new_settings = SETTINGS.write().expect("write config");
    *new_settings = match Settings::init() {
      Ok(c) => c,
      Err(e) => panic!("{}", e),
    };

    Ok(Self::read_config_file()?)
  }

  pub fn webfinger_regex(&self) -> Regex {
    WEBFINGER_REGEX.to_owned()
  }

  pub fn slur_regex(&self) -> Option<Regex> {
    self.slur_filter.as_ref().map(|slurs| {
      RegexBuilder::new(slurs)
        .case_insensitive(true)
        .build()
        .expect("compile regex")
    })
  }
}
