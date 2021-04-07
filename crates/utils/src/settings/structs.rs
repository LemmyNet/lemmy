use merge::Merge;
use serde::Deserialize;
use std::net::IpAddr;

#[derive(Debug, Deserialize, Clone, Merge)]
pub struct Settings {
  pub(crate) database: Option<DatabaseConfig>,
  pub(crate) rate_limit: Option<RateLimitConfig>,
  pub(crate) federation: Option<FederationConfig>,
  pub(crate) hostname: Option<String>,
  pub(crate) bind: Option<IpAddr>,
  pub(crate) port: Option<u16>,
  pub(crate) tls_enabled: Option<bool>,
  pub(crate) jwt_secret: Option<String>,
  pub(crate) pictrs_url: Option<String>,
  pub(crate) iframely_url: Option<String>,
  pub(crate) captcha: Option<CaptchaConfig>,
  pub(crate) email: Option<EmailConfig>,
  pub(crate) setup: Option<SetupConfig>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct CaptchaConfig {
  pub enabled: bool,
  pub difficulty: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct DatabaseConfig {
  pub(super) user: Option<String>,
  pub password: String,
  pub host: String,
  pub(super) port: Option<i32>,
  pub(super) database: Option<String>,
  pub(super) pool_size: Option<u32>,
}

impl DatabaseConfig {
  pub fn user(&self) -> String {
    self
      .user
      .to_owned()
      .unwrap_or_else(|| DatabaseConfig::default().user.expect("missing user"))
  }
  pub fn port(&self) -> i32 {
    self
      .port
      .unwrap_or_else(|| DatabaseConfig::default().port.expect("missing port"))
  }
  pub fn database(&self) -> String {
    self.database.to_owned().unwrap_or_else(|| {
      DatabaseConfig::default()
        .database
        .expect("missing database")
    })
  }
  pub fn pool_size(&self) -> u32 {
    self.pool_size.unwrap_or_else(|| {
      DatabaseConfig::default()
        .pool_size
        .expect("missing pool_size")
    })
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
pub struct FederationConfig {
  pub enabled: bool,
  pub allowed_instances: Option<Vec<String>>,
  pub blocked_instances: Option<Vec<String>>,
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

#[derive(Debug, Deserialize, Clone)]
pub struct SetupConfig {
  pub admin_username: String,
  pub admin_password: String,
  pub admin_email: Option<String>,
  pub site_name: String,
}
