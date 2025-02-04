use crate::{error::LemmyResult, location_info};
use anyhow::{anyhow, Context};
use deser_hjson::from_str;
use regex::Regex;
use std::{env, fs, sync::LazyLock};
use structs::{PictrsConfig, Settings};
use url::Url;

pub mod structs;

static DEFAULT_CONFIG_FILE: &str = "config/config.hjson";

#[allow(clippy::expect_used)]
pub static SETTINGS: LazyLock<Settings> = LazyLock::new(|| {
  if env::var("LEMMY_INITIALIZE_WITH_DEFAULT_SETTINGS").is_ok() {
    println!(
      "LEMMY_INITIALIZE_WITH_DEFAULT_SETTINGS was set, any configuration file has been ignored."
    );
    println!("Use with other environment variables to configure this instance further; e.g. LEMMY_DATABASE_URL.");
    Settings::default()
  } else {
    Settings::init().expect("Failed to load settings file, see documentation (https://join-lemmy.org/docs/en/administration/configuration.html).")
  }
});

#[allow(clippy::expect_used)]
static WEBFINGER_REGEX: LazyLock<Regex> = LazyLock::new(|| {
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
  pub(crate) fn init() -> LemmyResult<Self> {
    let path =
      env::var("LEMMY_CONFIG_LOCATION").unwrap_or_else(|_| DEFAULT_CONFIG_FILE.to_string());
    let plain = fs::read_to_string(path)?;
    let config = from_str::<Settings>(&plain)?;
    if config.hostname == "unset" {
      Err(anyhow!("Hostname variable is not set!").into())
    } else {
      Ok(config)
    }
  }

  pub fn get_database_url(&self) -> String {
    if let Ok(url) = env::var("LEMMY_DATABASE_URL") {
      url
    } else {
      self.database.connection.clone()
    }
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

  pub fn pictrs(&self) -> LemmyResult<PictrsConfig> {
    self
      .pictrs
      .clone()
      .ok_or_else(|| anyhow!("images_disabled").into())
  }
}
#[allow(clippy::expect_used)]
/// Necessary to avoid URL expect failures
fn pictrs_placeholder_url() -> Url {
  Url::parse("http://localhost:8080").expect("parse pictrs url")
}

#[cfg(test)]
mod tests {

  use super::*;

  #[test]
  fn test_load_config() -> LemmyResult<()> {
    Settings::init()?;
    Ok(())
  }
}
