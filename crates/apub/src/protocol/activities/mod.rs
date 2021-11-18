use serde::{Deserialize, Serialize};
use strum_macros::ToString;

pub mod community;
pub mod create_or_update;
pub mod deletion;
pub mod following;
pub mod private_message;
pub mod voting;

#[derive(Clone, Debug, ToString, Deserialize, Serialize, PartialEq)]
pub enum CreateOrUpdateType {
  Create,
  Update,
}
