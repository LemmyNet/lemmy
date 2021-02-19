use merge::Merge;
use serde::Deserialize;
use std::net::IpAddr;

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

#[derive(Debug, Deserialize, Clone)]
pub struct DatabaseConfig {
  pub user: String,
  pub password: String,
  pub host: String,
  pub port: i32,
  pub database: String,
  pub pool_size: u32,
}

#[derive(Debug, Deserialize, Clone)]
pub struct FederationConfig {
  pub enabled: bool,
  pub allowed_instances: Option<String>,
  pub blocked_instances: Option<String>,
}
