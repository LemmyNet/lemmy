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

impl Default for DatabaseConfig {
  fn default() -> Self {
    Self {
      user: Some("lemmy".to_string()),
      password: "password".into(),
      host: "localhost".into(),
      port: Some(5432),
      database: Some("lemmy".to_string()),
      pool_size: Some(5),
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
