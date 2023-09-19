#[cfg(feature = "full")]
pub mod build_response;
#[cfg(feature = "full")]
pub mod claims;
pub mod comment;
pub mod community;
#[cfg(feature = "full")]
pub mod context;
pub mod custom_emoji;
pub mod person;
pub mod post;
pub mod private_message;
#[cfg(feature = "full")]
pub mod request;
#[cfg(feature = "full")]
pub mod send_activity;
pub mod sensitive;
pub mod site;
#[cfg(feature = "full")]
pub mod utils;

pub extern crate lemmy_db_schema;
pub extern crate lemmy_db_views;
pub extern crate lemmy_db_views_actor;
pub extern crate lemmy_db_views_moderator;

use serde::{Deserialize, Serialize};
use ts_rs::TS;

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
#[cfg_attr(feature = "full", derive(TS))]
#[cfg_attr(feature = "full", ts(export))]
/// Saves settings for your user.
pub struct SuccessResponse {
  pub success: bool,
}

impl SuccessResponse {
  pub fn new() -> Self {
    SuccessResponse { success: true }
  }
}
