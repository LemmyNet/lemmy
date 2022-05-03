use crate::sensitive::Sensitive;
use lemmy_db_schema::newtypes::{CommunityId, PersonId};
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

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct Search {
  pub q: String,
  pub community_id: Option<CommunityId>,
  pub community_name: Option<String>,
  pub creator_id: Option<PersonId>,
  pub type_: Option<String>,
  pub sort: Option<String>,
  pub listing_type: Option<String>,
  pub page: Option<i64>,
  pub limit: Option<i64>,
  pub auth: Option<Sensitive<String>>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SearchResponse {
  pub type_: String,
  pub comments: Vec<CommentView>,
  pub posts: Vec<PostView>,
  pub communities: Vec<CommunityView>,
  pub users: Vec<PersonViewSafe>,
}

#[derive(Serialize, Deserialize, Debug)]
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

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct GetModlog {
  pub mod_person_id: Option<PersonId>,
  pub community_id: Option<CommunityId>,
  pub page: Option<i64>,
  pub limit: Option<i64>,
  pub auth: Option<Sensitive<String>>,
}

#[derive(Debug, Serialize, Deserialize)]
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
  pub hidden_communities: Vec<ModHideCommunityView>,
}

#[derive(Debug, Serialize, Deserialize)]
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

#[derive(Debug, Serialize, Deserialize, Default)]
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
  pub auth: Sensitive<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GetSite {
  pub auth: Option<Sensitive<String>>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SiteResponse {
  pub site_view: SiteView,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GetSiteResponse {
  pub site_view: Option<SiteView>, // Because the site might not be set up yet
  pub admins: Vec<PersonViewSafe>,
  pub online: usize,
  pub version: String,
  pub my_user: Option<MyUserInfo>,
  pub federated_instances: Option<FederatedInstances>, // Federation may be disabled
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MyUserInfo {
  pub local_user_view: LocalUserSettingsView,
  pub follows: Vec<CommunityFollowerView>,
  pub moderates: Vec<CommunityModeratorView>,
  pub community_blocks: Vec<CommunityBlockView>,
  pub person_blocks: Vec<PersonBlockView>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LeaveAdmin {
  pub auth: Sensitive<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GetSiteConfig {
  pub auth: Sensitive<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GetSiteConfigResponse {
  pub config_hjson: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SaveSiteConfig {
  pub config_hjson: String,
  pub auth: Sensitive<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FederatedInstances {
  pub linked: Vec<String>,
  pub allowed: Option<Vec<String>>,
  pub blocked: Option<Vec<String>>,
}

#[derive(Serialize, Deserialize)]
pub struct ListRegistrationApplications {
  /// Only shows the unread applications (IE those without an admin actor)
  pub unread_only: Option<bool>,
  pub page: Option<i64>,
  pub limit: Option<i64>,
  pub auth: String,
}

#[derive(Serialize, Deserialize)]
pub struct ListRegistrationApplicationsResponse {
  pub registration_applications: Vec<RegistrationApplicationView>,
}

#[derive(Serialize, Deserialize)]
pub struct ApproveRegistrationApplication {
  pub id: i32,
  pub approve: bool,
  pub deny_reason: Option<String>,
  pub auth: String,
}

#[derive(Serialize, Deserialize)]
pub struct RegistrationApplicationResponse {
  pub registration_application: RegistrationApplicationView,
}

#[derive(Serialize, Deserialize)]
pub struct GetUnreadRegistrationApplicationCount {
  pub auth: String,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct GetUnreadRegistrationApplicationCountResponse {
  pub registration_applications: i64,
}
