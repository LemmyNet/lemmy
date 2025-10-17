use crate::{error::LemmyResult, location_info};
use anyhow::{anyhow, Context};
use deser_hjson::from_str;
use std::{env, fs, sync::LazyLock};
use structs::{PictrsConfig, Settings};
use url::Url;
use urlencoding::encode;

pub mod structs;

static DEFAULT_CONFIG_FILE: &str = "config/config.hjson";

/// Some connection options to speed up queries
const CONNECTION_OPTIONS: [&str; 1] = ["geqo_threshold=12"];

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
  fn get_protocol_string(&self) -> &'static str {
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

  pub fn pictrs(&self) -> LemmyResult<PictrsConfig> {
    self
      .pictrs
      .clone()
      .ok_or_else(|| anyhow!("images_disabled").into())
  }

  /// Sets a few additional config options necessary for starting lemmy
  pub fn get_database_url_with_options(&self) -> LemmyResult<String> {
    let mut url = Url::parse(&self.get_database_url())?;

    // Set `lemmy.protocol_and_hostname` so triggers can use it
    let lemmy_protocol_and_hostname_option =
      "lemmy.protocol_and_hostname=".to_owned() + &self.get_protocol_and_hostname();
    let mut options = CONNECTION_OPTIONS.to_vec();
    options.push(&lemmy_protocol_and_hostname_option);

    // Create the connection uri portion
    let options_segments = options
      .iter()
      // The equal signs need to be encoded, since the url set_query doesn't do them,
      // and postgres requires them to be %3D
      // https://www.postgresql.org/docs/current/libpq-connect.html#LIBPQ-CONNSTRING
      .map(|o| format!("-c {}", encode(o)))
      .collect::<Vec<String>>()
      .join(" ");

    url.set_query(Some(&format!("options={options_segments}")));
    Ok(url.into())
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
