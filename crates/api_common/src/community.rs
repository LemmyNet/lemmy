use lemmy_db_schema::{
  newtypes::{CommunityId, LanguageId, PersonId},
  source::site::Site,
  ListingType,
  SortType,
};
use lemmy_db_views_actor::structs::{CommunityModeratorView, CommunityView, PersonView};
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
#[cfg(feature = "full")]
use ts_rs::TS;

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Clone, Default)]
#[cfg_attr(feature = "full", derive(TS))]
#[cfg_attr(feature = "full", ts(export))]
pub struct GetCommunity {
  pub id: Option<CommunityId>,
  /// Example: star_trek , or star_trek@xyz.tld
  pub name: Option<String>,
}

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "full", derive(TS))]
#[cfg_attr(feature = "full", ts(export))]
pub struct GetCommunityResponse {
  pub community_view: CommunityView,
  pub site: Option<Site>,
  pub moderators: Vec<CommunityModeratorView>,
  pub online: usize,
  pub discussion_languages: Vec<LanguageId>,
  /// Default language used for new posts if none is specified, generated based on community and
  /// user languages.
  pub default_post_language: Option<LanguageId>,
}

#[skip_serializing_none]
#[cfg_attr(feature = "full", derive(TS))]
#[cfg_attr(feature = "full", ts(export))]
#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct CreateCommunity {
  pub name: String,
  pub title: String,
  pub description: Option<String>,
  pub icon: Option<String>,
  pub banner: Option<String>,
  pub nsfw: Option<bool>,
  pub posting_restricted_to_mods: Option<bool>,
  pub discussion_languages: Option<Vec<LanguageId>>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "full", derive(TS))]
#[cfg_attr(feature = "full", ts(export))]
pub struct CommunityResponse {
  pub community_view: CommunityView,
  pub discussion_languages: Vec<LanguageId>,
}

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Clone, Default)]
#[cfg_attr(feature = "full", derive(TS))]
#[cfg_attr(feature = "full", ts(export))]
pub struct ListCommunities {
  pub type_: Option<ListingType>,
  pub sort: Option<SortType>,
  pub page: Option<i64>,
  pub limit: Option<i64>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "full", derive(TS))]
#[cfg_attr(feature = "full", ts(export))]
pub struct ListCommunitiesResponse {
  pub communities: Vec<CommunityView>,
}

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Clone, Default)]
#[cfg_attr(feature = "full", derive(TS))]
#[cfg_attr(feature = "full", ts(export))]
pub struct BanFromCommunity {
  pub community_id: CommunityId,
  pub person_id: PersonId,
  pub ban: bool,
  pub remove_data: Option<bool>,
  pub reason: Option<String>,
  pub expires: Option<i64>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "full", derive(TS))]
#[cfg_attr(feature = "full", ts(export))]
pub struct BanFromCommunityResponse {
  pub person_view: PersonView,
  pub banned: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
#[cfg_attr(feature = "full", derive(TS))]
#[cfg_attr(feature = "full", ts(export))]
pub struct AddModToCommunity {
  pub community_id: CommunityId,
  pub person_id: PersonId,
  pub added: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "full", derive(TS))]
#[cfg_attr(feature = "full", ts(export))]
pub struct AddModToCommunityResponse {
  pub moderators: Vec<CommunityModeratorView>,
}

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Clone, Default)]
#[cfg_attr(feature = "full", derive(TS))]
#[cfg_attr(feature = "full", ts(export))]
pub struct EditCommunity {
  pub community_id: CommunityId,
  pub title: Option<String>,
  pub description: Option<String>,
  pub icon: Option<String>,
  pub banner: Option<String>,
  pub nsfw: Option<bool>,
  pub posting_restricted_to_mods: Option<bool>,
  pub discussion_languages: Option<Vec<LanguageId>>,
}

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Clone, Default)]
#[cfg_attr(feature = "full", derive(TS))]
#[cfg_attr(feature = "full", ts(export))]
pub struct HideCommunity {
  pub community_id: CommunityId,
  pub hidden: bool,
  pub reason: Option<String>,
}

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Clone, Default)]
#[cfg_attr(feature = "full", derive(TS))]
#[cfg_attr(feature = "full", ts(export))]
pub struct DeleteCommunity {
  pub community_id: CommunityId,
  pub deleted: bool,
}

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Clone, Default)]
#[cfg_attr(feature = "full", derive(TS))]
#[cfg_attr(feature = "full", ts(export))]
pub struct RemoveCommunity {
  pub community_id: CommunityId,
  pub removed: bool,
  pub reason: Option<String>,
  pub expires: Option<i64>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
#[cfg_attr(feature = "full", derive(TS))]
#[cfg_attr(feature = "full", ts(export))]
pub struct FollowCommunity {
  pub community_id: CommunityId,
  pub follow: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
#[cfg_attr(feature = "full", derive(TS))]
#[cfg_attr(feature = "full", ts(export))]
pub struct BlockCommunity {
  pub community_id: CommunityId,
  pub block: bool,
}

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "full", derive(TS))]
#[cfg_attr(feature = "full", ts(export))]
pub struct BlockCommunityResponse {
  pub community_view: CommunityView,
  pub blocked: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
#[cfg_attr(feature = "full", derive(TS))]
#[cfg_attr(feature = "full", ts(export))]
pub struct TransferCommunity {
  pub community_id: CommunityId,
  pub person_id: PersonId,
}
