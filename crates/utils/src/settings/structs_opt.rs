use serde::Deserialize;
use std::net::IpAddr;

#[derive(Debug, Deserialize, Clone)]
pub struct SettingsOpt {
  pub setup: Option<SetupOpt>,
  pub database: Option<DatabaseConfigOpt>,
  pub hostname: Option<String>,
  pub bind: Option<IpAddr>,
  pub port: Option<u16>,
  pub tls_enabled: Option<bool>,
  pub jwt_secret: Option<String>,
  pub pictrs_url: Option<String>,
  pub iframely_url: Option<String>,
  pub rate_limit: Option<RateLimitConfigOpt>,
  pub email: Option<EmailConfigOpt>,
  pub federation: Option<FederationConfigOpt>,
  pub captcha: Option<CaptchaConfigOpt>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct SetupOpt {
  pub admin_username: Option<String>,
  pub admin_password: Option<String>,
  pub admin_email: Option<Option<String>>,
  pub site_name: Option<String>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct RateLimitConfigOpt {
  pub message: Option<i32>,
  pub message_per_second: Option<i32>,
  pub post: Option<i32>,
  pub post_per_second: Option<i32>,
  pub register: Option<i32>,
  pub register_per_second: Option<i32>,
  pub image: Option<i32>,
  pub image_per_second: Option<i32>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct EmailConfigOpt {
  pub smtp_server: Option<String>,
  pub smtp_login: Option<Option<String>>,
  pub smtp_password: Option<Option<String>>,
  pub smtp_from_address: Option<String>,
  pub use_tls: Option<bool>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct CaptchaConfigOpt {
  pub enabled: Option<bool>,
  pub difficulty: Option<String>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct DatabaseConfigOpt {
  pub user: Option<String>,
  pub password: Option<String>,
  pub host: Option<String>,
  pub port: Option<i32>,
  pub database: Option<String>,
  pub pool_size: Option<u32>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct FederationConfigOpt {
  pub enabled: Option<bool>,
  pub allowed_instances: Option<String>,
  pub blocked_instances: Option<String>,
}
