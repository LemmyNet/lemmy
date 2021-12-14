use serde::{Deserialize, Serialize};
use strum_macros::Display;

pub mod community;
pub mod create_or_update;
pub mod deletion;
pub mod following;
pub mod private_message;
pub mod voting;

#[derive(Clone, Debug, Display, Deserialize, Serialize, PartialEq)]
pub enum CreateOrUpdateType {
  Create,
  Update,
}
