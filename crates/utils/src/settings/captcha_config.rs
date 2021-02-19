use serde::Deserialize;

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
