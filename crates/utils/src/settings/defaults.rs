use crate::settings::{CaptchaConfig, DatabaseConfig, FederationConfig, RateLimitConfig, Settings};
use std::net::{IpAddr, Ipv4Addr};

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

pub(in crate::settings) static DEFAULT_DATABASE_USER: &str = "lemmy";
pub(in crate::settings) static DEFAULT_DATABASE_PORT: i32 = 5432;
pub(in crate::settings) static DEFAULT_DATABASE_DB: &str = "lemmy";
pub static DEFAULT_DATABASE_POOL_SIZE: u32 = 5;

impl Default for DatabaseConfig {
  fn default() -> Self {
    Self {
      user: Some(DEFAULT_DATABASE_USER.to_string()),
      password: "password".into(),
      host: "localhost".into(),
      port: Some(DEFAULT_DATABASE_PORT),
      database: Some(DEFAULT_DATABASE_DB.to_string()),
      pool_size: Some(DEFAULT_DATABASE_POOL_SIZE),
    }
  }
}

impl Default for CaptchaConfig {
  fn default() -> Self {
    Self {
      enabled: true,
      difficulty: "medium".into(),
    }
  }
}

impl Default for FederationConfig {
  fn default() -> Self {
    Self {
      enabled: false,
      allowed_instances: None,
      blocked_instances: None,
    }
  }
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
