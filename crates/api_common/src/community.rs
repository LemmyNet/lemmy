use lemmy_db_schema::newtypes::{CommunityId, PersonId};
use lemmy_db_views_actor::{
  community_moderator_view::CommunityModeratorView,
  community_view::CommunityView,
  person_view::PersonViewSafe,
};
use lemmy_utils::Sensitive;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct GetCommunity {
  pub id: Option<CommunityId>,
  /// Example: star_trek , or star_trek@xyz.tld
  pub name: Option<String>,
  pub auth: Option<Sensitive<String>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GetCommunityResponse {
  pub community_view: CommunityView,
  pub moderators: Vec<CommunityModeratorView>,
  pub online: usize,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateCommunity {
  pub name: String,
  pub title: String,
  pub description: Option<String>,
  pub icon: Option<String>,
  pub banner: Option<String>,
  pub nsfw: Option<bool>,
  pub auth: Sensitive<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CommunityResponse {
  pub community_view: CommunityView,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ListCommunities {
  pub type_: Option<String>,
  pub sort: Option<String>,
  pub page: Option<i64>,
  pub limit: Option<i64>,
  pub auth: Option<Sensitive<String>>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ListCommunitiesResponse {
  pub communities: Vec<CommunityView>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BanFromCommunity {
  pub community_id: CommunityId,
  pub person_id: PersonId,
  pub ban: bool,
  pub remove_data: Option<bool>,
  pub reason: Option<String>,
  pub expires: Option<i64>,
  pub auth: Sensitive<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BanFromCommunityResponse {
  pub person_view: PersonViewSafe,
  pub banned: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AddModToCommunity {
  pub community_id: CommunityId,
  pub person_id: PersonId,
  pub added: bool,
  pub auth: Sensitive<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AddModToCommunityResponse {
  pub moderators: Vec<CommunityModeratorView>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct EditCommunity {
  pub community_id: CommunityId,
  pub title: Option<String>,
  pub description: Option<String>,
  pub icon: Option<String>,
  pub banner: Option<String>,
  pub nsfw: Option<bool>,
  pub auth: Sensitive<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct HideCommunity {
  pub community_id: CommunityId,
  pub hidden: bool,
  pub reason: Option<String>,
  pub auth: Sensitive<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DeleteCommunity {
  pub community_id: CommunityId,
  pub deleted: bool,
  pub auth: Sensitive<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RemoveCommunity {
  pub community_id: CommunityId,
  pub removed: bool,
  pub reason: Option<String>,
  pub expires: Option<i64>,
  pub auth: Sensitive<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FollowCommunity {
  pub community_id: CommunityId,
  pub follow: bool,
  pub auth: Sensitive<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BlockCommunity {
  pub community_id: CommunityId,
  pub block: bool,
  pub auth: Sensitive<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BlockCommunityResponse {
  pub community_view: CommunityView,
  pub blocked: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TransferCommunity {
  pub community_id: CommunityId,
  pub person_id: PersonId,
  pub auth: Sensitive<String>,
}
