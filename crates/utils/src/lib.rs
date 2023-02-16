#[macro_use]
extern crate strum_macros;
#[macro_use]
extern crate smart_default;

pub mod apub;
pub mod email;
pub mod rate_limit;
pub mod settings;

pub mod claims;
pub mod error;
pub mod request;
pub mod utils;
pub mod version;

use serde::{Deserialize, Serialize};
use std::{fmt, time::Duration};
use url::Url;

pub type ConnectionId = usize;

pub const REQWEST_TIMEOUT: Duration = Duration::from_secs(10);

#[derive(PartialEq, Eq, Hash, Debug, Clone)]
pub struct IpAddr(pub String);

impl fmt::Display for IpAddr {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "{}", self.0)
  }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct WebfingerLink {
  pub rel: Option<String>,
  #[serde(rename = "type")]
  pub kind: Option<String>,
  pub href: Option<Url>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct WebfingerResponse {
  pub subject: String,
  pub links: Vec<WebfingerLink>,
}

#[macro_export]
macro_rules! location_info {
  () => {
    format!(
      "None value at {}:{}, column {}",
      file!(),
      line!(),
      column!()
    )
  };
}
