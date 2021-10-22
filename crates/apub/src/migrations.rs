use lemmy_apub_lib::values::PublicUrl;
use serde::{Deserialize, Serialize};

/// Migrate value of field `to` from single value to vec.
///
/// v0.14: send as single value, accept both
/// v0.15: send as vec, accept both
/// v0.16: send and accept only vec
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(untagged)]
pub enum PublicUrlMigration {
  Old(PublicUrl),
  New([PublicUrl; 1]),
}

impl PublicUrlMigration {
  pub(crate) fn create() -> PublicUrlMigration {
    PublicUrlMigration::Old(PublicUrl::Public)
  }
}
