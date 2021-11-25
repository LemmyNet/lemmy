use lemmy_db_schema::newtypes::{CommunityId, PostId};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct UserJoin {
  pub auth: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UserJoinResponse {
  pub joined: bool,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CommunityJoin {
  pub community_id: CommunityId,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CommunityJoinResponse {
  pub joined: bool,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ModJoin {
  pub community_id: CommunityId,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ModJoinResponse {
  pub joined: bool,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct PostJoin {
  pub post_id: PostId,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PostJoinResponse {
  pub joined: bool,
}
