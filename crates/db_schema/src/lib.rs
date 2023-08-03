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

#[cfg(feature = "full")]
#[macro_use]
extern crate macro_rules_attribute;

#[cfg(feature = "full")]
/// `macro_rules_attribute::derive(WithoutId!)` generates a variant of the struct with
/// `WithoutId` added to the name and no `id` field.
///
/// This is useful for making less redundant selections of multiple joined tables.
/// For example, selecting both `comment::post_id` and `post::id` is redundant if they
/// have the same value. In this case, the selection of `post::id` can be avoided by selecting
/// `PostWithoutId::as_select()` instead of `post::all_columns`.
///
/// This macro generates an `into_full` method, which converts to the sturct with `id`.
/// For example, `PostWithoutId` would have this:
///
/// `pub fn into_full(self, id: PostId) -> Post`
///
/// The `id` value can come from a column in another table, like `comment::post_id`.
///
/// The generated struct implements `Selectable` and `Queryable`.
macro_rules! WithoutId {
  (
    #[diesel(table_name = $table_name:ident)]
    $(#[$_struct_meta:meta])*
    $vis:vis struct $struct_name:ident {
      $(#[$_id_meta:meta])*
      $_id_vis:vis id: $id_type:ty,
      $(
        // TODO: more flexible and clean attribute matching
        $(#[doc = $_doc1:tt])*
        $(#[cfg($($cfgtt:tt)*)])*
        $(#[cfg_attr($($cfgattrtt:tt)*)] $(#[doc = $_doc2:tt])*)*
        $(#[serde($($serdett:tt)*)])*
        $field_vis:vis $field_name:ident : $field_type:ty,
      )*
    }
  ) => {
    ::paste::paste! {
      #[derive(::diesel::Queryable, ::diesel::Selectable)]
      #[diesel(table_name = $table_name)]
      $vis struct [<$struct_name WithoutId>] {
        $(
          $(#[cfg($($cfgtt)*)])*
          $field_vis $field_name : $field_type,
        )*
      }

      impl [<$struct_name WithoutId>] {
        pub fn into_full(self, id: $id_type) -> $struct_name {
          $struct_name {
            id,
            $($(#[cfg($($cfgtt)*)])* $field_name : self.$field_name,)*
          }
        }
      }
    }
  };

  // Keep on removing the first attribute until `diesel(table_name = ...)` becomes
  // the first, which will cause the first pattern to be matched.
  (#[$_meta:meta] $($remaining:tt)*) => {
    WithoutId!($($remaining)*);
  };

  // This pattern is matched when there's no attributes.
  ($_vis:vis struct $($_tt:tt)*) => {
    ::std::compile_error!("`#[diesel(table_name = ...)]` is missing");
  };
}

pub mod aggregates;
#[cfg(feature = "full")]
pub mod impls;
pub mod newtypes;
#[cfg(feature = "full")]
#[rustfmt::skip]
#[allow(clippy::wildcard_imports)]
pub mod schema;
#[cfg(feature = "full")]
pub mod aliases {
  use crate::schema::person;
  diesel::alias!(person as person1: Person1, person as person2: Person2);
}
pub mod source;
#[cfg(feature = "full")]
pub mod traits;
#[cfg(feature = "full")]
pub mod utils;

use serde::{Deserialize, Serialize};
use strum_macros::{Display, EnumString};
#[cfg(feature = "full")]
use ts_rs::TS;

#[derive(EnumString, Display, Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "full", derive(DbEnum, TS))]
#[cfg_attr(
  feature = "full",
  ExistingTypePath = "crate::schema::sql_types::SortTypeEnum"
)]
#[cfg_attr(feature = "full", DbValueStyle = "verbatim")]
#[cfg_attr(feature = "full", ts(export))]
/// The post sort types. See here for descriptions: https://join-lemmy.org/docs/en/users/03-votes-and-ranking.html
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
  TopHour,
  TopSixHour,
  TopTwelveHour,
  TopThreeMonths,
  TopSixMonths,
  TopNineMonths,
  Controversial,
}

#[derive(EnumString, Display, Debug, Serialize, Deserialize, Clone, Copy)]
#[cfg_attr(feature = "full", derive(TS))]
#[cfg_attr(feature = "full", ts(export))]
/// The comment sort types. See here for descriptions: https://join-lemmy.org/docs/en/users/03-votes-and-ranking.html
pub enum CommentSortType {
  Hot,
  Top,
  New,
  Old,
  Controversial,
}

#[derive(EnumString, Display, Debug, Serialize, Deserialize, Clone, Copy)]
#[cfg_attr(feature = "full", derive(TS))]
#[cfg_attr(feature = "full", ts(export))]
/// The person sort types. See here for descriptions: https://join-lemmy.org/docs/en/users/03-votes-and-ranking.html
pub enum PersonSortType {
  New,
  Old,
  MostComments,
  CommentScore,
  PostScore,
  PostCount,
}

#[derive(EnumString, Display, Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq)]
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
  Local,
  /// Content only from communities you've subscribed to.
  Subscribed,
}

#[derive(EnumString, Display, Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq)]
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
  Open,
}

#[derive(EnumString, Display, Debug, Serialize, Deserialize, Clone, Copy)]
#[cfg_attr(feature = "full", derive(TS))]
#[cfg_attr(feature = "full", ts(export))]
/// The type of content returned from a search.
pub enum SearchType {
  All,
  Comments,
  Posts,
  Communities,
  Users,
  Url,
}

#[derive(EnumString, Display, Debug, PartialEq, Eq, Serialize, Deserialize, Clone, Copy)]
#[cfg_attr(feature = "full", derive(TS))]
#[cfg_attr(feature = "full", ts(export))]
/// A type / status for a community subscribe.
pub enum SubscribedType {
  Subscribed,
  NotSubscribed,
  Pending,
}

#[derive(EnumString, Display, Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq)]
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
}

#[derive(
  EnumString, Display, Debug, Serialize, Deserialize, Clone, Copy, Default, PartialEq, Eq,
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
