use crate::{location_info, LemmyError};
use anyhow::Context;
use deser_hjson::from_str;
use merge::Merge;
use serde::Deserialize;
use std::{
  env,
  fs,
  io::Error,
  net::{IpAddr, Ipv4Addr},
  sync::RwLock,
};

static CONFIG_FILE: &str = "config/config.hjson";

#[derive(Debug, Deserialize, Clone, Merge)]
pub struct Settings {
  pub setup: Option<Setup>,
  pub database: Option<DatabaseConfig>,
  pub hostname: Option<String>,
  pub bind: Option<IpAddr>,
  pub port: Option<u16>,
  pub tls_enabled: Option<bool>,
  pub jwt_secret: Option<String>,
  pub pictrs_url: Option<String>,
  pub iframely_url: Option<String>,
  pub rate_limit: Option<RateLimitConfig>,
  pub email: Option<EmailConfig>,
  pub federation: Option<FederationConfig>,
  pub captcha: Option<CaptchaConfig>,
}

impl Default for Settings {
  fn default() -> Self {
    Self {
      database: Some(DatabaseConfig::default()),
      rate_limit: Some(RateLimitConfig::default()),
      federation: Some(FederationConfig::default()),
      captcha: Some(CaptchaConfig::default()),
      email: None,
      setup: None,
      hostname: None,
      bind: Some(IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0))),
      port: Some(8536),
      tls_enabled: Some(true),
      jwt_secret: Some("changeme".into()),
      pictrs_url: Some("http://pictrs:8080".into()),
      iframely_url: Some("http://iframely".into()),
    }
  }
}

#[derive(Debug, Deserialize, Clone)]
pub struct Setup {
  pub admin_username: String,
  pub admin_password: String,
  pub admin_email: Option<String>,
  pub site_name: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct RateLimitConfig {
  pub message: i32,
  pub message_per_second: i32,
  pub post: i32,
  pub post_per_second: i32,
  pub register: i32,
  pub register_per_second: i32,
  pub image: i32,
  pub image_per_second: i32,
}

impl Default for RateLimitConfig {
  fn default() -> Self {
    Self {
      message: 180,
      message_per_second: 60,
      post: 6,
      post_per_second: 600,
      register: 3,
      register_per_second: 3600,
      image: 6,
      image_per_second: 3600,
    }
  }
}

#[derive(Debug, Deserialize, Clone)]
pub struct EmailConfig {
  pub smtp_server: String,
  pub smtp_login: Option<String>,
  pub smtp_password: Option<String>,
  pub smtp_from_address: String,
  pub use_tls: bool,
}

#[derive(Debug, Deserialize, Clone)]
pub struct CaptchaConfig {
  pub enabled: bool,
  pub difficulty: String,
}

impl Default for CaptchaConfig {
  fn default() -> Self {
    Self {
      enabled: true,
      difficulty: "medium".into(),
    }
  }
}

#[derive(Debug, Deserialize, Clone)]
pub struct DatabaseConfig {
  pub user: String,
  pub password: String,
  pub host: String,
  pub port: i32,
  pub database: String,
  pub pool_size: u32,
}

impl Default for DatabaseConfig {
  fn default() -> Self {
    Self {
      user: "lemmy".into(),
      password: "password".into(),
      host: "localhost".into(),
      port: 5432,
      database: "lemmy".into(),
      pool_size: 5,
    }
  }
}

#[derive(Debug, Deserialize, Clone)]
pub struct FederationConfig {
  pub enabled: bool,
  pub allowed_instances: Option<String>,
  pub blocked_instances: Option<String>,
}

impl Default for FederationConfig {
  fn default() -> Self {
    Self {
      enabled: false,
      allowed_instances: Some("".into()),
      blocked_instances: Some("".into()),
    }
  }
}

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
