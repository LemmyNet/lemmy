use crate::sensitive::Sensitive;
use lemmy_db_schema::{
  newtypes::{CommentId, CommunityId, PersonId, PostId},
  ListingType,
  SearchType,
  SortType,
};
use lemmy_db_views::structs::{
  CommentView,
  LocalUserSettingsView,
  PostView,
  RegistrationApplicationView,
  SiteView,
};
use lemmy_db_views_actor::structs::{
  CommunityBlockView,
  CommunityFollowerView,
  CommunityModeratorView,
  CommunityView,
  PersonBlockView,
  PersonViewSafe,
};
use lemmy_db_views_moderator::structs::{
  AdminPurgeCommentView,
  AdminPurgeCommunityView,
  AdminPurgePersonView,
  AdminPurgePostView,
  ModAddCommunityView,
  ModAddView,
  ModBanFromCommunityView,
  ModBanView,
  ModHideCommunityView,
  ModLockPostView,
  ModRemoveCommentView,
  ModRemoveCommunityView,
  ModRemovePostView,
  ModStickyPostView,
  ModTransferCommunityView,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct Search {
  pub q: String,
  pub community_id: Option<CommunityId>,
  pub community_name: Option<String>,
  pub creator_id: Option<PersonId>,
  pub type_: Option<SearchType>,
  pub sort: Option<SortType>,
  pub listing_type: Option<ListingType>,
  pub page: Option<i64>,
  pub limit: Option<i64>,
  pub auth: Option<Sensitive<String>>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SearchResponse {
  pub type_: String,
  pub comments: Vec<CommentView>,
  pub posts: Vec<PostView>,
  pub communities: Vec<CommunityView>,
  pub users: Vec<PersonViewSafe>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct ResolveObject {
  pub q: String,
  pub auth: Option<Sensitive<String>>,
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct ResolveObjectResponse {
  pub comment: Option<CommentView>,
  pub post: Option<PostView>,
  pub community: Option<CommunityView>,
  pub person: Option<PersonViewSafe>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct GetModlog {
  pub mod_person_id: Option<PersonId>,
  pub community_id: Option<CommunityId>,
  pub page: Option<i64>,
  pub limit: Option<i64>,
  pub auth: Option<Sensitive<String>>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GetModlogResponse {
  pub removed_posts: Vec<ModRemovePostView>,
  pub locked_posts: Vec<ModLockPostView>,
  pub stickied_posts: Vec<ModStickyPostView>,
  pub removed_comments: Vec<ModRemoveCommentView>,
  pub removed_communities: Vec<ModRemoveCommunityView>,
  pub banned_from_community: Vec<ModBanFromCommunityView>,
  pub banned: Vec<ModBanView>,
  pub added_to_community: Vec<ModAddCommunityView>,
  pub transferred_to_community: Vec<ModTransferCommunityView>,
  pub added: Vec<ModAddView>,
  pub admin_purged_persons: Vec<AdminPurgePersonView>,
  pub admin_purged_communities: Vec<AdminPurgeCommunityView>,
  pub admin_purged_posts: Vec<AdminPurgePostView>,
  pub admin_purged_comments: Vec<AdminPurgeCommentView>,
  pub hidden_communities: Vec<ModHideCommunityView>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct CreateSite {
  pub name: String,
  pub sidebar: Option<String>,
  pub description: Option<String>,
  pub icon: Option<String>,
  pub banner: Option<String>,
  pub enable_downvotes: Option<bool>,
  pub open_registration: Option<bool>,
  pub enable_nsfw: Option<bool>,
  pub community_creation_admin_only: Option<bool>,
  pub require_email_verification: Option<bool>,
  pub require_application: Option<bool>,
  pub application_question: Option<String>,
  pub private_instance: Option<bool>,
  pub default_theme: Option<String>,
  pub default_post_listing_type: Option<String>,
  pub auth: Sensitive<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct EditSite {
  pub name: Option<String>,
  pub sidebar: Option<String>,
  pub description: Option<String>,
  pub icon: Option<String>,
  pub banner: Option<String>,
  pub enable_downvotes: Option<bool>,
  pub open_registration: Option<bool>,
  pub enable_nsfw: Option<bool>,
  pub community_creation_admin_only: Option<bool>,
  pub require_email_verification: Option<bool>,
  pub require_application: Option<bool>,
  pub application_question: Option<String>,
  pub private_instance: Option<bool>,
  pub default_theme: Option<String>,
  pub default_post_listing_type: Option<String>,
  pub legal_information: Option<String>,
  pub auth: Sensitive<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct GetSite {
  pub auth: Option<Sensitive<String>>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SiteResponse {
  pub site_view: SiteView,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GetSiteResponse {
  pub site_view: Option<SiteView>, // Because the site might not be set up yet
  pub admins: Vec<PersonViewSafe>,
  pub online: usize,
  pub version: String,
  pub my_user: Option<MyUserInfo>,
  pub federated_instances: Option<FederatedInstances>, // Federation may be disabled
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MyUserInfo {
  pub local_user_view: LocalUserSettingsView,
  pub follows: Vec<CommunityFollowerView>,
  pub moderates: Vec<CommunityModeratorView>,
  pub community_blocks: Vec<CommunityBlockView>,
  pub person_blocks: Vec<PersonBlockView>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct LeaveAdmin {
  pub auth: Sensitive<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FederatedInstances {
  pub linked: Vec<String>,
  pub allowed: Option<Vec<String>>,
  pub blocked: Option<Vec<String>>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PurgePerson {
  pub person_id: PersonId,
  pub reason: Option<String>,
  pub auth: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PurgeCommunity {
  pub community_id: CommunityId,
  pub reason: Option<String>,
  pub auth: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PurgePost {
  pub post_id: PostId,
  pub reason: Option<String>,
  pub auth: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PurgeComment {
  pub comment_id: CommentId,
  pub reason: Option<String>,
  pub auth: String,
}

#[derive(Serialize, Deserialize)]
pub struct PurgeItemResponse {
  pub success: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct ListRegistrationApplications {
  /// Only shows the unread applications (IE those without an admin actor)
  pub unread_only: Option<bool>,
  pub page: Option<i64>,
  pub limit: Option<i64>,
  pub auth: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ListRegistrationApplicationsResponse {
  pub registration_applications: Vec<RegistrationApplicationView>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct ApproveRegistrationApplication {
  pub id: i32,
  pub approve: bool,
  pub deny_reason: Option<String>,
  pub auth: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RegistrationApplicationResponse {
  pub registration_application: RegistrationApplicationView,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GetUnreadRegistrationApplicationCount {
  pub auth: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GetUnreadRegistrationApplicationCountResponse {
  pub registration_applications: i64,
}
