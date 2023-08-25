use crate::sensitive::Sensitive;
use lemmy_db_schema::{
  newtypes::{CommunityId, LanguageId, PersonId},
  source::site::Site,
  ListingType,
  SortType,
};
use lemmy_db_views_actor::structs::{CommunityModeratorView, CommunityView, PersonView};
use lemmy_proc_macros::lemmy_dto;

#[lemmy_dto(Default)]
/// Get a community. Must provide either an id, or a name.
pub struct GetCommunity {
  pub id: Option<CommunityId>,
  /// Example: star_trek , or star_trek@xyz.tld
  pub name: Option<String>,
  pub auth: Option<Sensitive<String>>,
}

#[lemmy_dto]
/// The community response.
pub struct GetCommunityResponse {
  pub community_view: CommunityView,
  pub site: Option<Site>,
  pub moderators: Vec<CommunityModeratorView>,
  pub discussion_languages: Vec<LanguageId>,
}

#[lemmy_dto(Default)]
/// Create a community.
pub struct CreateCommunity {
  /// The unique name.
  pub name: String,
  /// A longer title.
  pub title: String,
  /// A longer sidebar, or description of your community, in markdown.
  pub description: Option<String>,
  /// An icon URL.
  pub icon: Option<String>,
  /// A banner URL.
  pub banner: Option<String>,
  /// Whether its an NSFW community.
  pub nsfw: Option<bool>,
  /// Whether to restrict posting only to moderators.
  pub posting_restricted_to_mods: Option<bool>,
  pub discussion_languages: Option<Vec<LanguageId>>,
  pub auth: Sensitive<String>,
}

#[lemmy_dto]
/// A simple community response.
pub struct CommunityResponse {
  pub community_view: CommunityView,
  pub discussion_languages: Vec<LanguageId>,
}

#[lemmy_dto(Default)]
/// Fetches a list of communities.
pub struct ListCommunities {
  pub type_: Option<ListingType>,
  pub sort: Option<SortType>,
  pub show_nsfw: Option<bool>,
  pub page: Option<i64>,
  pub limit: Option<i64>,
  pub auth: Option<Sensitive<String>>,
}

#[lemmy_dto]
/// The response for listing communities.
pub struct ListCommunitiesResponse {
  pub communities: Vec<CommunityView>,
}

#[lemmy_dto(Default)]
/// Ban a user from a community.
pub struct BanFromCommunity {
  pub community_id: CommunityId,
  pub person_id: PersonId,
  pub ban: bool,
  pub remove_data: Option<bool>,
  pub reason: Option<String>,
  pub expires: Option<i64>,
  pub auth: Sensitive<String>,
}

#[lemmy_dto]
/// The response for banning a user from a community.
pub struct BanFromCommunityResponse {
  pub person_view: PersonView,
  pub banned: bool,
}

#[lemmy_dto(Default)]
/// Add a moderator to a community.
pub struct AddModToCommunity {
  pub community_id: CommunityId,
  pub person_id: PersonId,
  pub added: bool,
  pub auth: Sensitive<String>,
}

#[lemmy_dto]
/// The response of adding a moderator to a community.
pub struct AddModToCommunityResponse {
  pub moderators: Vec<CommunityModeratorView>,
}

#[lemmy_dto(Default)]
/// Edit a community.
pub struct EditCommunity {
  pub community_id: CommunityId,
  /// A longer title.
  pub title: Option<String>,
  /// A longer sidebar, or description of your community, in markdown.
  pub description: Option<String>,
  /// An icon URL.
  pub icon: Option<String>,
  /// A banner URL.
  pub banner: Option<String>,
  /// Whether its an NSFW community.
  pub nsfw: Option<bool>,
  /// Whether to restrict posting only to moderators.
  pub posting_restricted_to_mods: Option<bool>,
  pub discussion_languages: Option<Vec<LanguageId>>,
  pub auth: Sensitive<String>,
}

#[lemmy_dto(Default)]
/// Hide a community from the main view.
// TODO this should really be a part of edit community. And why does it contain a reason, that should be in the mod tables.
pub struct HideCommunity {
  pub community_id: CommunityId,
  pub hidden: bool,
  pub reason: Option<String>,
  pub auth: Sensitive<String>,
}

#[lemmy_dto(Default)]
/// Delete your own community.
pub struct DeleteCommunity {
  pub community_id: CommunityId,
  pub deleted: bool,
  pub auth: Sensitive<String>,
}

#[lemmy_dto(Default)]
/// Remove a community (only doable by moderators).
pub struct RemoveCommunity {
  pub community_id: CommunityId,
  pub removed: bool,
  pub reason: Option<String>,
  pub expires: Option<i64>,
  pub auth: Sensitive<String>,
}

#[lemmy_dto(Default)]
/// Follow / subscribe to a community.
pub struct FollowCommunity {
  pub community_id: CommunityId,
  pub follow: bool,
  pub auth: Sensitive<String>,
}

#[lemmy_dto(Default)]
/// Block a community.
pub struct BlockCommunity {
  pub community_id: CommunityId,
  pub block: bool,
  pub auth: Sensitive<String>,
}

#[lemmy_dto]
/// The block community response.
pub struct BlockCommunityResponse {
  pub community_view: CommunityView,
  pub blocked: bool,
}

#[lemmy_dto(Default)]
/// Transfer a community to a new owner.
pub struct TransferCommunity {
  pub community_id: CommunityId,
  pub person_id: PersonId,
  pub auth: Sensitive<String>,
}
