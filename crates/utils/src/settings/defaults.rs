use crate::settings::{CaptchaConfig, DatabaseConfig, FederationConfig, RateLimitConfig};

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
