use lemmy_db_views_actor::{
  community_follower_view::CommunityFollowerView,
  community_moderator_view::CommunityModeratorView,
  community_view::CommunityView,
  user_view::UserViewSafe,
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
  pub community_view: CommunityView,
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
  pub community_view: CommunityView,
}

#[derive(Deserialize, Debug)]
pub struct ListCommunities {
  pub type_: String,
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
  pub remove_data: bool,
  pub reason: Option<String>,
  pub expires: Option<i64>,
  pub auth: String,
}

#[derive(Serialize, Clone)]
pub struct BanFromCommunityResponse {
  pub user_view: UserViewSafe,
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
  pub community_id: i32,
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
  pub community_id: i32,
  pub deleted: bool,
  pub auth: String,
}

#[derive(Deserialize)]
pub struct RemoveCommunity {
  pub community_id: i32,
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
