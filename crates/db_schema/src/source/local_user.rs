#[cfg(feature = "full")]
use crate::schema::local_user;
use crate::{
  newtypes::{LocalUserId, PersonId},
  ListingType,
  PostListingMode,
  SortType,
};
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
#[cfg(feature = "full")]
use ts_rs::TS;
use typed_builder::TypedBuilder;

#[skip_serializing_none]
#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "full", derive(Queryable, Identifiable, TS))]
#[cfg_attr(feature = "full", diesel(table_name = local_user))]
#[cfg_attr(feature = "full", ts(export))]
/// A local user.
pub struct LocalUser {
  pub id: LocalUserId,
  /// The person_id for the local user.
  pub person_id: PersonId,
  #[serde(skip)]
  pub password_encrypted: String,
  pub email: Option<String>,
  /// Whether to show NSFW content.
  pub show_nsfw: bool,
  pub theme: String,
  pub default_sort_type: SortType,
  pub default_listing_type: ListingType,
  pub interface_language: String,
  /// Whether to show avatars.
  pub show_avatars: bool,
  pub send_notifications_to_email: bool,
  /// Whether to show comment / post scores.
  pub show_scores: bool,
  /// Whether to show bot accounts.
  pub show_bot_accounts: bool,
  /// Whether to show read posts.
  pub show_read_posts: bool,
  /// Whether to show new posts as notifications.
  pub show_new_post_notifs: bool,
  /// Whether their email has been verified.
  pub email_verified: bool,
  /// Whether their registration application has been accepted.
  pub accepted_application: bool,
  #[serde(skip)]
  pub totp_2fa_secret: Option<String>,
  /// A URL to add their 2-factor auth.
  pub totp_2fa_url: Option<String>,
  /// Open links in a new tab.
  pub open_links_in_new_tab: bool,
  pub blur_nsfw: bool,
  pub auto_expand: bool,
  /// Whether infinite scroll is enabled.
  pub infinite_scroll_enabled: bool,
  /// Whether the person is an admin.
  pub admin: bool,
  pub post_listing_mode: PostListingMode,
}

#[derive(Clone, TypedBuilder)]
#[builder(field_defaults(default))]
#[cfg_attr(feature = "full", derive(Insertable))]
#[cfg_attr(feature = "full", diesel(table_name = local_user))]
pub struct LocalUserInsertForm {
  #[builder(!default)]
  pub person_id: PersonId,
  #[builder(!default)]
  pub password_encrypted: String,
  pub email: Option<String>,
  pub show_nsfw: Option<bool>,
  pub theme: Option<String>,
  pub default_sort_type: Option<SortType>,
  pub default_listing_type: Option<ListingType>,
  pub interface_language: Option<String>,
  pub show_avatars: Option<bool>,
  pub send_notifications_to_email: Option<bool>,
  pub show_bot_accounts: Option<bool>,
  pub show_scores: Option<bool>,
  pub show_read_posts: Option<bool>,
  pub show_new_post_notifs: Option<bool>,
  pub email_verified: Option<bool>,
  pub accepted_application: Option<bool>,
  pub totp_2fa_secret: Option<Option<String>>,
  pub totp_2fa_url: Option<Option<String>>,
  pub open_links_in_new_tab: Option<bool>,
  pub blur_nsfw: Option<bool>,
  pub auto_expand: Option<bool>,
  pub infinite_scroll_enabled: Option<bool>,
  pub admin: Option<bool>,
  pub post_listing_mode: Option<PostListingMode>,
}

#[derive(Clone, Default)]
#[cfg_attr(feature = "full", derive(AsChangeset))]
#[cfg_attr(feature = "full", diesel(table_name = local_user))]
pub struct LocalUserUpdateForm {
  pub password_encrypted: Option<String>,
  pub email: Option<Option<String>>,
  pub show_nsfw: Option<bool>,
  pub theme: Option<String>,
  pub default_sort_type: Option<SortType>,
  pub default_listing_type: Option<ListingType>,
  pub interface_language: Option<String>,
  pub show_avatars: Option<bool>,
  pub send_notifications_to_email: Option<bool>,
  pub show_bot_accounts: Option<bool>,
  pub show_scores: Option<bool>,
  pub show_read_posts: Option<bool>,
  pub show_new_post_notifs: Option<bool>,
  pub email_verified: Option<bool>,
  pub accepted_application: Option<bool>,
  pub totp_2fa_secret: Option<Option<String>>,
  pub totp_2fa_url: Option<Option<String>>,
  pub open_links_in_new_tab: Option<bool>,
  pub blur_nsfw: Option<bool>,
  pub auto_expand: Option<bool>,
  pub infinite_scroll_enabled: Option<bool>,
  pub admin: Option<bool>,
  pub post_listing_mode: Option<PostListingMode>,
}
