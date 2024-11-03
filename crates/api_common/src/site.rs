use crate::federate_retry_sleep_duration;
use chrono::{DateTime, Utc};
use lemmy_db_schema::{
  newtypes::{
    CommentId,
    CommunityId,
    InstanceId,
    LanguageId,
    PersonId,
    PostId,
    RegistrationApplicationId,
  },
  source::{
    community::Community,
    federation_queue_state::FederationQueueState,
    instance::Instance,
    language::Language,
    local_site_url_blocklist::LocalSiteUrlBlocklist,
    oauth_provider::{OAuthProvider, PublicOAuthProvider},
    person::Person,
    tagline::Tagline,
  },
  CommentSortType,
  FederationMode,
  ListingType,
  ModlogActionType,
  PostListingMode,
  PostSortType,
  RegistrationMode,
  SearchType,
};
use lemmy_db_views::structs::{
  CommentView,
  LocalUserView,
  PostView,
  RegistrationApplicationView,
  SiteView,
};
use lemmy_db_views_actor::structs::{
  CommunityFollowerView,
  CommunityModeratorView,
  CommunityView,
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
use serde_with::skip_serializing_none;
#[cfg(feature = "full")]
use ts_rs::TS;

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "full", derive(TS))]
#[cfg_attr(feature = "full", ts(export))]
/// Searches the site, given a query string, and some optional filters.
pub struct Search {
  pub q: String,
  #[ts(optional)]
  pub community_id: Option<CommunityId>,
  #[ts(optional)]
  pub community_name: Option<String>,
  #[ts(optional)]
  pub creator_id: Option<PersonId>,
  #[ts(optional)]
  pub type_: Option<SearchType>,
  #[ts(optional)]
  pub sort: Option<PostSortType>,
  #[ts(optional)]
  pub listing_type: Option<ListingType>,
  #[ts(optional)]
  pub page: Option<i64>,
  #[ts(optional)]
  pub limit: Option<i64>,
  #[ts(optional)]
  pub title_only: Option<bool>,
  #[ts(optional)]
  pub post_url_only: Option<bool>,
  #[ts(optional)]
  pub saved_only: Option<bool>,
  #[ts(optional)]
  pub liked_only: Option<bool>,
  #[ts(optional)]
  pub disliked_only: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "full", derive(TS))]
#[cfg_attr(feature = "full", ts(export))]
/// The search response, containing lists of the return type possibilities
// TODO this should be redone as a list of tagged enums
pub struct SearchResponse {
  pub type_: SearchType,
  pub comments: Vec<CommentView>,
  pub posts: Vec<PostView>,
  pub communities: Vec<CommunityView>,
  pub users: Vec<PersonView>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "full", derive(TS))]
#[cfg_attr(feature = "full", ts(export))]
/// Does an apub fetch for an object.
pub struct ResolveObject {
  /// Can be the full url, or a shortened version like: !fediverse@lemmy.ml
  pub q: String,
}

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Default)]
#[cfg_attr(feature = "full", derive(TS))]
#[cfg_attr(feature = "full", ts(export))]
// TODO Change this to an enum
/// The response of an apub object fetch.
pub struct ResolveObjectResponse {
  #[ts(optional)]
  pub comment: Option<CommentView>,
  #[ts(optional)]
  pub post: Option<PostView>,
  #[ts(optional)]
  pub community: Option<CommunityView>,
  #[ts(optional)]
  pub person: Option<PersonView>,
}

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Clone, Copy, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "full", derive(TS))]
#[cfg_attr(feature = "full", ts(export))]
/// Fetches the modlog.
pub struct GetModlog {
  #[ts(optional)]
  pub mod_person_id: Option<PersonId>,
  #[ts(optional)]
  pub community_id: Option<CommunityId>,
  #[ts(optional)]
  pub page: Option<i64>,
  #[ts(optional)]
  pub limit: Option<i64>,
  #[ts(optional)]
  pub type_: Option<ModlogActionType>,
  #[ts(optional)]
  pub other_person_id: Option<PersonId>,
  #[ts(optional)]
  pub post_id: Option<PostId>,
  #[ts(optional)]
  pub comment_id: Option<CommentId>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "full", derive(TS))]
#[cfg_attr(feature = "full", ts(export))]
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

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "full", derive(TS))]
#[cfg_attr(feature = "full", ts(export))]
/// Creates a site. Should be done after first running lemmy.
pub struct CreateSite {
  pub name: String,
  #[ts(optional)]
  pub sidebar: Option<String>,
  #[ts(optional)]
  pub description: Option<String>,
  #[ts(optional)]
  pub icon: Option<String>,
  #[ts(optional)]
  pub banner: Option<String>,
  #[ts(optional)]
  pub enable_nsfw: Option<bool>,
  #[ts(optional)]
  pub community_creation_admin_only: Option<bool>,
  #[ts(optional)]
  pub require_email_verification: Option<bool>,
  #[ts(optional)]
  pub application_question: Option<String>,
  #[ts(optional)]
  pub private_instance: Option<bool>,
  #[ts(optional)]
  pub default_theme: Option<String>,
  #[ts(optional)]
  pub default_post_listing_type: Option<ListingType>,
  #[ts(optional)]
  pub default_post_listing_mode: Option<PostListingMode>,
  #[ts(optional)]
  pub default_post_sort_type: Option<PostSortType>,
  #[ts(optional)]
  pub default_comment_sort_type: Option<CommentSortType>,
  #[ts(optional)]
  pub legal_information: Option<String>,
  #[ts(optional)]
  pub application_email_admins: Option<bool>,
  #[ts(optional)]
  pub hide_modlog_mod_names: Option<bool>,
  #[ts(optional)]
  pub discussion_languages: Option<Vec<LanguageId>>,
  #[ts(optional)]
  pub slur_filter_regex: Option<String>,
  #[ts(optional)]
  pub actor_name_max_length: Option<i32>,
  #[ts(optional)]
  pub rate_limit_message: Option<i32>,
  #[ts(optional)]
  pub rate_limit_message_per_second: Option<i32>,
  #[ts(optional)]
  pub rate_limit_post: Option<i32>,
  #[ts(optional)]
  pub rate_limit_post_per_second: Option<i32>,
  #[ts(optional)]
  pub rate_limit_register: Option<i32>,
  #[ts(optional)]
  pub rate_limit_register_per_second: Option<i32>,
  #[ts(optional)]
  pub rate_limit_image: Option<i32>,
  #[ts(optional)]
  pub rate_limit_image_per_second: Option<i32>,
  #[ts(optional)]
  pub rate_limit_comment: Option<i32>,
  #[ts(optional)]
  pub rate_limit_comment_per_second: Option<i32>,
  #[ts(optional)]
  pub rate_limit_search: Option<i32>,
  #[ts(optional)]
  pub rate_limit_search_per_second: Option<i32>,
  #[ts(optional)]
  pub federation_enabled: Option<bool>,
  #[ts(optional)]
  pub federation_debug: Option<bool>,
  #[ts(optional)]
  pub captcha_enabled: Option<bool>,
  #[ts(optional)]
  pub captcha_difficulty: Option<String>,
  #[ts(optional)]
  pub allowed_instances: Option<Vec<String>>,
  #[ts(optional)]
  pub blocked_instances: Option<Vec<String>>,
  #[ts(optional)]
  pub registration_mode: Option<RegistrationMode>,
  #[ts(optional)]
  pub oauth_registration: Option<bool>,
  #[ts(optional)]
  pub content_warning: Option<String>,
  #[ts(optional)]
  pub post_upvotes: Option<FederationMode>,
  #[ts(optional)]
  pub post_downvotes: Option<FederationMode>,
  #[ts(optional)]
  pub comment_upvotes: Option<FederationMode>,
  #[ts(optional)]
  pub comment_downvotes: Option<FederationMode>,
}

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "full", derive(TS))]
#[cfg_attr(feature = "full", ts(export))]
/// Edits a site.
pub struct EditSite {
  #[ts(optional)]
  pub name: Option<String>,
  /// A sidebar for the site, in markdown.
  #[ts(optional)]
  pub sidebar: Option<String>,
  /// A shorter, one line description of your site.
  #[ts(optional)]
  pub description: Option<String>,
  /// A url for your site's icon.
  #[ts(optional)]
  pub icon: Option<String>,
  /// A url for your site's banner.
  #[ts(optional)]
  pub banner: Option<String>,
  /// Whether to enable NSFW.
  #[ts(optional)]
  pub enable_nsfw: Option<bool>,
  /// Limits community creation to admins only.
  #[ts(optional)]
  pub community_creation_admin_only: Option<bool>,
  /// Whether to require email verification.
  #[ts(optional)]
  pub require_email_verification: Option<bool>,
  /// Your application question form. This is in markdown, and can be many questions.
  #[ts(optional)]
  pub application_question: Option<String>,
  /// Whether your instance is public, or private.
  #[ts(optional)]
  pub private_instance: Option<bool>,
  /// The default theme. Usually "browser"
  #[ts(optional)]
  pub default_theme: Option<String>,
  /// The default post listing type, usually "local"
  #[ts(optional)]
  pub default_post_listing_type: Option<ListingType>,
  /// Default value for listing mode, usually "list"
  #[ts(optional)]
  pub default_post_listing_mode: Option<PostListingMode>,
  /// The default post sort, usually "active"
  #[ts(optional)]
  pub default_post_sort_type: Option<PostSortType>,
  /// The default comment sort, usually "hot"
  #[ts(optional)]
  pub default_comment_sort_type: Option<CommentSortType>,
  /// An optional page of legal information
  #[ts(optional)]
  pub legal_information: Option<String>,
  /// Whether to email admins when receiving a new application.
  #[ts(optional)]
  pub application_email_admins: Option<bool>,
  /// Whether to hide moderator names from the modlog.
  #[ts(optional)]
  pub hide_modlog_mod_names: Option<bool>,
  /// A list of allowed discussion languages.
  #[ts(optional)]
  pub discussion_languages: Option<Vec<LanguageId>>,
  /// A regex string of items to filter.
  #[ts(optional)]
  pub slur_filter_regex: Option<String>,
  /// The max length of actor names.
  #[ts(optional)]
  pub actor_name_max_length: Option<i32>,
  /// The number of messages allowed in a given time frame.
  #[ts(optional)]
  pub rate_limit_message: Option<i32>,
  #[ts(optional)]
  pub rate_limit_message_per_second: Option<i32>,
  /// The number of posts allowed in a given time frame.
  #[ts(optional)]
  pub rate_limit_post: Option<i32>,
  #[ts(optional)]
  pub rate_limit_post_per_second: Option<i32>,
  /// The number of registrations allowed in a given time frame.
  #[ts(optional)]
  pub rate_limit_register: Option<i32>,
  #[ts(optional)]
  pub rate_limit_register_per_second: Option<i32>,
  /// The number of image uploads allowed in a given time frame.
  #[ts(optional)]
  pub rate_limit_image: Option<i32>,
  #[ts(optional)]
  pub rate_limit_image_per_second: Option<i32>,
  /// The number of comments allowed in a given time frame.
  #[ts(optional)]
  pub rate_limit_comment: Option<i32>,
  #[ts(optional)]
  pub rate_limit_comment_per_second: Option<i32>,
  /// The number of searches allowed in a given time frame.
  #[ts(optional)]
  pub rate_limit_search: Option<i32>,
  #[ts(optional)]
  pub rate_limit_search_per_second: Option<i32>,
  /// Whether to enable federation.
  #[ts(optional)]
  pub federation_enabled: Option<bool>,
  /// Enables federation debugging.
  #[ts(optional)]
  pub federation_debug: Option<bool>,
  /// Whether to enable captchas for signups.
  #[ts(optional)]
  pub captcha_enabled: Option<bool>,
  /// The captcha difficulty. Can be easy, medium, or hard
  #[ts(optional)]
  pub captcha_difficulty: Option<String>,
  /// A list of allowed instances. If none are set, federation is open.
  #[ts(optional)]
  pub allowed_instances: Option<Vec<String>>,
  /// A list of blocked instances.
  #[ts(optional)]
  pub blocked_instances: Option<Vec<String>>,
  /// A list of blocked URLs
  #[ts(optional)]
  pub blocked_urls: Option<Vec<String>>,
  #[ts(optional)]
  pub registration_mode: Option<RegistrationMode>,
  /// Whether to email admins for new reports.
  #[ts(optional)]
  pub reports_email_admins: Option<bool>,
  /// If present, nsfw content is visible by default. Should be displayed by frontends/clients
  /// when the site is first opened by a user.
  #[ts(optional)]
  pub content_warning: Option<String>,
  /// Whether or not external auth methods can auto-register users.
  #[ts(optional)]
  pub oauth_registration: Option<bool>,
  /// What kind of post upvotes your site allows.
  #[ts(optional)]
  pub post_upvotes: Option<FederationMode>,
  /// What kind of post downvotes your site allows.
  #[ts(optional)]
  pub post_downvotes: Option<FederationMode>,
  /// What kind of comment upvotes your site allows.
  #[ts(optional)]
  pub comment_upvotes: Option<FederationMode>,
  /// What kind of comment downvotes your site allows.
  #[ts(optional)]
  pub comment_downvotes: Option<FederationMode>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "full", derive(TS))]
#[cfg_attr(feature = "full", ts(export))]
/// The response for a site.
pub struct SiteResponse {
  pub site_view: SiteView,
  /// deprecated, use field `tagline` or /api/v3/tagline/list
  pub taglines: Vec<()>,
}

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "full", derive(TS))]
#[cfg_attr(feature = "full", ts(export))]
/// An expanded response for a site.
pub struct GetSiteResponse {
  pub site_view: SiteView,
  pub admins: Vec<PersonView>,
  pub version: String,
  #[ts(optional)]
  pub my_user: Option<MyUserInfo>,
  pub all_languages: Vec<Language>,
  pub discussion_languages: Vec<LanguageId>,
  /// deprecated, use field `tagline` or /api/v3/tagline/list
  pub taglines: Vec<()>,
  /// deprecated, use /api/v3/custom_emoji/list
  pub custom_emojis: Vec<()>,
  /// If the site has any taglines, a random one is included here for displaying
  #[ts(optional)]
  pub tagline: Option<Tagline>,
  /// A list of external auth methods your site supports.
  #[ts(optional)]
  pub oauth_providers: Option<Vec<PublicOAuthProvider>>,
  #[ts(optional)]
  pub admin_oauth_providers: Option<Vec<OAuthProvider>>,
  pub blocked_urls: Vec<LocalSiteUrlBlocklist>,
}

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "full", derive(TS))]
#[cfg_attr(feature = "full", ts(export))]
/// A response of federated instances.
pub struct GetFederatedInstancesResponse {
  /// Optional, because federation may be disabled.
  #[ts(optional)]
  pub federated_instances: Option<FederatedInstances>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "full", derive(TS))]
#[cfg_attr(feature = "full", ts(export))]
/// Your user info.
pub struct MyUserInfo {
  pub local_user_view: LocalUserView,
  pub follows: Vec<CommunityFollowerView>,
  pub moderates: Vec<CommunityModeratorView>,
  pub community_blocks: Vec<Community>,
  pub instance_blocks: Vec<Instance>,
  pub person_blocks: Vec<Person>,
  pub discussion_languages: Vec<LanguageId>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "full", derive(TS))]
#[cfg_attr(feature = "full", ts(export))]
/// A list of federated instances.
pub struct FederatedInstances {
  pub linked: Vec<InstanceWithFederationState>,
  pub allowed: Vec<InstanceWithFederationState>,
  pub blocked: Vec<InstanceWithFederationState>,
}

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "full", derive(TS))]
#[cfg_attr(feature = "full", ts(export))]
pub struct ReadableFederationState {
  #[serde(flatten)]
  internal_state: FederationQueueState,
  /// timestamp of the next retry attempt (null if fail count is 0)
  #[ts(optional)]
  next_retry: Option<DateTime<Utc>>,
}

impl From<FederationQueueState> for ReadableFederationState {
  fn from(internal_state: FederationQueueState) -> Self {
    ReadableFederationState {
      next_retry: internal_state.last_retry.map(|r| {
        r + chrono::Duration::from_std(federate_retry_sleep_duration(internal_state.fail_count))
          .expect("sleep duration longer than 2**63 ms (262 million years)")
      }),
      internal_state,
    }
  }
}

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "full", derive(TS))]
#[cfg_attr(feature = "full", ts(export))]
pub struct InstanceWithFederationState {
  #[serde(flatten)]
  pub instance: Instance,
  /// if federation to this instance is or was active, show state of outgoing federation to this
  /// instance
  #[ts(optional)]
  pub federation_state: Option<ReadableFederationState>,
}

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "full", derive(TS))]
#[cfg_attr(feature = "full", ts(export))]
/// Purges a person from the database. This will delete all content attached to that person.
pub struct PurgePerson {
  pub person_id: PersonId,
  #[ts(optional)]
  pub reason: Option<String>,
}

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "full", derive(TS))]
#[cfg_attr(feature = "full", ts(export))]
/// Purges a community from the database. This will delete all content attached to that community.
pub struct PurgeCommunity {
  pub community_id: CommunityId,
  #[ts(optional)]
  pub reason: Option<String>,
}

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "full", derive(TS))]
#[cfg_attr(feature = "full", ts(export))]
/// Purges a post from the database. This will delete all content attached to that post.
pub struct PurgePost {
  pub post_id: PostId,
  #[ts(optional)]
  pub reason: Option<String>,
}

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "full", derive(TS))]
#[cfg_attr(feature = "full", ts(export))]
/// Purges a comment from the database. This will delete all content attached to that comment.
pub struct PurgeComment {
  pub comment_id: CommentId,
  #[ts(optional)]
  pub reason: Option<String>,
}

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Clone, Copy, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "full", derive(TS))]
#[cfg_attr(feature = "full", ts(export))]
/// Fetches a list of registration applications.
pub struct ListRegistrationApplications {
  /// Only shows the unread applications (IE those without an admin actor)
  #[ts(optional)]
  pub unread_only: Option<bool>,
  #[ts(optional)]
  pub page: Option<i64>,
  #[ts(optional)]
  pub limit: Option<i64>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "full", derive(TS))]
#[cfg_attr(feature = "full", ts(export))]
/// The list of registration applications.
pub struct ListRegistrationApplicationsResponse {
  pub registration_applications: Vec<RegistrationApplicationView>,
}

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Clone, Copy, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "full", derive(TS))]
#[cfg_attr(feature = "full", ts(export))]
/// Gets a registration application for a person
pub struct GetRegistrationApplication {
  pub person_id: PersonId,
}

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "full", derive(TS))]
#[cfg_attr(feature = "full", ts(export))]
/// Approves a registration application.
pub struct ApproveRegistrationApplication {
  pub id: RegistrationApplicationId,
  pub approve: bool,
  #[ts(optional)]
  pub deny_reason: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "full", derive(TS))]
#[cfg_attr(feature = "full", ts(export))]
/// The response of an action done to a registration application.
pub struct RegistrationApplicationResponse {
  pub registration_application: RegistrationApplicationView,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "full", derive(TS))]
#[cfg_attr(feature = "full", ts(export))]
/// The count of unread registration applications.
pub struct GetUnreadRegistrationApplicationCountResponse {
  pub registration_applications: i64,
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "full", derive(TS))]
#[cfg_attr(feature = "full", ts(export))]
/// Block an instance as user
pub struct BlockInstance {
  pub instance_id: InstanceId,
  pub block: bool,
}

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "full", derive(TS))]
#[cfg_attr(feature = "full", ts(export))]
pub struct BlockInstanceResponse {
  pub blocked: bool,
}
