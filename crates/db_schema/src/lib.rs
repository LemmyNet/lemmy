#[cfg(feature = "full")]
#[macro_use]
extern crate diesel;
#[cfg(feature = "full")]
#[macro_use]
extern crate diesel_derive_newtype;
// this is used in tests
#[cfg(feature = "full")]
#[allow(unused_imports)]
#[macro_use]
extern crate diesel_migrations;

pub mod aggregates;
#[cfg(feature = "full")]
pub mod impls;
pub mod newtypes;
#[cfg(feature = "full")]
pub mod schema;
pub mod source;
#[cfg(feature = "full")]
pub mod traits;
#[cfg(feature = "full")]
pub mod utils;

use serde::{Deserialize, Serialize};
use strum_macros::{Display, EnumString};

#[derive(EnumString, Display, Debug, Serialize, Deserialize, Clone, Copy)]
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

#[derive(EnumString, Display, Debug, Serialize, Deserialize, Clone, Copy, PartialEq)]
pub enum ListingType {
  All,
  Local,
  Subscribed,
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

#[derive(EnumString, Display, Debug, PartialEq, Serialize, Deserialize, Clone, Copy)]
pub enum SubscribedType {
  Subscribed,
  NotSubscribed,
  Pending,
}
