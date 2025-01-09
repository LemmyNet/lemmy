#![recursion_limit = "256"]

#[cfg(feature = "full")]
#[macro_use]
extern crate diesel;
#[cfg(feature = "full")]
#[macro_use]
extern crate diesel_derive_newtype;

#[cfg(feature = "full")]
#[macro_use]
extern crate diesel_derive_enum;

#[cfg(feature = "full")]
#[macro_use]
extern crate async_trait;

pub mod aggregates;
#[cfg(feature = "full")]
pub mod impls;
pub mod newtypes;
pub mod sensitive;
#[cfg(feature = "full")]
#[rustfmt::skip]
pub mod schema;
#[cfg(feature = "full")]
pub mod aliases {
  use crate::schema::{community_actions, person};
  diesel::alias!(
    community_actions as creator_community_actions: CreatorCommunityActions,
    person as person1: Person1,
    person as person2: Person2,
  );
}
pub mod source;
#[cfg(feature = "full")]
pub mod traits;
#[cfg(feature = "full")]
pub mod utils;

#[cfg(feature = "full")]
pub mod schema_setup;

use serde::{Deserialize, Serialize};
use strum::{Display, EnumString};
#[cfg(feature = "full")]
use ts_rs::TS;

#[derive(
  EnumString, Display, Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Default, Hash,
)]
#[cfg_attr(feature = "full", derive(DbEnum, TS))]
#[cfg_attr(
  feature = "full",
  ExistingTypePath = "crate::schema::sql_types::PostSortTypeEnum"
)]
#[cfg_attr(feature = "full", DbValueStyle = "verbatim")]
#[cfg_attr(feature = "full", ts(export))]
// TODO add the controversial and scaled rankings to the doc below
/// The post sort types. See here for descriptions: https://join-lemmy.org/docs/en/users/03-votes-and-ranking.html
pub enum PostSortType {
  #[default]
  Active,
  Hot,
  New,
  Old,
  TopDay,
  TopWeek,
  TopMonth,
  TopYear,
  TopAll,
  MostComments,
  NewComments,
  TopHour,
  TopSixHour,
  TopTwelveHour,
  TopThreeMonths,
  TopSixMonths,
  TopNineMonths,
  Controversial,
  Scaled,
}

#[derive(
  EnumString, Display, Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Default, Hash,
)]
#[cfg_attr(feature = "full", derive(DbEnum, TS))]
#[cfg_attr(
  feature = "full",
  ExistingTypePath = "crate::schema::sql_types::CommentSortTypeEnum"
)]
#[cfg_attr(feature = "full", DbValueStyle = "verbatim")]
#[cfg_attr(feature = "full", ts(export))]
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
#[cfg_attr(feature = "full", derive(DbEnum, TS))]
#[cfg_attr(
  feature = "full",
  ExistingTypePath = "crate::schema::sql_types::ListingTypeEnum"
)]
#[cfg_attr(feature = "full", DbValueStyle = "verbatim")]
#[cfg_attr(feature = "full", ts(export))]
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
}

#[derive(
  EnumString, Display, Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Default, Hash,
)]
#[cfg_attr(feature = "full", derive(DbEnum, TS))]
#[cfg_attr(
  feature = "full",
  ExistingTypePath = "crate::schema::sql_types::RegistrationModeEnum"
)]
#[cfg_attr(feature = "full", DbValueStyle = "verbatim")]
#[cfg_attr(feature = "full", ts(export))]
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
#[cfg_attr(feature = "full", derive(DbEnum, TS))]
#[cfg_attr(
  feature = "full",
  ExistingTypePath = "crate::schema::sql_types::PostListingModeEnum"
)]
#[cfg_attr(feature = "full", DbValueStyle = "verbatim")]
#[cfg_attr(feature = "full", ts(export))]
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

#[derive(EnumString, Display, Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "full", derive(TS))]
#[cfg_attr(feature = "full", ts(export))]
/// The type of content returned from a search.
pub enum SearchType {
  All,
  Comments,
  Posts,
  Communities,
  Users,
}

#[derive(EnumString, Display, Debug, PartialEq, Eq, Serialize, Deserialize, Clone, Copy, Hash)]
#[cfg_attr(feature = "full", derive(TS))]
#[cfg_attr(feature = "full", ts(export))]
/// A type / status for a community subscribe.
pub enum SubscribedType {
  Subscribed,
  NotSubscribed,
  Pending,
  ApprovalRequired,
}

#[derive(EnumString, Display, Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "full", derive(TS))]
#[cfg_attr(feature = "full", ts(export))]
/// A list of possible types for the various modlog actions.
pub enum ModlogActionType {
  All,
  ModRemovePost,
  ModLockPost,
  ModFeaturePost,
  ModRemoveComment,
  ModRemoveCommunity,
  ModBanFromCommunity,
  ModAddCommunity,
  ModTransferCommunity,
  ModAdd,
  ModBan,
  ModHideCommunity,
  AdminPurgePerson,
  AdminPurgeCommunity,
  AdminPurgePost,
  AdminPurgeComment,
  AdminBlockInstance,
  AdminAllowInstance,
}

#[derive(EnumString, Display, Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "full", derive(TS))]
#[cfg_attr(feature = "full", ts(export))]
/// A list of possible types for the various modlog actions.
pub enum PersonContentType {
  All,
  Comments,
  Posts,
}

#[derive(
  EnumString, Display, Debug, Serialize, Deserialize, Clone, Copy, Default, PartialEq, Eq, Hash,
)]
#[cfg_attr(feature = "full", derive(TS))]
#[cfg_attr(feature = "full", ts(export))]
/// The feature type for a post.
pub enum PostFeatureType {
  #[default]
  /// Features to the top of your site.
  Local,
  /// Features to the top of the community.
  Community,
}

#[derive(
  EnumString, Display, Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Default, Hash,
)]
#[cfg_attr(feature = "full", derive(DbEnum, TS))]
#[cfg_attr(
  feature = "full",
  ExistingTypePath = "crate::schema::sql_types::CommunityVisibility"
)]
#[cfg_attr(feature = "full", DbValueStyle = "verbatim")]
#[cfg_attr(feature = "full", ts(export))]
/// Defines who can browse and interact with content in a community.
pub enum CommunityVisibility {
  /// Public community, any local or federated user can interact.
  #[default]
  Public,
  /// Unfederated community, only local users can interact.
  LocalOnly,
  /// Users need to be approved by mods before they are able to browse or post.
  Private,
}

#[derive(
  EnumString, Display, Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Default, Hash,
)]
#[cfg_attr(feature = "full", derive(DbEnum, TS))]
#[cfg_attr(
  feature = "full",
  ExistingTypePath = "crate::schema::sql_types::FederationModeEnum"
)]
#[cfg_attr(feature = "full", DbValueStyle = "verbatim")]
#[cfg_attr(feature = "full", ts(export))]
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

/// Wrapper for assert_eq! macro. Checks that vec matches the given length, and prints the
/// vec on failure.
#[macro_export]
macro_rules! assert_length {
  ($len:expr, $vec:expr) => {{
    assert_eq!($len, $vec.len(), "Vec has wrong length: {:?}", $vec)
  }};
}
