#[cfg(feature = "full")]
use diesel_derive_enum::DbEnum;
use serde::{Deserialize, Serialize};
use strum::{Display, EnumString};

#[derive(
  EnumString, Display, Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Default, Hash,
)]
#[cfg_attr(feature = "full", derive(DbEnum))]
#[cfg_attr(
  feature = "full",
  ExistingTypePath = "crate::schema::sql_types::PostSortTypeEnum"
)]
#[cfg_attr(feature = "full", DbValueStyle = "verbatim")]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(export))]
// TODO add the controversial and scaled rankings to the doc below
/// The post sort types. See here for descriptions: https://join-lemmy.org/docs/en/users/03-votes-and-ranking.html
pub enum PostSortType {
  #[default]
  Active,
  Hot,
  New,
  Old,
  Top,
  MostComments,
  NewComments,
  Controversial,
  Scaled,
}

#[derive(
  EnumString, Display, Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Default, Hash,
)]
#[cfg_attr(feature = "full", derive(DbEnum))]
#[cfg_attr(
  feature = "full",
  ExistingTypePath = "crate::schema::sql_types::CommentSortTypeEnum"
)]
#[cfg_attr(feature = "full", DbValueStyle = "verbatim")]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(export))]
/// The comment sort types. See here for descriptions: https://join-lemmy.org/docs/en/users/03-votes-and-ranking.html
pub enum CommentSortType {
  #[default]
  Hot,
  Top,
  New,
  Old,
  Controversial,
}

#[derive(
  EnumString, Display, Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Default, Hash,
)]
#[cfg_attr(feature = "full", derive(DbEnum))]
#[cfg_attr(
  feature = "full",
  ExistingTypePath = "crate::schema::sql_types::ListingTypeEnum"
)]
#[cfg_attr(feature = "full", DbValueStyle = "verbatim")]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(export))]
/// A listing type for post and comment list fetches.
pub enum ListingType {
  /// Content from your own site, as well as all connected / federated sites.
  All,
  /// Content from your site only.
  #[default]
  Local,
  /// Content only from communities you've subscribed to.
  Subscribed,
  /// Content that you can moderate (because you are a moderator of the community it is posted to)
  ModeratorView,
  /// Communities which are recommended by local instance admins
  Suggested,
}

#[derive(
  EnumString, Display, Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Default, Hash,
)]
#[cfg_attr(feature = "full", derive(DbEnum))]
#[cfg_attr(
  feature = "full",
  ExistingTypePath = "crate::schema::sql_types::RegistrationModeEnum"
)]
#[cfg_attr(feature = "full", DbValueStyle = "verbatim")]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(export))]
/// The registration mode for your site. Determines what happens after a user signs up.
pub enum RegistrationMode {
  /// Closed to public.
  Closed,
  /// Open, but pending approval of a registration application.
  RequireApplication,
  /// Open to all.
  #[default]
  Open,
}

#[derive(
  EnumString, Display, Debug, Serialize, Deserialize, Default, Clone, Copy, PartialEq, Eq, Hash,
)]
#[cfg_attr(feature = "full", derive(DbEnum))]
#[cfg_attr(
  feature = "full",
  ExistingTypePath = "crate::schema::sql_types::PostListingModeEnum"
)]
#[cfg_attr(feature = "full", DbValueStyle = "verbatim")]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(export))]
/// A post-view mode that changes how multiple post listings look.
pub enum PostListingMode {
  /// A compact, list-type view.
  #[default]
  List,
  /// A larger card-type view.
  Card,
  /// A smaller card-type view, usually with images as thumbnails
  SmallCard,
}

#[derive(
  EnumString, Display, Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Default, Hash,
)]
#[cfg_attr(feature = "full", derive(DbEnum))]
#[cfg_attr(
  feature = "full",
  ExistingTypePath = "crate::schema::sql_types::CommunityVisibility"
)]
#[cfg_attr(feature = "full", DbValueStyle = "verbatim")]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(export))]
/// Defines who can browse and interact with content in a community.
pub enum CommunityVisibility {
  /// Public community, any local or federated user can interact.
  #[default]
  Public,
  /// Community is unlisted/hidden and doesn't appear in community list. Posts from the community
  /// are not shown in Local and All feeds, except for subscribed users.
  Unlisted,
  /// Unfederated community, only local users can interact (with or without login).
  LocalOnlyPublic,
  /// Unfederated  community, only logged-in local users can interact.
  LocalOnlyPrivate,
  /// Users need to be approved by mods before they are able to browse or post.
  Private,
}

impl CommunityVisibility {
  pub fn can_federate(&self) -> bool {
    use CommunityVisibility::*;
    self != &LocalOnlyPublic && self != &LocalOnlyPrivate
  }
  pub fn can_view_without_login(&self) -> bool {
    use CommunityVisibility::*;
    self == &Public || self == &LocalOnlyPublic
  }
}

#[derive(
  EnumString, Display, Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Default, Hash,
)]
#[cfg_attr(feature = "full", derive(DbEnum))]
#[cfg_attr(
  feature = "full",
  ExistingTypePath = "crate::schema::sql_types::FederationModeEnum"
)]
#[cfg_attr(feature = "full", DbValueStyle = "verbatim")]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(export))]
/// The federation mode for an item
pub enum FederationMode {
  #[default]
  /// Allows all
  All,
  /// Allows only local
  Local,
  /// Disables
  Disable,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "full", derive(DbEnum))]
#[cfg_attr(
  feature = "full",
  ExistingTypePath = "crate::schema::sql_types::ActorTypeEnum"
)]
pub enum ActorType {
  Site,
  Community,
  Person,
  MultiCommunity,
}

#[derive(EnumString, Display, Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "full", derive(DbEnum))]
#[cfg_attr(
  feature = "full",
  ExistingTypePath = "crate::schema::sql_types::CommunityFollowerState"
)]
#[cfg_attr(feature = "full", DbValueStyle = "verbatim")]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(export))]
pub enum CommunityFollowerState {
  Accepted,
  Pending,
  ApprovalRequired,
}

#[derive(
  EnumString, Display, Debug, Serialize, Deserialize, Default, Clone, Copy, PartialEq, Eq, Hash,
)]
#[cfg_attr(feature = "full", derive(DbEnum))]
#[cfg_attr(
  feature = "full",
  ExistingTypePath = "crate::schema::sql_types::VoteShowEnum"
)]
#[cfg_attr(feature = "full", DbValueStyle = "verbatim")]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(export))]
/// Lets you show votes for others only, show all votes, or hide all votes.
pub enum VoteShow {
  #[default]
  Show,
  ShowForOthers,
  Hide,
}

#[derive(
  EnumString, Display, Debug, Serialize, Deserialize, Default, Clone, Copy, PartialEq, Eq, Hash,
)]
#[cfg_attr(feature = "full", derive(DbEnum))]
#[cfg_attr(
  feature = "full",
  ExistingTypePath = "crate::schema::sql_types::PostNotificationsModeEnum"
)]
#[cfg_attr(feature = "full", DbValueStyle = "verbatim")]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(export))]
/// Available settings for post notifications
pub enum PostNotificationsMode {
  AllComments,
  #[default]
  RepliesAndMentions,
  Mute,
}

#[derive(
  EnumString, Display, Debug, Serialize, Deserialize, Default, Clone, Copy, PartialEq, Eq, Hash,
)]
#[cfg_attr(feature = "full", derive(DbEnum))]
#[cfg_attr(
  feature = "full",
  ExistingTypePath = "crate::schema::sql_types::CommunityNotificationsModeEnum"
)]
#[cfg_attr(feature = "full", DbValueStyle = "verbatim")]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(export))]
/// Available settings for community notifications
pub enum CommunityNotificationsMode {
  AllPostsAndComments,
  AllPosts,
  #[default]
  RepliesAndMentions,
  Mute,
}

#[derive(EnumString, Display, Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "full", derive(DbEnum))]
#[cfg_attr(
  feature = "full",
  ExistingTypePath = "crate::schema::sql_types::NotificationTypeEnum"
)]
#[cfg_attr(feature = "full", DbValueStyle = "verbatim")]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(export))]
/// Types of notifications which can be received in inbox
pub enum NotificationType {
  Mention,
  Reply,
  Subscribed,
  PrivateMessage,
  ModAction,
}
