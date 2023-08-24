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
use lemmy_proc_macros::lemmy_dto;

#[lemmy_dto(default, skip_none)]
/// Searches the site, given a query string, and some optional filters.
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

#[lemmy_dto]
/// The search response, containing lists of the return type possibilities
// TODO this should be redone as a list of tagged enums
pub struct SearchResponse {
  pub type_: SearchType,
  pub comments: Vec<CommentView>,
  pub posts: Vec<PostView>,
  pub communities: Vec<CommunityView>,
  pub users: Vec<PersonView>,
}

#[lemmy_dto(default)]
/// Does an apub fetch for an object.
pub struct ResolveObject {
  /// Can be the full url, or a shortened version like: !fediverse@lemmy.ml
  pub q: String,
  pub auth: Option<Sensitive<String>>,
}

#[lemmy_dto(default, skip_none)]
// TODO Change this to an enum
/// The response of an apub object fetch.
pub struct ResolveObjectResponse {
  pub comment: Option<CommentView>,
  pub post: Option<PostView>,
  pub community: Option<CommunityView>,
  pub person: Option<PersonView>,
}

#[lemmy_dto(default, skip_none)]
/// Fetches the modlog.
pub struct GetModlog {
  pub mod_person_id: Option<PersonId>,
  pub community_id: Option<CommunityId>,
  pub page: Option<i64>,
  pub limit: Option<i64>,
  pub type_: Option<ModlogActionType>,
  pub other_person_id: Option<PersonId>,
  pub auth: Option<Sensitive<String>>,
}

#[lemmy_dto]
/// The modlog fetch response.
// TODO this should be redone as a list of tagged enums
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

#[lemmy_dto(default, skip_none)]
/// Creates a site. Should be done after first running lemmy.
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
  pub captcha_enabled: Option<bool>,
  pub captcha_difficulty: Option<String>,
  pub allowed_instances: Option<Vec<String>>,
  pub blocked_instances: Option<Vec<String>>,
  pub taglines: Option<Vec<String>>,
  pub registration_mode: Option<RegistrationMode>,
  pub auth: Sensitive<String>,
}

#[lemmy_dto(default, skip_none)]
/// Edits a site.
pub struct EditSite {
  pub name: Option<String>,
  pub sidebar: Option<String>,
  /// A shorter, one line description of your site.
  pub description: Option<String>,
  /// A url for your site's icon.
  pub icon: Option<String>,
  /// A url for your site's banner.
  pub banner: Option<String>,
  /// Whether to enable downvotes.
  pub enable_downvotes: Option<bool>,
  /// Whether to enable NSFW.
  pub enable_nsfw: Option<bool>,
  /// Limits community creation to admins only.
  pub community_creation_admin_only: Option<bool>,
  /// Whether to require email verification.
  pub require_email_verification: Option<bool>,
  /// Your application question form. This is in markdown, and can be many questions.
  pub application_question: Option<String>,
  /// Whether your instance is public, or private.
  pub private_instance: Option<bool>,
  /// The default theme. Usually "browser"
  pub default_theme: Option<String>,
  pub default_post_listing_type: Option<ListingType>,
  /// An optional page of legal information
  pub legal_information: Option<String>,
  /// Whether to email admins when receiving a new application.
  pub application_email_admins: Option<bool>,
  /// Whether to hide moderator names from the modlog.
  pub hide_modlog_mod_names: Option<bool>,
  /// A list of allowed discussion languages.
  pub discussion_languages: Option<Vec<LanguageId>>,
  /// A regex string of items to filter.
  pub slur_filter_regex: Option<String>,
  /// The max length of actor names.
  pub actor_name_max_length: Option<i32>,
  /// The number of messages allowed in a given time frame.
  pub rate_limit_message: Option<i32>,
  pub rate_limit_message_per_second: Option<i32>,
  /// The number of posts allowed in a given time frame.
  pub rate_limit_post: Option<i32>,
  pub rate_limit_post_per_second: Option<i32>,
  /// The number of registrations allowed in a given time frame.
  pub rate_limit_register: Option<i32>,
  pub rate_limit_register_per_second: Option<i32>,
  /// The number of image uploads allowed in a given time frame.
  pub rate_limit_image: Option<i32>,
  pub rate_limit_image_per_second: Option<i32>,
  /// The number of comments allowed in a given time frame.
  pub rate_limit_comment: Option<i32>,
  pub rate_limit_comment_per_second: Option<i32>,
  /// The number of searches allowed in a given time frame.
  pub rate_limit_search: Option<i32>,
  pub rate_limit_search_per_second: Option<i32>,
  /// Whether to enable federation.
  pub federation_enabled: Option<bool>,
  /// Enables federation debugging.
  pub federation_debug: Option<bool>,
  /// Whether to enable captchas for signups.
  pub captcha_enabled: Option<bool>,
  /// The captcha difficulty. Can be easy, medium, or hard
  pub captcha_difficulty: Option<String>,
  /// A list of allowed instances. If none are set, federation is open.
  pub allowed_instances: Option<Vec<String>>,
  /// A list of blocked instances.
  pub blocked_instances: Option<Vec<String>>,
  /// A list of taglines shown at the top of the front page.
  pub taglines: Option<Vec<String>>,
  pub registration_mode: Option<RegistrationMode>,
  /// Whether to email admins for new reports.
  pub reports_email_admins: Option<bool>,
  pub auth: Sensitive<String>,
}

#[lemmy_dto(default, skip_none)]
/// Fetches the site.
pub struct GetSite {
  pub auth: Option<Sensitive<String>>,
}

#[lemmy_dto]
/// The response for a site.
pub struct SiteResponse {
  pub site_view: SiteView,
  pub taglines: Vec<Tagline>,
}

#[lemmy_dto(skip_none)]
/// An expanded response for a site.
pub struct GetSiteResponse {
  pub site_view: SiteView,
  pub admins: Vec<PersonView>,
  pub version: String,
  pub my_user: Option<MyUserInfo>,
  pub all_languages: Vec<Language>,
  pub discussion_languages: Vec<LanguageId>,
  /// A list of taglines shown at the top of the front page.
  pub taglines: Vec<Tagline>,
  /// A list of custom emojis your site supports.
  pub custom_emojis: Vec<CustomEmojiView>,
}

#[lemmy_dto(skip_none)]
/// Fetches the federated instances for your site.
pub struct GetFederatedInstances {
  pub auth: Option<Sensitive<String>>,
}

#[lemmy_dto(skip_none)]
/// A response of federated instances.
pub struct GetFederatedInstancesResponse {
  /// Optional, because federation may be disabled.
  pub federated_instances: Option<FederatedInstances>,
}

#[lemmy_dto]
/// Your user info.
pub struct MyUserInfo {
  pub local_user_view: LocalUserView,
  pub follows: Vec<CommunityFollowerView>,
  pub moderates: Vec<CommunityModeratorView>,
  pub community_blocks: Vec<CommunityBlockView>,
  pub person_blocks: Vec<PersonBlockView>,
  pub discussion_languages: Vec<LanguageId>,
}

#[lemmy_dto]
/// Leaves the admin team.
pub struct LeaveAdmin {
  pub auth: Sensitive<String>,
}

#[lemmy_dto]
/// A list of federated instances.
pub struct FederatedInstances {
  pub linked: Vec<Instance>,
  pub allowed: Vec<Instance>,
  pub blocked: Vec<Instance>,
}

#[lemmy_dto(skip_none)]
/// Purges a person from the database. This will delete all content attached to that person.
pub struct PurgePerson {
  pub person_id: PersonId,
  pub reason: Option<String>,
  pub auth: Sensitive<String>,
}

#[lemmy_dto(skip_none)]
/// Purges a community from the database. This will delete all content attached to that community.
pub struct PurgeCommunity {
  pub community_id: CommunityId,
  pub reason: Option<String>,
  pub auth: Sensitive<String>,
}

#[lemmy_dto(skip_none)]
/// Purges a post from the database. This will delete all content attached to that post.
pub struct PurgePost {
  pub post_id: PostId,
  pub reason: Option<String>,
  pub auth: Sensitive<String>,
}

#[lemmy_dto(skip_none)]
/// Purges a comment from the database. This will delete all content attached to that comment.
pub struct PurgeComment {
  pub comment_id: CommentId,
  pub reason: Option<String>,
  pub auth: Sensitive<String>,
}

#[lemmy_dto]
/// The response for purged items.
pub struct PurgeItemResponse {
  pub success: bool,
}

#[lemmy_dto(default, skip_none)]
/// Fetches a list of registration applications.
pub struct ListRegistrationApplications {
  /// Only shows the unread applications (IE those without an admin actor)
  pub unread_only: Option<bool>,
  pub page: Option<i64>,
  pub limit: Option<i64>,
  pub auth: Sensitive<String>,
}

#[lemmy_dto]
/// The list of registration applications.
pub struct ListRegistrationApplicationsResponse {
  pub registration_applications: Vec<RegistrationApplicationView>,
}

#[lemmy_dto(default, skip_none)]
/// Approves a registration application.
pub struct ApproveRegistrationApplication {
  pub id: i32,
  pub approve: bool,
  pub deny_reason: Option<String>,
  pub auth: Sensitive<String>,
}

#[lemmy_dto]
/// The response of an action done to a registration application.
pub struct RegistrationApplicationResponse {
  pub registration_application: RegistrationApplicationView,
}

#[lemmy_dto]
/// Gets a count of unread registration applications.
pub struct GetUnreadRegistrationApplicationCount {
  pub auth: Sensitive<String>,
}

#[lemmy_dto]
/// The count of unread registration applications.
pub struct GetUnreadRegistrationApplicationCountResponse {
  pub registration_applications: i64,
}
