use crate::{location_info, settings::structs::Settings, LemmyError};
use anyhow::{anyhow, Context};
use deser_hjson::from_str;
use regex::{Regex, RegexBuilder};
use std::{env, fs, io::Error};

pub mod structs;

static CONFIG_FILE: &str = "config/config.hjson";

impl Settings {
  /// Reads config from configuration file.
  ///
  /// Note: The env var `LEMMY_DATABASE_URL` is parsed in
  /// `lemmy_db_queries/src/lib.rs::get_database_url_from_env()`
  /// Warning: Only call this once.
  pub fn init() -> Result<Self, LemmyError> {
    // Read the config file
    let mut config = from_str::<Settings>(&Self::read_config_file()?)?;

    if config.hostname == "unset" {
      return Err(anyhow!("Hostname variable is not set!").into());
    }

    // Initialize the regexes
    config.webfinger_community_regex = Some(
      Regex::new(&format!("^group:([a-z0-9_]{{3,}})@{}$", config.hostname))
        .expect("compile webfinger regex"),
    );
    config.webfinger_username_regex = Some(
      Regex::new(&format!("^acct:([a-z0-9_]{{3,}})@{}$", config.hostname))
        .expect("compile webfinger regex"),
    );

    Ok(config)
  }

  pub fn get_database_url(&self) -> String {
    let conf = &self.database;
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
    fs::write(CONFIG_FILE, data)?;
    Ok(Self::read_config_file()?)
  }

  pub fn webfinger_community_regex(&self) -> Regex {
    self
      .webfinger_community_regex
      .to_owned()
      .expect("compile webfinger regex")
  }

  pub fn webfinger_username_regex(&self) -> Regex {
    self
      .webfinger_username_regex
      .to_owned()
      .expect("compile webfinger regex")
  }

  pub fn slur_regex(&self) -> Regex {
    let mut slurs = r"(fag(g|got|tard)?\b|cock\s?sucker(s|ing)?|ni((g{2,}|q)+|[gq]{2,})[e3r]+(s|z)?|mudslime?s?|kikes?|\bspi(c|k)s?\b|\bchinks?|gooks?|bitch(es|ing|y)?|whor(es?|ing)|\btr(a|@)nn?(y|ies?)|\b(b|re|r)tard(ed)?s?)".to_string();
    if let Some(additional_slurs) = &self.additional_slurs {
      slurs.push('|');
      slurs.push_str(additional_slurs);
    };
    RegexBuilder::new(&slurs)
      .case_insensitive(true)
      .build()
      .expect("compile regex")
  }
}
