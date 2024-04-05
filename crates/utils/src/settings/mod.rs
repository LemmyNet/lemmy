use crate::{error::LemmyError, location_info};
use anyhow::{anyhow, Context};
use deser_hjson::from_str;
use once_cell::sync::Lazy;
use regex::Regex;
use std::{env, fs, io::Error};
use urlencoding::encode;

pub mod structs;

use structs::{DatabaseConnection, PictrsConfig, PictrsImageMode, Settings};

static DEFAULT_CONFIG_FILE: &str = "config/config.hjson";

pub static SETTINGS: Lazy<Settings> = Lazy::new(|| {
  if env::var("LEMMY_INITIALIZE_WITH_DEFAULT_SETTINGS").is_ok() {
    println!("LEMMY_INITIALIZE_WITH_DEFAULT_SETTINGS was set, any configuration file has been ignored");
    println!("Use with other environment variables to configure this instance further; e.g. LEMMY_DATABASE_URL");
    return Settings::default();
  }
  Settings::init().expect("Failed to load settings file, see documentation (https://join-lemmy.org/docs/en/administration/configuration.html)")
});

static WEBFINGER_REGEX: Lazy<Regex> = Lazy::new(|| {
  Regex::new(&format!(
    "^acct:([a-zA-Z0-9_]{{3,}})@{}$",
    SETTINGS.hostname
  ))
  .expect("compile webfinger regex")
});

impl Settings {
  /// Reads config from configuration file.
  ///
  /// Note: The env var `LEMMY_DATABASE_URL` is parsed in
  /// `lemmy_db_schema/src/lib.rs::get_database_url_from_env()`
  /// Warning: Only call this once.
  pub(crate) fn init() -> Result<Self, LemmyError> {
    let config = from_str::<Settings>(&Self::read_config_file()?)?;
    if config.hostname == "unset" {
      Err(anyhow!("Hostname variable is not set!").into())
    } else {
      Ok(config)
    }
  }

  pub fn get_database_url(&self) -> String {
    if let Ok(url) = env::var("LEMMY_DATABASE_URL") {
      return url;
    }
    match &self.database.connection {
      DatabaseConnection::Uri { uri } => uri.clone(),
      DatabaseConnection::Parts(parts) => {
        format!(
          "postgres://{}:{}@{}:{}/{}",
          encode(&parts.user),
          encode(&parts.password),
          parts.host,
          parts.port,
          encode(&parts.database),
        )
      }
    }
  }

  fn get_config_location() -> String {
    env::var("LEMMY_CONFIG_LOCATION").unwrap_or_else(|_| DEFAULT_CONFIG_FILE.to_string())
  }

  fn read_config_file() -> Result<String, Error> {
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
      (*self
        .hostname
        .split(':')
        .collect::<Vec<&str>>()
        .first()
        .context(location_info!())?)
      .to_string(),
    )
  }

  pub fn webfinger_regex(&self) -> Regex {
    WEBFINGER_REGEX.clone()
  }

  pub fn pictrs_config(&self) -> Result<PictrsConfig, LemmyError> {
    self
      .pictrs
      .clone()
      .ok_or_else(|| anyhow!("images_disabled").into())
  }
}

impl PictrsConfig {
  pub fn image_mode(&self) -> PictrsImageMode {
    if let Some(cache_external_link_previews) = self.cache_external_link_previews {
      if cache_external_link_previews {
        PictrsImageMode::StoreLinkPreviews
      } else {
        PictrsImageMode::None
      }
    } else {
      self.image_mode.clone()
    }
  }
}
