use lemmy_db::{
  community_view::{CommunityFollowerView, CommunityModeratorView, CommunityView},
  user_view::UserView,
};
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct GetCommunity {
  pub id: Option<i32>,
  pub name: Option<String>,
  pub auth: Option<String>,
}

#[derive(Serialize)]
pub struct GetCommunityResponse {
  pub community: CommunityView,
  pub moderators: Vec<CommunityModeratorView>,
  pub online: usize,
}

#[derive(Deserialize)]
pub struct CreateCommunity {
  pub name: String,
  pub title: String,
  pub description: Option<String>,
  pub icon: Option<String>,
  pub banner: Option<String>,
  pub category_id: i32,
  pub nsfw: bool,
  pub auth: String,
}

#[derive(Serialize, Clone)]
pub struct CommunityResponse {
  pub community: CommunityView,
}

#[derive(Deserialize, Debug)]
pub struct ListCommunities {
  pub sort: String,
  pub page: Option<i64>,
  pub limit: Option<i64>,
  pub auth: Option<String>,
}

#[derive(Serialize, Debug)]
pub struct ListCommunitiesResponse {
  pub communities: Vec<CommunityView>,
}

#[derive(Deserialize, Clone)]
pub struct BanFromCommunity {
  pub community_id: i32,
  pub user_id: i32,
  pub ban: bool,
  pub remove_data: Option<bool>,
  pub reason: Option<String>,
  pub expires: Option<i64>,
  pub auth: String,
}

#[derive(Serialize, Clone)]
pub struct BanFromCommunityResponse {
  pub user: UserView,
  pub banned: bool,
}

#[derive(Deserialize)]
pub struct AddModToCommunity {
  pub community_id: i32,
  pub user_id: i32,
  pub added: bool,
  pub auth: String,
}

#[derive(Serialize, Clone)]
pub struct AddModToCommunityResponse {
  pub moderators: Vec<CommunityModeratorView>,
}

#[derive(Deserialize)]
pub struct EditCommunity {
  pub edit_id: i32,
  pub title: String,
  pub description: Option<String>,
  pub icon: Option<String>,
  pub banner: Option<String>,
  pub category_id: i32,
  pub nsfw: bool,
  pub auth: String,
}

#[derive(Deserialize)]
pub struct DeleteCommunity {
  pub edit_id: i32,
  pub deleted: bool,
  pub auth: String,
}

#[derive(Deserialize)]
pub struct RemoveCommunity {
  pub edit_id: i32,
  pub removed: bool,
  pub reason: Option<String>,
  pub expires: Option<i64>,
  pub auth: String,
}

#[derive(Deserialize)]
pub struct FollowCommunity {
  pub community_id: i32,
  pub follow: bool,
  pub auth: String,
}

#[derive(Deserialize)]
pub struct GetFollowedCommunities {
  pub auth: String,
}

#[derive(Serialize)]
pub struct GetFollowedCommunitiesResponse {
  pub communities: Vec<CommunityFollowerView>,
}

#[derive(Deserialize)]
pub struct TransferCommunity {
  pub community_id: i32,
  pub user_id: i32,
  pub auth: String,
}

#[derive(Deserialize, Debug)]
pub struct CommunityJoin {
  pub community_id: i32,
}

#[derive(Serialize, Clone)]
pub struct CommunityJoinResponse {
  pub joined: bool,
}
