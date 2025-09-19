#[cfg(feature = "full")]
use diesel::Queryable;
use lemmy_db_schema::source::person::Person;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

#[cfg(feature = "full")]
pub mod impls;

#[skip_serializing_none]
#[derive(Debug, PartialEq, Eq, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "full", derive(Queryable))]
#[cfg_attr(feature = "full", diesel(check_for_backend(diesel::pg::Pg)))]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
/// A vote view for checking a post or comments votes.
pub struct VoteView {
  pub creator: Person,
  pub creator_banned: bool,
  pub creator_banned_from_community: bool,
  pub is_upvote: bool,
}
