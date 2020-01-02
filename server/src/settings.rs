extern crate lazy_static;
use config::{Config, ConfigError, Environment, File};
use serde::Deserialize;
use std::env;
use std::net::IpAddr;

static CONFIG_FILE_DEFAULTS: &str = "config/defaults.hjson";
static CONFIG_FILE: &str = "config/config.hjson";

#[derive(Debug, Deserialize)]
pub struct Settings {
  pub database: Database,
  pub hostname: String,
  pub bind: IpAddr,
  pub port: u16,
  pub jwt_secret: String,
  pub front_end_dir: String,
  pub rate_limit: RateLimitConfig,
  pub email: Option<EmailConfig>,
  pub federation_enabled: bool,
}

#[derive(Debug, Deserialize)]
pub struct RateLimitConfig {
  pub message: i32,
  pub message_per_second: i32,
  pub post: i32,
  pub post_per_second: i32,
  pub register: i32,
  pub register_per_second: i32,
}

#[derive(Debug, Deserialize)]
pub struct EmailConfig {
  pub smtp_server: String,
  pub smtp_login: String,
  pub smtp_password: String,
  pub smtp_from_address: String,
}

#[derive(Debug, Deserialize)]
pub struct Database {
  pub user: String,
  pub password: String,
  pub host: String,
  pub port: i32,
  pub database: String,
  pub pool_size: u32,
}

lazy_static! {
  static ref SETTINGS: Settings = {
    match Settings::init() {
      Ok(c) => c,
      Err(e) => panic!("{}", e),
    }
  };
}

impl Settings {
  /// Reads config from the files and environment.
  /// First, defaults are loaded from CONFIG_FILE_DEFAULTS, then these values can be overwritten
  /// from CONFIG_FILE (optional). Finally, values from the environment (with prefix LEMMY) are
  /// added to the config.
  fn init() -> Result<Self, ConfigError> {
    let mut s = Config::new();

    s.merge(File::with_name(CONFIG_FILE_DEFAULTS))?;

    s.merge(File::with_name(CONFIG_FILE).required(false))?;

    // Add in settings from the environment (with a prefix of LEMMY)
    // Eg.. `LEMMY_DEBUG=1 ./target/app` would set the `debug` key
    // Note: we need to use double underscore here, because otherwise variables containing
    //       underscore cant be set from environmnet.
    // https://github.com/mehcode/config-rs/issues/73
    s.merge(Environment::with_prefix("LEMMY").separator("__"))?;

    s.try_into()
  }

  /// Returns the config as a struct.
  pub fn get() -> &'static Self {
    &SETTINGS
  }

  /// Returns the postgres connection url. If LEMMY_DATABASE_URL is set, that is used,
  /// otherwise the connection url is generated from the config.
  pub fn get_database_url(&self) -> String {
    match env::var("LEMMY_DATABASE_URL") {
      Ok(url) => url,
      Err(_) => format!(
        "postgres://{}:{}@{}:{}/{}",
        self.database.user,
        self.database.password,
        self.database.host,
        self.database.port,
        self.database.database
      ),
    }
  }

  pub fn api_endpoint(&self) -> String {
    format!("{}/api/v1", self.hostname)
  }
}
