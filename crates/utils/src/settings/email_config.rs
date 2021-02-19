use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
pub struct EmailConfig {
  pub smtp_server: String,
  pub smtp_login: Option<String>,
  pub smtp_password: Option<String>,
  pub smtp_from_address: String,
  pub use_tls: bool,
}
