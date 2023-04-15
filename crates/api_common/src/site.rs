use crate::sensitive::Sensitive;
use lemmy_db_schema::{
  newtypes::{CommentId, CommunityId, LanguageId, PersonId, PostId},
  source::{instance::Instance, language::Language, tagline::Tagline},
  ListingType,
  ModlogActionType,
  RegistrationMode,
  SearchType,
  SortType,
};
use lemmy_db_views::structs::{
  CommentView,
  CustomEmojiView,
  LocalUserView,
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
  PersonView,
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
  ModFeaturePostView,
  ModHideCommunityView,
  ModLockPostView,
  ModRemoveCommentView,
  ModRemoveCommunityView,
  ModRemovePostView,
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
  pub users: Vec<PersonView>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct ResolveObject {
  pub q: String,
  pub auth: Sensitive<String>,
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct ResolveObjectResponse {
  pub comment: Option<CommentView>,
  pub post: Option<PostView>,
  pub community: Option<CommunityView>,
  pub person: Option<PersonView>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct GetModlog {
  pub mod_person_id: Option<PersonId>,
  pub community_id: Option<CommunityId>,
  pub page: Option<i64>,
  pub limit: Option<i64>,
  pub auth: Option<Sensitive<String>>,
  pub type_: Option<ModlogActionType>,
  pub other_person_id: Option<PersonId>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GetModlogResponse {
  pub removed_posts: Vec<ModRemovePostView>,
  pub locked_posts: Vec<ModLockPostView>,
  pub featured_posts: Vec<ModFeaturePostView>,
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
  pub enable_nsfw: Option<bool>,
  pub community_creation_admin_only: Option<bool>,
  pub require_email_verification: Option<bool>,
  pub application_question: Option<String>,
  pub private_instance: Option<bool>,
  pub default_theme: Option<String>,
  pub default_post_listing_type: Option<ListingType>,
  pub legal_information: Option<String>,
  pub application_email_admins: Option<bool>,
  pub hide_modlog_mod_names: Option<bool>,
  pub discussion_languages: Option<Vec<LanguageId>>,
  pub slur_filter_regex: Option<String>,
  pub actor_name_max_length: Option<i32>,
  pub rate_limit_message: Option<i32>,
  pub rate_limit_message_per_second: Option<i32>,
  pub rate_limit_post: Option<i32>,
  pub rate_limit_post_per_second: Option<i32>,
  pub rate_limit_register: Option<i32>,
  pub rate_limit_register_per_second: Option<i32>,
  pub rate_limit_image: Option<i32>,
  pub rate_limit_image_per_second: Option<i32>,
  pub rate_limit_comment: Option<i32>,
  pub rate_limit_comment_per_second: Option<i32>,
  pub rate_limit_search: Option<i32>,
  pub rate_limit_search_per_second: Option<i32>,
  pub federation_enabled: Option<bool>,
  pub federation_debug: Option<bool>,
  pub federation_worker_count: Option<i32>,
  pub captcha_enabled: Option<bool>,
  pub captcha_difficulty: Option<String>,
  pub allowed_instances: Option<Vec<String>>,
  pub blocked_instances: Option<Vec<String>>,
  pub taglines: Option<Vec<String>>,
  pub registration_mode: Option<RegistrationMode>,
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
  pub enable_nsfw: Option<bool>,
  pub community_creation_admin_only: Option<bool>,
  pub require_email_verification: Option<bool>,
  pub application_question: Option<String>,
  pub private_instance: Option<bool>,
  pub default_theme: Option<String>,
  pub default_post_listing_type: Option<ListingType>,
  pub legal_information: Option<String>,
  pub application_email_admins: Option<bool>,
  pub hide_modlog_mod_names: Option<bool>,
  pub discussion_languages: Option<Vec<LanguageId>>,
  pub slur_filter_regex: Option<String>,
  pub actor_name_max_length: Option<i32>,
  pub rate_limit_message: Option<i32>,
  pub rate_limit_message_per_second: Option<i32>,
  pub rate_limit_post: Option<i32>,
  pub rate_limit_post_per_second: Option<i32>,
  pub rate_limit_register: Option<i32>,
  pub rate_limit_register_per_second: Option<i32>,
  pub rate_limit_image: Option<i32>,
  pub rate_limit_image_per_second: Option<i32>,
  pub rate_limit_comment: Option<i32>,
  pub rate_limit_comment_per_second: Option<i32>,
  pub rate_limit_search: Option<i32>,
  pub rate_limit_search_per_second: Option<i32>,
  pub federation_enabled: Option<bool>,
  pub federation_debug: Option<bool>,
  pub federation_worker_count: Option<i32>,
  pub captcha_enabled: Option<bool>,
  pub captcha_difficulty: Option<String>,
  pub allowed_instances: Option<Vec<String>>,
  pub blocked_instances: Option<Vec<String>>,
  pub taglines: Option<Vec<String>>,
  pub registration_mode: Option<RegistrationMode>,
  pub reports_email_admins: Option<bool>,
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
  pub site_view: SiteView,
  pub admins: Vec<PersonView>,
  pub online: usize,
  pub version: String,
  pub my_user: Option<MyUserInfo>,
  pub federated_instances: Option<FederatedInstances>, // Federation may be disabled
  pub all_languages: Vec<Language>,
  pub discussion_languages: Vec<LanguageId>,
  pub taglines: Vec<Tagline>,
  pub custom_emojis: Vec<CustomEmojiView>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MyUserInfo {
  pub local_user_view: LocalUserView,
  pub follows: Vec<CommunityFollowerView>,
  pub moderates: Vec<CommunityModeratorView>,
  pub community_blocks: Vec<CommunityBlockView>,
  pub person_blocks: Vec<PersonBlockView>,
  pub discussion_languages: Vec<LanguageId>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct LeaveAdmin {
  pub auth: Sensitive<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FederatedInstances {
  pub linked: Vec<Instance>,
  pub allowed: Option<Vec<Instance>>,
  pub blocked: Option<Vec<Instance>>,
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
