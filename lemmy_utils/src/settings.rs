use config::{Config, ConfigError, Environment, File};
use serde::Deserialize;
use std::{env, fs, io::Error, net::IpAddr, path::PathBuf, sync::RwLock};

static CONFIG_FILE_DEFAULTS: &str = "config/defaults.hjson";
static CONFIG_FILE: &str = "config/config.hjson";

#[derive(Debug, Deserialize, Clone)]
/// Lemmy's Settings
pub struct Settings {
  /// An optional provided setup
  pub setup: Option<Setup>,
  /// The database config
  pub database: DatabaseConfig,
  /// The server hostname
  pub hostname: String,
  /// The server IP address
  pub bind: IpAddr,
  /// The server port
  pub port: u16,
  /// Whether TLS is enabled
  pub tls_enabled: bool,
  /// The docs dir
  pub docs_dir: PathBuf,
  /// The JWT secret
  pub jwt_secret: String,
  /// The pict-rs url
  pub pictrs_url: String,
  /// The iframely url
  pub iframely_url: String,
  /// The rate limit config
  pub rate_limit: RateLimitConfig,
  /// The email config
  pub email: Option<EmailConfig>,
  /// The federation config
  pub federation: FederationConfig,
  /// The Captcha config
  pub captcha: CaptchaConfig,
}

#[derive(Debug, Deserialize, Clone)]
/// An optional provided setup
pub struct Setup {
  /// The admin username
  pub admin_username: String,
  /// The admin password
  pub admin_password: String,
  /// The admin email
  pub admin_email: Option<String>,
  /// The site name
  pub site_name: String,
}

#[derive(Debug, Deserialize, Clone)]
/// The rate limit config
pub struct RateLimitConfig {
  /// The message count
  pub message: i32,
  /// messages / second
  pub message_per_second: i32,
  /// The post count
  pub post: i32,
  /// posts / second
  pub post_per_second: i32,
  /// The register count
  pub register: i32,
  /// registers / second
  pub register_per_second: i32,
  /// The image count
  pub image: i32,
  /// images / second
  pub image_per_second: i32,
}

#[derive(Debug, Deserialize, Clone)]
/// The email config
pub struct EmailConfig {
  /// The smtp server
  pub smtp_server: String,
  /// The smtp login
  pub smtp_login: Option<String>,
  /// The smtp password
  pub smtp_password: Option<String>,
  /// The smtp from address
  pub smtp_from_address: String,
  /// Whether to use TLS
  pub use_tls: bool,
}

#[derive(Debug, Deserialize, Clone)]
/// The captcha config
pub struct CaptchaConfig {
  // TODO Is this necessary? CaptchaConfig is optional, so it could just check the existence
  // there
  /// Whether its enabled
  pub enabled: bool,
  /// The captcha difficulty, can be easy, medium, or hard
  pub difficulty: String,
}

#[derive(Debug, Deserialize, Clone)]
/// The Database Config
pub struct DatabaseConfig {
  /// The username
  pub user: String,
  /// The password
  pub password: String,
  /// The DB host
  pub host: String,
  /// The DB port
  pub port: i32,
  /// The database
  pub database: String,
  /// The pool size
  pub pool_size: u32,
}

#[derive(Debug, Deserialize, Clone)]
/// The federation config
pub struct FederationConfig {
  /// is federation enabled
  pub enabled: bool, // TODO again do we need this, the top level is optional
  /// A comma-delimited list of allowed instances (leave empty if you have blocked instances)
  pub allowed_instances: String,
  /// A comma-delimited list of blocked instances
  pub blocked_instances: String,
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
  /// `lemmy_db/src/lib.rs::get_database_url_from_env()`
  fn init() -> Result<Self, ConfigError> {
    let mut s = Config::new();

    s.merge(File::with_name(&Self::get_config_defaults_location()))?;

    s.merge(File::with_name(&Self::get_config_location()).required(false))?;

    // Add in settings from the environment (with a prefix of LEMMY)
    // Eg.. `LEMMY_DEBUG=1 ./target/app` would set the `debug` key
    // Note: we need to use double underscore here, because otherwise variables containing
    //       underscore cant be set from environmnet.
    // https://github.com/mehcode/config-rs/issues/73
    s.merge(Environment::with_prefix("LEMMY").separator("__"))?;

    s.try_into()
  }

  /// Returns the config as a struct.
  pub fn get() -> Self {
    SETTINGS.read().unwrap().to_owned()
  }

  /// The database url
  pub fn get_database_url(&self) -> String {
    format!(
      "postgres://{}:{}@{}:{}/{}",
      self.database.user,
      self.database.password,
      self.database.host,
      self.database.port,
      self.database.database
    )
  }

  /// The config file defaults location
  pub fn get_config_defaults_location() -> String {
    env::var("LEMMY_CONFIG_DEFAULTS_LOCATION").unwrap_or_else(|_| CONFIG_FILE_DEFAULTS.to_string())
  }

  /// The config file location
  pub fn get_config_location() -> String {
    env::var("LEMMY_CONFIG_LOCATION").unwrap_or_else(|_| CONFIG_FILE.to_string())
  }

  /// Read the config file
  pub fn read_config_file() -> Result<String, Error> {
    fs::read_to_string(Self::get_config_location())
  }

  /// Get a list of allowed instances
  pub fn get_allowed_instances(&self) -> Vec<String> {
    let mut allowed_instances: Vec<String> = self
      .federation
      .allowed_instances
      .split(',')
      .map(|d| d.trim().to_string())
      .collect();

    // The defaults.hjson config always returns a [""]
    allowed_instances.retain(|d| !d.eq(""));

    allowed_instances
  }

  /// Get a list of blocked instances
  pub fn get_blocked_instances(&self) -> Vec<String> {
    let mut blocked_instances: Vec<String> = self
      .federation
      .blocked_instances
      .split(',')
      .map(|d| d.trim().to_string())
      .collect();

    // The defaults.hjson config always returns a [""]
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

  /// Returns something like `http://localhost` or `https://dev.lemmy.ml`,
  /// with the correct protocol and hostname.
  pub fn get_protocol_and_hostname(&self) -> String {
    format!("{}://{}", self.get_protocol_string(), self.hostname)
  }

  /// Save the config file
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
