#[cfg(feature = "full")]
#[macro_use]
extern crate strum_macros;
#[cfg(feature = "full")]
#[macro_use]
extern crate smart_default;

#[cfg(feature = "full")]
pub mod claims;
#[cfg(feature = "full")]
pub mod email;
pub mod error;
#[cfg(feature = "full")]
pub mod rate_limit;
#[cfg(feature = "full")]
pub mod settings;
#[cfg(test)]
mod test;
#[cfg(feature = "full")]
pub mod utils;
#[cfg(feature = "full")]
pub mod version;

use std::{
  fmt,
  fmt::{Debug, Display},
};

pub type ConnectionId = usize;

#[derive(PartialEq, Eq, Hash, Debug, Clone)]
pub struct IpAddr(pub String);

impl Display for IpAddr {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "{}", self.0)
  }
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
