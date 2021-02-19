use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
pub struct SetupConfig {
  pub admin_username: String,
  pub admin_password: String,
  pub admin_email: Option<String>,
  pub site_name: String,
}
