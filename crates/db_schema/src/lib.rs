#[cfg(feature = "full")]
#[macro_use]
extern crate diesel;
#[cfg(feature = "full")]
#[macro_use]
extern crate diesel_derive_newtype;

#[cfg(feature = "full")]
pub mod impls;
pub mod newtypes;
pub mod source;
#[cfg(feature = "full")]
pub mod test_data;
#[cfg(feature = "full")]
pub mod traits;
#[cfg(feature = "full")]
pub mod utils;

use lemmy_db_schema_file::enums::{ModlogKind, NotificationType};
use serde::{Deserialize, Serialize};
use strum::{Display, EnumString};
#[cfg(feature = "full")]
use {
  diesel::query_source::AliasedField,
  lemmy_db_schema_file::{
    aliases,
    schema::{instance_actions, person},
  },
};

#[derive(
  EnumString, Display, Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Default, Hash,
)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(export))]
/// The search sort types.
pub enum SearchSortType {
  #[default]
  New,
  Top,
  Old,
}

/// The community sort types. See here for descriptions: https://join-lemmy.org/docs/en/users/03-votes-and-ranking.html
#[derive(Debug, Serialize, Deserialize, Clone, Copy, Default, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(export))]
pub enum CommunitySortType {
  ActiveSixMonths,
  #[default]
  ActiveMonthly,
  ActiveWeekly,
  ActiveDaily,
  Hot,
  New,
  Old,
  NameAsc,
  NameDesc,
  Comments,
  Posts,
  Subscribers,
  SubscribersLocal,
}

/// The local user sort type.
#[derive(Debug, Serialize, Deserialize, Clone, Copy, Default, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(export))]
pub enum LocalUserSortType {
  #[default]
  New,
  Old,
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy, Default, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(export))]
pub enum MultiCommunitySortType {
  New,
  Old,
  NameAsc,
  NameDesc,
  Communities,
  #[default]
  Subscribers,
  SubscribersLocal,
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy, Default, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(export))]
/// A listing type for multi-community fetches.
pub enum MultiCommunityListingType {
  /// Content from your own site, as well as all connected / federated sites.
  All,
  /// Content from your site only.
  #[default]
  Local,
  /// Content only from communities you've subscribed to.
  Subscribed,
}

#[derive(
  EnumString, Display, Debug, Serialize, Deserialize, Default, Clone, Copy, PartialEq, Eq, Hash,
)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(export))]
/// The type of content returned from a search.
pub enum SearchType {
  #[default]
  All,
  Comments,
  Posts,
  Communities,
  Users,
  MultiCommunities,
}

#[derive(
  EnumString, Display, Debug, Serialize, Deserialize, Default, Clone, Copy, PartialEq, Eq, Hash,
)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(export))]
/// A list of possible types for the inbox.
pub enum NotificationTypeFilter {
  #[default]
  All,
  #[serde(untagged)]
  Other(NotificationType),
}

#[derive(
  EnumString, Display, Debug, Serialize, Deserialize, Default, Clone, Copy, PartialEq, Eq, Hash,
)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(export))]
/// A list of possible types for the various modlog actions.
pub enum ModlogKindFilter {
  #[default]
  All,
  #[serde(untagged)]
  Other(ModlogKind),
}

#[derive(EnumString, Display, Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(export))]
/// A list of possible types for a person's content.
pub enum PersonContentType {
  All,
  Comments,
  Posts,
}

#[derive(EnumString, Display, Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(export))]
/// A list of possible types for reports.
pub enum ReportType {
  All,
  Posts,
  Comments,
  PrivateMessages,
  Communities,
}

#[derive(
  EnumString, Display, Debug, Serialize, Deserialize, Clone, Copy, Default, PartialEq, Eq, Hash,
)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(export))]
/// The feature type for a post.
pub enum PostFeatureType {
  #[default]
  /// Features to the top of your site.
  Local,
  /// Features to the top of the community.
  Community,
}

#[derive(
  EnumString, Display, Debug, Serialize, Deserialize, Clone, Copy, Default, PartialEq, Eq, Hash,
)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(export))]
/// The like_type for a persons liked content.
pub enum LikeType {
  #[default]
  All,
  LikedOnly,
  DislikedOnly,
}

/// Wrapper for assert_eq! macro. Checks that vec matches the given length, and prints the
/// vec on failure.
#[macro_export]
macro_rules! assert_length {
  ($len:expr, $vec:expr) => {{ assert_eq!($len, $vec.len(), "Vec has wrong length: {:?}", $vec) }};
}

#[cfg(feature = "full")]
/// A helper tuple for person 1 alias columns
pub type Person1AliasAllColumnsTuple = (
  AliasedField<aliases::Person1, person::id>,
  AliasedField<aliases::Person1, person::name>,
  AliasedField<aliases::Person1, person::display_name>,
  AliasedField<aliases::Person1, person::avatar>,
  AliasedField<aliases::Person1, person::published_at>,
  AliasedField<aliases::Person1, person::updated_at>,
  AliasedField<aliases::Person1, person::ap_id>,
  AliasedField<aliases::Person1, person::bio>,
  AliasedField<aliases::Person1, person::local>,
  AliasedField<aliases::Person1, person::private_key>,
  AliasedField<aliases::Person1, person::public_key>,
  AliasedField<aliases::Person1, person::last_refreshed_at>,
  AliasedField<aliases::Person1, person::banner>,
  AliasedField<aliases::Person1, person::deleted>,
  AliasedField<aliases::Person1, person::inbox_url>,
  AliasedField<aliases::Person1, person::matrix_user_id>,
  AliasedField<aliases::Person1, person::bot_account>,
  AliasedField<aliases::Person1, person::instance_id>,
  AliasedField<aliases::Person1, person::post_count>,
  AliasedField<aliases::Person1, person::post_score>,
  AliasedField<aliases::Person1, person::comment_count>,
  AliasedField<aliases::Person1, person::comment_score>,
);

#[cfg(feature = "full")]
/// A helper tuple for person 2 alias columns
pub type Person2AliasAllColumnsTuple = (
  AliasedField<aliases::Person2, person::id>,
  AliasedField<aliases::Person2, person::name>,
  AliasedField<aliases::Person2, person::display_name>,
  AliasedField<aliases::Person2, person::avatar>,
  AliasedField<aliases::Person2, person::published_at>,
  AliasedField<aliases::Person2, person::updated_at>,
  AliasedField<aliases::Person2, person::ap_id>,
  AliasedField<aliases::Person2, person::bio>,
  AliasedField<aliases::Person2, person::local>,
  AliasedField<aliases::Person2, person::private_key>,
  AliasedField<aliases::Person2, person::public_key>,
  AliasedField<aliases::Person2, person::last_refreshed_at>,
  AliasedField<aliases::Person2, person::banner>,
  AliasedField<aliases::Person2, person::deleted>,
  AliasedField<aliases::Person2, person::inbox_url>,
  AliasedField<aliases::Person2, person::matrix_user_id>,
  AliasedField<aliases::Person2, person::bot_account>,
  AliasedField<aliases::Person2, person::instance_id>,
  AliasedField<aliases::Person2, person::post_count>,
  AliasedField<aliases::Person2, person::post_score>,
  AliasedField<aliases::Person2, person::comment_count>,
  AliasedField<aliases::Person2, person::comment_score>,
);

#[cfg(feature = "full")]
/// A helper tuple for more my instance persons actions
pub type MyInstancePersonsActionsAllColumnsTuple = (
  AliasedField<aliases::MyInstancePersonsActions, instance_actions::blocked_communities_at>,
  AliasedField<aliases::MyInstancePersonsActions, instance_actions::person_id>,
  AliasedField<aliases::MyInstancePersonsActions, instance_actions::instance_id>,
  AliasedField<aliases::MyInstancePersonsActions, instance_actions::received_ban_at>,
  AliasedField<aliases::MyInstancePersonsActions, instance_actions::ban_expires_at>,
  AliasedField<aliases::MyInstancePersonsActions, instance_actions::blocked_persons_at>,
);
