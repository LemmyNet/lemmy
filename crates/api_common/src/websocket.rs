use lemmy_db_schema::{CommunityId, PostId};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Debug)]
pub struct UserJoin {
  pub auth: String,
}

#[derive(Serialize, Clone)]
pub struct UserJoinResponse {
  pub joined: bool,
}

#[derive(Deserialize, Debug)]
pub struct CommunityJoin {
  pub community_id: CommunityId,
}

#[derive(Serialize, Clone)]
pub struct CommunityJoinResponse {
  pub joined: bool,
}

#[derive(Deserialize, Debug)]
pub struct ModJoin {
  pub community_id: CommunityId,
}

#[derive(Serialize, Clone)]
pub struct ModJoinResponse {
  pub joined: bool,
}

#[derive(Deserialize, Debug)]
pub struct PostJoin {
  pub post_id: PostId,
}

#[derive(Serialize, Clone)]
pub struct PostJoinResponse {
  pub joined: bool,
}
