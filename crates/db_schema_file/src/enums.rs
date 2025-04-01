use diesel_derive_enum::DbEnum;
use serde::{Deserialize, Serialize};
use strum::{Display, EnumString};
use ts_rs::TS;

#[derive(
  EnumString,
  Display,
  Debug,
  Serialize,
  Deserialize,
  Clone,
  Copy,
  PartialEq,
  Eq,
  Default,
  Hash,
  DbEnum,
  TS,
)]
#[ExistingTypePath = "crate::schema::sql_types::PostSortTypeEnum"]
#[DbValueStyle = "verbatim"]
#[ts(export)]
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
  EnumString,
  Display,
  Debug,
  Serialize,
  Deserialize,
  Clone,
  Copy,
  PartialEq,
  Eq,
  Default,
  Hash,
  DbEnum,
  TS,
)]
#[ExistingTypePath = "crate::schema::sql_types::CommentSortTypeEnum"]
#[DbValueStyle = "verbatim"]
#[ts(export)]
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
  EnumString, Display, Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Default, Hash, TS,
)]
#[ts(export)]
/// The search sort types.
pub enum SearchSortType {
  #[default]
  New,
  Top,
  Old,
}

#[derive(
  EnumString,
  Display,
  Debug,
  Serialize,
  Deserialize,
  Clone,
  Copy,
  PartialEq,
  Eq,
  Default,
  Hash,
  DbEnum,
  TS,
)]
#[ExistingTypePath = "crate::schema::sql_types::ListingTypeEnum"]
#[DbValueStyle = "verbatim"]
#[ts(export)]
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
  EnumString,
  Display,
  Debug,
  Serialize,
  Deserialize,
  Clone,
  Copy,
  PartialEq,
  Eq,
  Default,
  Hash,
  DbEnum,
  TS,
)]
#[ExistingTypePath = "crate::schema::sql_types::RegistrationModeEnum"]
#[DbValueStyle = "verbatim"]
#[ts(export)]
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
  EnumString,
  Display,
  Debug,
  Serialize,
  Deserialize,
  Default,
  Clone,
  Copy,
  PartialEq,
  Eq,
  Hash,
  DbEnum,
  TS,
)]
#[ExistingTypePath = "crate::schema::sql_types::PostListingModeEnum"]
#[DbValueStyle = "verbatim"]
#[ts(export)]
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
  EnumString,
  Display,
  Debug,
  Serialize,
  Deserialize,
  Clone,
  Copy,
  PartialEq,
  Eq,
  Default,
  Hash,
  DbEnum,
  TS,
)]
#[ExistingTypePath = "crate::schema::sql_types::CommunityVisibility"]
#[DbValueStyle = "verbatim"]
#[ts(export)]
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
  EnumString,
  Display,
  Debug,
  Serialize,
  Deserialize,
  Clone,
  Copy,
  PartialEq,
  Eq,
  Default,
  Hash,
  DbEnum,
  TS,
)]
#[ExistingTypePath = "crate::schema::sql_types::FederationModeEnum"]
#[DbValueStyle = "verbatim"]
#[ts(export)]
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

#[derive(Clone, Copy, Debug, DbEnum, PartialEq, Eq)]
#[ExistingTypePath = "crate::schema::sql_types::ActorTypeEnum"]
pub enum ActorType {
  Site,
  Community,
  Person,
}

#[derive(
  EnumString, Display, Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Hash, DbEnum, TS,
)]
#[ExistingTypePath = "crate::schema::sql_types::CommunityFollowerState"]
#[DbValueStyle = "verbatim"]
#[ts(export)]
pub enum CommunityFollowerState {
  Accepted,
  Pending,
  ApprovalRequired,
}
