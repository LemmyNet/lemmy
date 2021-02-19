use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
pub struct DatabaseConfig {
  pub user: String,
  pub password: String,
  pub host: String,
  pub port: i32,
  pub database: String,
  pub pool_size: u32,
}

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
