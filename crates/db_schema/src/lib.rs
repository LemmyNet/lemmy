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

// this is used in tests
#[cfg(feature = "full")]
#[macro_use]
extern crate diesel_migrations;

#[cfg(feature = "full")]
#[macro_use]
extern crate async_trait;

pub mod aggregates;
#[cfg(feature = "full")]
pub mod impls;
pub mod newtypes;
#[cfg(feature = "full")]
#[rustfmt::skip]
pub mod schema;
pub mod source;
#[cfg(feature = "full")]
pub mod traits;
#[cfg(feature = "full")]
pub mod utils;

use serde::{Deserialize, Serialize};
use strum_macros::{Display, EnumString};

#[derive(EnumString, Display, Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "full", derive(DbEnum))]
#[cfg_attr(
  feature = "full",
  ExistingTypePath = "crate::schema::sql_types::SortTypeEnum"
)]
#[cfg_attr(feature = "full", DbValueStyle = "verbatim")]
pub enum SortType {
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
}

#[derive(EnumString, Display, Debug, Serialize, Deserialize, Clone, Copy)]
pub enum CommentSortType {
  Hot,
  Top,
  New,
  Old,
}

#[derive(EnumString, Display, Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "full", derive(DbEnum))]
#[cfg_attr(
  feature = "full",
  ExistingTypePath = "crate::schema::sql_types::ListingTypeEnum"
)]
#[cfg_attr(feature = "full", DbValueStyle = "verbatim")]
pub enum ListingType {
  All,
  Local,
  Subscribed,
}

#[derive(EnumString, Display, Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "full", derive(DbEnum))]
#[cfg_attr(
  feature = "full",
  ExistingTypePath = "crate::schema::sql_types::RegistrationModeEnum"
)]
#[cfg_attr(feature = "full", DbValueStyle = "verbatim")]
pub enum RegistrationMode {
  Closed,
  RequireApplication,
  Open,
}

#[derive(EnumString, Display, Debug, Serialize, Deserialize, Clone, Copy)]
pub enum SearchType {
  All,
  Comments,
  Posts,
  Communities,
  Users,
  Url,
}

#[derive(EnumString, Display, Debug, PartialEq, Eq, Serialize, Deserialize, Clone, Copy)]
pub enum SubscribedType {
  Subscribed,
  NotSubscribed,
  Pending,
}

#[derive(EnumString, Display, Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq)]
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
}

#[derive(
  EnumString, Display, Debug, Serialize, Deserialize, Clone, Copy, Default, PartialEq, Eq,
)]
pub enum PostFeatureType {
  #[default]
  Local,
  Community,
}
