use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone)]
pub struct BlackListCommunityResponse {
  pub blacklist_id: i32,
}

#[derive(Serialize, Deserialize)]
pub struct BlackListCommunity {
  /// Example: star_trek , or star_trek@xyz.tld
  pub community_id: i32,
  pub reason: Option<String>,
  pub auth: String,
}

#[derive(Serialize, Deserialize)]
pub struct DeleteBlackListCommunity {
  pub community_id: i32,
  pub auth: String,
}


