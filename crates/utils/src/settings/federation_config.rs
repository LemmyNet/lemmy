use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
pub struct FederationConfig {
  pub enabled: bool,
  pub allowed_instances: Option<Vec<String>>,
  pub blocked_instances: Option<Vec<String>>,
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
