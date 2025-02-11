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
  SearchSortType,
  SearchType,
};
use lemmy_db_views::structs::{
  CommentView,
  CommunityFollowerView,
  CommunityModeratorView,
  CommunityView,
  LocalUserView,
  ModlogCombinedPaginationCursor,
  ModlogCombinedView,
  PersonView,
  PostView,
  RegistrationApplicationView,
  SearchCombinedPaginationCursor,
  SearchCombinedView,
  SiteView,
};
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
#[cfg(feature = "full")]
use ts_rs::TS;

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "full", derive(TS))]
#[cfg_attr(feature = "full", ts(export))]
/// Searches the site, given a search term, and some optional filters.
pub struct Search {
  #[cfg_attr(feature = "full", ts(optional))]
  pub search_term: Option<String>,
  #[cfg_attr(feature = "full", ts(optional))]
  pub community_id: Option<CommunityId>,
  #[cfg_attr(feature = "full", ts(optional))]
  pub community_name: Option<String>,
  #[cfg_attr(feature = "full", ts(optional))]
  pub creator_id: Option<PersonId>,
  #[cfg_attr(feature = "full", ts(optional))]
  pub type_: Option<SearchType>,
  #[cfg_attr(feature = "full", ts(optional))]
  pub sort: Option<SearchSortType>,
  #[cfg_attr(feature = "full", ts(optional))]
  /// Filter to within a given time range, in seconds.
  /// IE 60 would give results for the past minute.
  pub time_range_seconds: Option<i32>,
  #[cfg_attr(feature = "full", ts(optional))]
  pub listing_type: Option<ListingType>,
  #[cfg_attr(feature = "full", ts(optional))]
  pub title_only: Option<bool>,
  #[cfg_attr(feature = "full", ts(optional))]
  pub post_url_only: Option<bool>,
  #[cfg_attr(feature = "full", ts(optional))]
  pub liked_only: Option<bool>,
  #[cfg_attr(feature = "full", ts(optional))]
  pub disliked_only: Option<bool>,
  #[cfg_attr(feature = "full", ts(optional))]
  pub page_cursor: Option<SearchCombinedPaginationCursor>,
  #[cfg_attr(feature = "full", ts(optional))]
  pub page_back: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "full", derive(TS))]
#[cfg_attr(feature = "full", ts(export))]
/// The search response, containing lists of the return type possibilities
pub struct SearchResponse {
  pub results: Vec<SearchCombinedView>,
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
  #[cfg_attr(feature = "full", ts(optional))]
  pub comment: Option<CommentView>,
  #[cfg_attr(feature = "full", ts(optional))]
  pub post: Option<PostView>,
  #[cfg_attr(feature = "full", ts(optional))]
  pub community: Option<CommunityView>,
  #[cfg_attr(feature = "full", ts(optional))]
  pub person: Option<PersonView>,
}

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "full", derive(TS))]
#[cfg_attr(feature = "full", ts(export))]
/// Fetches the modlog.
pub struct GetModlog {
  /// Filter by the moderator.
  #[cfg_attr(feature = "full", ts(optional))]
  pub mod_person_id: Option<PersonId>,
  /// Filter by the community.
  #[cfg_attr(feature = "full", ts(optional))]
  pub community_id: Option<CommunityId>,
  /// Filter by the modlog action type.
  #[cfg_attr(feature = "full", ts(optional))]
  pub type_: Option<ModlogActionType>,
  /// Filter by listing type. When not using All, it will remove the non-community modlog entries,
  /// such as site bans, instance blocks, adding an admin, etc.
  #[cfg_attr(feature = "full", ts(optional))]
  pub listing_type: Option<ListingType>,
  /// Filter by the other / modded person.
  #[cfg_attr(feature = "full", ts(optional))]
  pub other_person_id: Option<PersonId>,
  /// Filter by post. Will include comments of that post.
  #[cfg_attr(feature = "full", ts(optional))]
  pub post_id: Option<PostId>,
  /// Filter by comment.
  #[cfg_attr(feature = "full", ts(optional))]
  pub comment_id: Option<CommentId>,
  #[cfg_attr(feature = "full", ts(optional))]
  pub page_cursor: Option<ModlogCombinedPaginationCursor>,
  #[cfg_attr(feature = "full", ts(optional))]
  pub page_back: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "full", derive(TS))]
#[cfg_attr(feature = "full", ts(export))]
/// The modlog fetch response.
pub struct GetModlogResponse {
  pub modlog: Vec<ModlogCombinedView>,
}

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "full", derive(TS))]
#[cfg_attr(feature = "full", ts(export))]
/// Creates a site. Should be done after first running lemmy.
pub struct CreateSite {
  pub name: String,
  #[cfg_attr(feature = "full", ts(optional))]
  pub sidebar: Option<String>,
  #[cfg_attr(feature = "full", ts(optional))]
  pub description: Option<String>,
  #[cfg_attr(feature = "full", ts(optional))]
  #[cfg_attr(feature = "full", ts(optional))]
  pub community_creation_admin_only: Option<bool>,
  #[cfg_attr(feature = "full", ts(optional))]
  pub require_email_verification: Option<bool>,
  #[cfg_attr(feature = "full", ts(optional))]
  pub application_question: Option<String>,
  #[cfg_attr(feature = "full", ts(optional))]
  pub private_instance: Option<bool>,
  #[cfg_attr(feature = "full", ts(optional))]
  pub default_theme: Option<String>,
  #[cfg_attr(feature = "full", ts(optional))]
  pub default_post_listing_type: Option<ListingType>,
  #[cfg_attr(feature = "full", ts(optional))]
  pub default_post_listing_mode: Option<PostListingMode>,
  #[cfg_attr(feature = "full", ts(optional))]
  pub default_post_sort_type: Option<PostSortType>,
  #[cfg_attr(feature = "full", ts(optional))]
  pub default_post_time_range_seconds: Option<i32>,
  #[cfg_attr(feature = "full", ts(optional))]
  pub default_comment_sort_type: Option<CommentSortType>,
  #[cfg_attr(feature = "full", ts(optional))]
  pub legal_information: Option<String>,
  #[cfg_attr(feature = "full", ts(optional))]
  pub application_email_admins: Option<bool>,
  #[cfg_attr(feature = "full", ts(optional))]
  pub hide_modlog_mod_names: Option<bool>,
  #[cfg_attr(feature = "full", ts(optional))]
  pub discussion_languages: Option<Vec<LanguageId>>,
  #[cfg_attr(feature = "full", ts(optional))]
  pub slur_filter_regex: Option<String>,
  #[cfg_attr(feature = "full", ts(optional))]
  pub actor_name_max_length: Option<i32>,
  #[cfg_attr(feature = "full", ts(optional))]
  pub rate_limit_message: Option<i32>,
  #[cfg_attr(feature = "full", ts(optional))]
  pub rate_limit_message_per_second: Option<i32>,
  #[cfg_attr(feature = "full", ts(optional))]
  pub rate_limit_post: Option<i32>,
  #[cfg_attr(feature = "full", ts(optional))]
  pub rate_limit_post_per_second: Option<i32>,
  #[cfg_attr(feature = "full", ts(optional))]
  pub rate_limit_register: Option<i32>,
  #[cfg_attr(feature = "full", ts(optional))]
  pub rate_limit_register_per_second: Option<i32>,
  #[cfg_attr(feature = "full", ts(optional))]
  pub rate_limit_image: Option<i32>,
  #[cfg_attr(feature = "full", ts(optional))]
  pub rate_limit_image_per_second: Option<i32>,
  #[cfg_attr(feature = "full", ts(optional))]
  pub rate_limit_comment: Option<i32>,
  #[cfg_attr(feature = "full", ts(optional))]
  pub rate_limit_comment_per_second: Option<i32>,
  #[cfg_attr(feature = "full", ts(optional))]
  pub rate_limit_search: Option<i32>,
  #[cfg_attr(feature = "full", ts(optional))]
  pub rate_limit_search_per_second: Option<i32>,
  #[cfg_attr(feature = "full", ts(optional))]
  pub federation_enabled: Option<bool>,
  #[cfg_attr(feature = "full", ts(optional))]
  pub captcha_enabled: Option<bool>,
  #[cfg_attr(feature = "full", ts(optional))]
  pub captcha_difficulty: Option<String>,
  #[cfg_attr(feature = "full", ts(optional))]
  pub registration_mode: Option<RegistrationMode>,
  #[cfg_attr(feature = "full", ts(optional))]
  pub oauth_registration: Option<bool>,
  #[cfg_attr(feature = "full", ts(optional))]
  pub content_warning: Option<String>,
  #[cfg_attr(feature = "full", ts(optional))]
  pub post_upvotes: Option<FederationMode>,
  #[cfg_attr(feature = "full", ts(optional))]
  pub post_downvotes: Option<FederationMode>,
  #[cfg_attr(feature = "full", ts(optional))]
  pub comment_upvotes: Option<FederationMode>,
  #[cfg_attr(feature = "full", ts(optional))]
  pub comment_downvotes: Option<FederationMode>,
  #[cfg_attr(feature = "full", ts(optional))]
  pub disable_donation_dialog: Option<bool>,
}

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "full", derive(TS))]
#[cfg_attr(feature = "full", ts(export))]
/// Edits a site.
pub struct EditSite {
  #[cfg_attr(feature = "full", ts(optional))]
  pub name: Option<String>,
  /// A sidebar for the site, in markdown.
  #[cfg_attr(feature = "full", ts(optional))]
  pub sidebar: Option<String>,
  /// A shorter, one line description of your site.
  #[cfg_attr(feature = "full", ts(optional))]
  pub description: Option<String>,
  /// Limits community creation to admins only.
  #[cfg_attr(feature = "full", ts(optional))]
  pub community_creation_admin_only: Option<bool>,
  /// Whether to require email verification.
  #[cfg_attr(feature = "full", ts(optional))]
  pub require_email_verification: Option<bool>,
  /// Your application question form. This is in markdown, and can be many questions.
  #[cfg_attr(feature = "full", ts(optional))]
  pub application_question: Option<String>,
  /// Whether your instance is public, or private.
  #[cfg_attr(feature = "full", ts(optional))]
  pub private_instance: Option<bool>,
  /// The default theme. Usually "browser"
  #[cfg_attr(feature = "full", ts(optional))]
  pub default_theme: Option<String>,
  /// The default post listing type, usually "local"
  #[cfg_attr(feature = "full", ts(optional))]
  pub default_post_listing_type: Option<ListingType>,
  /// Default value for listing mode, usually "list"
  #[cfg_attr(feature = "full", ts(optional))]
  pub default_post_listing_mode: Option<PostListingMode>,
  /// The default post sort, usually "active"
  #[cfg_attr(feature = "full", ts(optional))]
  pub default_post_sort_type: Option<PostSortType>,
  /// A default time range limit to apply to post sorts, in seconds. 0 means none.
  #[cfg_attr(feature = "full", ts(optional))]
  pub default_post_time_range_seconds: Option<i32>,
  /// The default comment sort, usually "hot"
  #[cfg_attr(feature = "full", ts(optional))]
  pub default_comment_sort_type: Option<CommentSortType>,
  /// An optional page of legal information
  #[cfg_attr(feature = "full", ts(optional))]
  pub legal_information: Option<String>,
  /// Whether to email admins when receiving a new application.
  #[cfg_attr(feature = "full", ts(optional))]
  pub application_email_admins: Option<bool>,
  /// Whether to hide moderator names from the modlog.
  #[cfg_attr(feature = "full", ts(optional))]
  pub hide_modlog_mod_names: Option<bool>,
  /// A list of allowed discussion languages.
  #[cfg_attr(feature = "full", ts(optional))]
  pub discussion_languages: Option<Vec<LanguageId>>,
  /// A regex string of items to filter.
  #[cfg_attr(feature = "full", ts(optional))]
  pub slur_filter_regex: Option<String>,
  /// The max length of actor names.
  #[cfg_attr(feature = "full", ts(optional))]
  pub actor_name_max_length: Option<i32>,
  /// The number of messages allowed in a given time frame.
  #[cfg_attr(feature = "full", ts(optional))]
  pub rate_limit_message: Option<i32>,
  #[cfg_attr(feature = "full", ts(optional))]
  pub rate_limit_message_per_second: Option<i32>,
  /// The number of posts allowed in a given time frame.
  #[cfg_attr(feature = "full", ts(optional))]
  pub rate_limit_post: Option<i32>,
  #[cfg_attr(feature = "full", ts(optional))]
  pub rate_limit_post_per_second: Option<i32>,
  /// The number of registrations allowed in a given time frame.
  #[cfg_attr(feature = "full", ts(optional))]
  pub rate_limit_register: Option<i32>,
  #[cfg_attr(feature = "full", ts(optional))]
  pub rate_limit_register_per_second: Option<i32>,
  /// The number of image uploads allowed in a given time frame.
  #[cfg_attr(feature = "full", ts(optional))]
  pub rate_limit_image: Option<i32>,
  #[cfg_attr(feature = "full", ts(optional))]
  pub rate_limit_image_per_second: Option<i32>,
  /// The number of comments allowed in a given time frame.
  #[cfg_attr(feature = "full", ts(optional))]
  pub rate_limit_comment: Option<i32>,
  #[cfg_attr(feature = "full", ts(optional))]
  pub rate_limit_comment_per_second: Option<i32>,
  /// The number of searches allowed in a given time frame.
  #[cfg_attr(feature = "full", ts(optional))]
  pub rate_limit_search: Option<i32>,
  #[cfg_attr(feature = "full", ts(optional))]
  pub rate_limit_search_per_second: Option<i32>,
  /// Whether to enable federation.
  #[cfg_attr(feature = "full", ts(optional))]
  pub federation_enabled: Option<bool>,
  /// Whether to enable captchas for signups.
  #[cfg_attr(feature = "full", ts(optional))]
  pub captcha_enabled: Option<bool>,
  /// The captcha difficulty. Can be easy, medium, or hard
  #[cfg_attr(feature = "full", ts(optional))]
  pub captcha_difficulty: Option<String>,
  /// A list of blocked URLs
  #[cfg_attr(feature = "full", ts(optional))]
  pub blocked_urls: Option<Vec<String>>,
  #[cfg_attr(feature = "full", ts(optional))]
  pub registration_mode: Option<RegistrationMode>,
  /// Whether to email admins for new reports.
  #[cfg_attr(feature = "full", ts(optional))]
  pub reports_email_admins: Option<bool>,
  /// If present, nsfw content is visible by default. Should be displayed by frontends/clients
  /// when the site is first opened by a user.
  #[cfg_attr(feature = "full", ts(optional))]
  pub content_warning: Option<String>,
  /// Whether or not external auth methods can auto-register users.
  #[cfg_attr(feature = "full", ts(optional))]
  pub oauth_registration: Option<bool>,
  /// What kind of post upvotes your site allows.
  #[cfg_attr(feature = "full", ts(optional))]
  pub post_upvotes: Option<FederationMode>,
  /// What kind of post downvotes your site allows.
  #[cfg_attr(feature = "full", ts(optional))]
  pub post_downvotes: Option<FederationMode>,
  /// What kind of comment upvotes your site allows.
  #[cfg_attr(feature = "full", ts(optional))]
  pub comment_upvotes: Option<FederationMode>,
  /// What kind of comment downvotes your site allows.
  #[cfg_attr(feature = "full", ts(optional))]
  pub comment_downvotes: Option<FederationMode>,
  /// If this is true, users will never see the dialog asking to support Lemmy development with
  /// donations.
  #[cfg_attr(feature = "full", ts(optional))]
  pub disable_donation_dialog: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "full", derive(TS))]
#[cfg_attr(feature = "full", ts(export))]
/// The response for a site.
pub struct SiteResponse {
  pub site_view: SiteView,
  /// deprecated, use field `tagline` or /api/v4/tagline/list
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
  #[cfg_attr(feature = "full", ts(skip))]
  pub my_user: Option<MyUserInfo>,
  pub all_languages: Vec<Language>,
  pub discussion_languages: Vec<LanguageId>,
  /// If the site has any taglines, a random one is included here for displaying
  #[cfg_attr(feature = "full", ts(optional))]
  pub tagline: Option<Tagline>,
  /// A list of external auth methods your site supports.
  #[cfg_attr(feature = "full", ts(optional))]
  pub oauth_providers: Option<Vec<PublicOAuthProvider>>,
  #[cfg_attr(feature = "full", ts(optional))]
  pub admin_oauth_providers: Option<Vec<OAuthProvider>>,
  pub blocked_urls: Vec<LocalSiteUrlBlocklist>,
  // If true then uploads for post images or markdown images are disabled. Only avatars, icons and
  // banners can be set.
  pub image_upload_disabled: bool,
}

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "full", derive(TS))]
#[cfg_attr(feature = "full", ts(export))]
/// A response of federated instances.
pub struct GetFederatedInstancesResponse {
  /// Optional, because federation may be disabled.
  #[cfg_attr(feature = "full", ts(optional))]
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
  #[cfg_attr(feature = "full", ts(optional))]
  next_retry: Option<DateTime<Utc>>,
}

#[allow(clippy::expect_used)]
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
  #[cfg_attr(feature = "full", ts(optional))]
  pub federation_state: Option<ReadableFederationState>,
}

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "full", derive(TS))]
#[cfg_attr(feature = "full", ts(export))]
/// Purges a person from the database. This will delete all content attached to that person.
pub struct PurgePerson {
  pub person_id: PersonId,
  #[cfg_attr(feature = "full", ts(optional))]
  pub reason: Option<String>,
}

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "full", derive(TS))]
#[cfg_attr(feature = "full", ts(export))]
/// Purges a community from the database. This will delete all content attached to that community.
pub struct PurgeCommunity {
  pub community_id: CommunityId,
  #[cfg_attr(feature = "full", ts(optional))]
  pub reason: Option<String>,
}

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "full", derive(TS))]
#[cfg_attr(feature = "full", ts(export))]
/// Purges a post from the database. This will delete all content attached to that post.
pub struct PurgePost {
  pub post_id: PostId,
  #[cfg_attr(feature = "full", ts(optional))]
  pub reason: Option<String>,
}

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "full", derive(TS))]
#[cfg_attr(feature = "full", ts(export))]
/// Purges a comment from the database. This will delete all content attached to that comment.
pub struct PurgeComment {
  pub comment_id: CommentId,
  #[cfg_attr(feature = "full", ts(optional))]
  pub reason: Option<String>,
}

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Clone, Copy, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "full", derive(TS))]
#[cfg_attr(feature = "full", ts(export))]
/// Fetches a list of registration applications.
pub struct ListRegistrationApplications {
  /// Only shows the unread applications (IE those without an admin actor)
  #[cfg_attr(feature = "full", ts(optional))]
  pub unread_only: Option<bool>,
  #[cfg_attr(feature = "full", ts(optional))]
  pub page: Option<i64>,
  #[cfg_attr(feature = "full", ts(optional))]
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
  #[cfg_attr(feature = "full", ts(optional))]
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
pub struct UserBlockInstanceParams {
  pub instance_id: InstanceId,
  pub block: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "full", derive(TS))]
#[cfg_attr(feature = "full", ts(export))]
pub struct AdminBlockInstanceParams {
  pub instance: String,
  pub block: bool,
  #[cfg_attr(feature = "full", ts(optional))]
  pub reason: Option<String>,
  #[cfg_attr(feature = "full", ts(optional))]
  pub expires: Option<DateTime<Utc>>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "full", derive(TS))]
#[cfg_attr(feature = "full", ts(export))]
pub struct AdminAllowInstanceParams {
  pub instance: String,
  pub allow: bool,
  #[cfg_attr(feature = "full", ts(optional))]
  pub reason: Option<String>,
}
