use lemmy_db_schema::newtypes::LanguageId;
use lemmy_db_schema_file::enums::{
  CommentSortType, FederationMode, ListingType, PostListingMode, PostSortType, RegistrationMode,
};
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
#[cfg(feature = "full")]
use ts_rs::TS;

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
  pub disallow_nsfw_content: Option<bool>,
  #[cfg_attr(feature = "full", ts(optional))]
  pub disable_email_notifications: Option<bool>,
}
