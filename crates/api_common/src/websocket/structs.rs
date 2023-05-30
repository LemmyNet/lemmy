use crate::sensitive::Sensitive;
use lemmy_db_schema::newtypes::{CommunityId, PostId};
use serde::{Deserialize, Serialize};
#[cfg(feature = "full")]
use ts_rs::TS;

#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "full", derive(TS))]
#[cfg_attr(feature = "full", ts(export))]
/// Join a user room.
pub struct UserJoin {
  pub auth: Sensitive<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "full", derive(TS))]
#[cfg_attr(feature = "full", ts(export))]
/// The join response.
pub struct UserJoinResponse {
  pub joined: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "full", derive(TS))]
#[cfg_attr(feature = "full", ts(export))]
/// Join a community room.
pub struct CommunityJoin {
  pub community_id: CommunityId,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "full", derive(TS))]
#[cfg_attr(feature = "full", ts(export))]
/// The join response.
pub struct CommunityJoinResponse {
  pub joined: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "full", derive(TS))]
#[cfg_attr(feature = "full", ts(export))]
/// Join a mod room.
pub struct ModJoin {
  pub community_id: CommunityId,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "full", derive(TS))]
#[cfg_attr(feature = "full", ts(export))]
/// The join response.
pub struct ModJoinResponse {
  pub joined: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "full", derive(TS))]
#[cfg_attr(feature = "full", ts(export))]
/// Join a post room.
pub struct PostJoin {
  pub post_id: PostId,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "full", derive(TS))]
#[cfg_attr(feature = "full", ts(export))]
/// The join response.
pub struct PostJoinResponse {
  pub joined: bool,
}

#[derive(Debug)]
pub struct CaptchaItem {
  pub uuid: String,
  pub answer: String,
  pub expires: chrono::NaiveDateTime,
}
