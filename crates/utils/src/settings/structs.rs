use serde::Deserialize;
use std::net::{IpAddr, Ipv4Addr};

#[derive(Debug, Deserialize, Clone, SmartDefault)]
#[serde(default)]
pub struct Settings {
  #[serde(default)]
  pub database: DatabaseConfig,
  #[default(Some(RateLimitConfig::default()))]
  pub rate_limit: Option<RateLimitConfig>,
  #[default(FederationConfig::default())]
  pub federation: FederationConfig,
  #[default(CaptchaConfig::default())]
  pub captcha: CaptchaConfig,
  #[default(None)]
  pub email: Option<EmailConfig>,
  #[default(None)]
  pub setup: Option<SetupConfig>,
  #[default("unset")]
  pub hostname: String,
  #[default(IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)))]
  pub bind: IpAddr,
  #[default(8536)]
  pub port: u16,
  #[default(true)]
  pub tls_enabled: bool,
  #[default("changeme")]
  pub jwt_secret: String,
  #[default(None)]
  pub pictrs_url: Option<String>,
  #[default(None)]
  pub additional_slurs: Option<String>,
  #[default(20)]
  pub actor_name_max_length: usize,
}

#[derive(Debug, Deserialize, Clone, SmartDefault)]
#[serde(default)]
pub struct CaptchaConfig {
  #[default(false)]
  pub enabled: bool,
  #[default("medium")]
  pub difficulty: String,
}

#[derive(Debug, Deserialize, Clone, SmartDefault)]
#[serde(default)]
pub struct DatabaseConfig {
  #[default("lemmy")]
  pub(super) user: String,
  #[default("password")]
  pub password: String,
  #[default("localhost")]
  pub host: String,
  #[default(5432)]
  pub(super) port: i32,
  #[default("lemmy")]
  pub(super) database: String,
  #[default(5)]
  pub pool_size: u32,
}

#[derive(Debug, Deserialize, Clone)]
pub struct EmailConfig {
  pub smtp_server: String,
  pub smtp_login: Option<String>,
  pub smtp_password: Option<String>,
  pub smtp_from_address: String,
  pub use_tls: bool,
}

#[derive(Debug, Deserialize, Clone, SmartDefault)]
#[serde(default)]
pub struct FederationConfig {
  #[default(false)]
  pub enabled: bool,
  #[default(None)]
  pub allowed_instances: Option<Vec<String>>,
  #[default(None)]
  pub blocked_instances: Option<Vec<String>>,
  #[default(true)]
  pub strict_allowlist: bool,
}

#[derive(Debug, Deserialize, Clone, SmartDefault)]
#[serde(default)]
pub struct RateLimitConfig {
  #[default(180)]
  pub message: i32,
  #[default(60)]
  pub message_per_second: i32,
  #[default(6)]
  pub post: i32,
  #[default(600)]
  pub post_per_second: i32,
  #[default(3)]
  pub register: i32,
  #[default(3600)]
  pub register_per_second: i32,
  #[default(6)]
  pub image: i32,
  #[default(3600)]
  pub image_per_second: i32,
}

#[derive(Debug, Deserialize, Clone, SmartDefault)]
pub struct SetupConfig {
  pub admin_username: String,
  pub admin_password: String,
  pub site_name: String,
  #[default(None)]
  pub admin_email: Option<String>,
  #[default(None)]
  pub sidebar: Option<String>,
  #[default(None)]
  pub description: Option<String>,
  #[default(None)]
  pub icon: Option<String>,
  #[default(None)]
  pub banner: Option<String>,
  #[default(None)]
  pub enable_downvotes: Option<bool>,
  #[default(None)]
  pub open_registration: Option<bool>,
  #[default(None)]
  pub enable_nsfw: Option<bool>,
  #[default(None)]
  pub community_creation_admin_only: Option<bool>,
}
