use crate::{error::LemmyResult, location_info};
use anyhow::{Context, anyhow};
use deser_hjson::from_str;
use std::{env, fs, sync::LazyLock};
use structs::{PictrsConfig, Settings};
use url::Url;

pub mod structs;

static DEFAULT_CONFIG_FILE: &str = "config/config.hjson";

/// Some connection options to speed up queries
const CONNECTION_OPTIONS: [&str; 1] = ["geqo_threshold=12"];

#[expect(clippy::expect_used)]
pub static SETTINGS: LazyLock<Settings> = LazyLock::new(|| {
  if env::var("LEMMY_INITIALIZE_WITH_DEFAULT_SETTINGS").is_ok() {
    println!(
      "LEMMY_INITIALIZE_WITH_DEFAULT_SETTINGS was set, any configuration file has been ignored."
    );
    println!(
      "Use with other environment variables to configure this instance further; e.g. LEMMY_DATABASE_URL."
    );
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
    if self.tls_enabled { "https" } else { "http" }
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
    inject_postgres_connection_options(&self.get_database_url(), &self.get_protocol_and_hostname())
  }
}

/// Appends Lemmy's Postgres `options=` settings without replacing existing query params
/// such as `sslmode=require`.
fn inject_postgres_connection_options(
  database_url: &str,
  protocol_and_hostname: &str,
) -> LemmyResult<String> {
  let mut url = Url::parse(database_url)?;

  // Set `lemmy.protocol_and_hostname` so triggers can use it
  let hostname_option = format!("lemmy.protocol_and_hostname={protocol_and_hostname}");
  let options_segments = CONNECTION_OPTIONS
    .iter()
    .copied()
    .chain(std::iter::once(hostname_option.as_str()))
    .map(|o| format!("-c {o}"))
    .collect::<Vec<_>>()
    .join(" ");

  // Append to the existing query string instead of replacing it. `Url::set_query` would
  // drop admin-configured params such as `sslmode=require`.
  // https://www.postgresql.org/docs/current/libpq-connect.html#LIBPQ-CONNSTRING
  url
    .query_pairs_mut()
    .append_pair("options", &options_segments);
  let result = url.to_string();

  if database_url.contains("sslmode=require") && !result.contains("sslmode=require") {
    return Err(anyhow!("sslmode=require was dropped from the database connection URL").into());
  }
  Ok(result)
}
#[expect(clippy::expect_used)]
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

  #[test]
  fn test_database_url_preserves_sslmode() -> LemmyResult<()> {
    let url = inject_postgres_connection_options(
      "postgres://lemmy:password@localhost:5432/lemmy?sslmode=require",
      "https://example.com",
    )?;
    assert!(
      url.contains("sslmode=require"),
      "sslmode=require must survive options injection: {url}"
    );
    assert!(url.contains("options="), "options= must be appended: {url}");
    Ok(())
  }

  #[test]
  fn test_database_url_appends_options() -> LemmyResult<()> {
    let url = inject_postgres_connection_options(
      "postgres://lemmy:password@localhost:5432/lemmy",
      "https://example.com",
    )?;
    assert!(url.contains("options="), "{url}");
    assert!(url.contains("geqo_threshold"), "{url}");
    assert!(url.contains("lemmy.protocol_and_hostname"), "{url}");
    Ok(())
  }
}
