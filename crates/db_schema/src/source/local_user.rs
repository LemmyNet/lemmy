#[cfg(feature = "full")]
use crate::schema::local_user;
use crate::{
  newtypes::{LocalUserId, PersonId},
  sensitive::SensitiveString,
  ListingType,
  PostListingMode,
  SortType,
};
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
#[cfg(feature = "full")]
use ts_rs::TS;

#[skip_serializing_none]
#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "full", derive(Queryable, Selectable, Identifiable, TS))]
#[cfg_attr(feature = "full", diesel(table_name = local_user))]
#[cfg_attr(feature = "full", diesel(check_for_backend(diesel::pg::Pg)))]
#[cfg_attr(feature = "full", ts(export))]
/// A local user.
pub struct LocalUser {
  pub id: LocalUserId,
  /// The person_id for the local user.
  pub person_id: PersonId,
  #[serde(skip)]
  pub password_encrypted: SensitiveString,
  pub email: Option<SensitiveString>,
  /// Whether to show NSFW content.
  pub show_nsfw: bool,
  pub theme: String,
  pub default_sort_type: SortType,
  pub default_listing_type: ListingType,
  pub interface_language: String,
  /// Whether to show avatars.
  pub show_avatars: bool,
  pub send_notifications_to_email: bool,
  /// Whether to show bot accounts.
  pub show_bot_accounts: bool,
  /// Whether to show read posts.
  pub show_read_posts: bool,
  /// Whether their email has been verified.
  pub email_verified: bool,
  /// Whether their registration application has been accepted.
  pub accepted_application: bool,
  #[serde(skip)]
  pub totp_2fa_secret: Option<SensitiveString>,
  /// Open links in a new tab.
  pub open_links_in_new_tab: bool,
  pub blur_nsfw: bool,
  pub auto_expand: bool,
  /// Whether infinite scroll is enabled.
  pub infinite_scroll_enabled: bool,
  /// Whether the person is an admin.
  pub admin: bool,
  /// A post-view mode that changes how multiple post listings look.
  pub post_listing_mode: PostListingMode,
  pub totp_2fa_enabled: bool,
  /// Whether to allow keyboard navigation (for browsing and interacting with posts and comments).
  pub enable_keyboard_navigation: bool,
  /// Whether user avatars and inline images in the UI that are gifs should be allowed to play or
  /// should be paused
  pub enable_animated_images: bool,
  /// Whether to auto-collapse bot comments.
  pub collapse_bot_comments: bool,
}

#[derive(Clone, derive_new::new)]
#[cfg_attr(feature = "full", derive(Insertable))]
#[cfg_attr(feature = "full", diesel(table_name = local_user))]
pub struct LocalUserInsertForm {
  pub person_id: PersonId,
  pub password_encrypted: String,
  #[new(default)]
  pub email: Option<String>,
  #[new(default)]
  pub show_nsfw: Option<bool>,
  #[new(default)]
  pub theme: Option<String>,
  #[new(default)]
  pub default_sort_type: Option<SortType>,
  #[new(default)]
  pub default_listing_type: Option<ListingType>,
  #[new(default)]
  pub interface_language: Option<String>,
  #[new(default)]
  pub show_avatars: Option<bool>,
  #[new(default)]
  pub send_notifications_to_email: Option<bool>,
  #[new(default)]
  pub show_bot_accounts: Option<bool>,
  #[new(default)]
  pub show_read_posts: Option<bool>,
  #[new(default)]
  pub email_verified: Option<bool>,
  #[new(default)]
  pub accepted_application: Option<bool>,
  #[new(default)]
  pub totp_2fa_secret: Option<Option<String>>,
  #[new(default)]
  pub open_links_in_new_tab: Option<bool>,
  #[new(default)]
  pub blur_nsfw: Option<bool>,
  #[new(default)]
  pub auto_expand: Option<bool>,
  #[new(default)]
  pub infinite_scroll_enabled: Option<bool>,
  #[new(default)]
  pub admin: Option<bool>,
  #[new(default)]
  pub post_listing_mode: Option<PostListingMode>,
  #[new(default)]
  pub totp_2fa_enabled: Option<bool>,
  #[new(default)]
  pub enable_keyboard_navigation: Option<bool>,
  #[new(default)]
  pub enable_animated_images: Option<bool>,
  #[new(default)]
  pub collapse_bot_comments: Option<bool>,
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
  pub show_read_posts: Option<bool>,
  pub email_verified: Option<bool>,
  pub accepted_application: Option<bool>,
  pub totp_2fa_secret: Option<Option<String>>,
  pub open_links_in_new_tab: Option<bool>,
  pub blur_nsfw: Option<bool>,
  pub auto_expand: Option<bool>,
  pub infinite_scroll_enabled: Option<bool>,
  pub admin: Option<bool>,
  pub post_listing_mode: Option<PostListingMode>,
  pub totp_2fa_enabled: Option<bool>,
  pub enable_keyboard_navigation: Option<bool>,
  pub enable_animated_images: Option<bool>,
  pub collapse_bot_comments: Option<bool>,
}
