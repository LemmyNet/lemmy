use crate::newtypes::LocalUserId;
#[cfg(feature = "full")]
use crate::schema::local_user_vote_display_mode;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
#[cfg(feature = "full")]
use ts_rs::TS;
use typed_builder::TypedBuilder;

#[skip_serializing_none]
#[derive(PartialEq, Eq, Debug, Clone, Default, Serialize, Deserialize)]
#[cfg_attr(feature = "full", derive(Queryable, Selectable, Identifiable, TS))]
#[cfg_attr(feature = "full", diesel(table_name = local_user_vote_display_mode))]
#[cfg_attr(feature = "full", diesel(primary_key(local_user_id)))]
#[cfg_attr(
  feature = "full",
  diesel(belongs_to(crate::source::local_site::LocalUser))
)]
#[cfg_attr(feature = "full", diesel(check_for_backend(diesel::pg::Pg)))]
#[cfg_attr(feature = "full", ts(export))]
/// The vote display settings for your user.
pub struct LocalUserVoteDisplayMode {
  #[serde(skip)]
  pub local_user_id: LocalUserId,
  pub score: bool,
  pub upvotes: bool,
  pub downvotes: bool,
  pub upvote_percentage: bool,
}

#[derive(Clone, TypedBuilder)]
#[builder(field_defaults(default))]
#[cfg_attr(feature = "full", derive(Insertable))]
#[cfg_attr(feature = "full", diesel(table_name = local_user_vote_display_mode))]
pub struct LocalUserVoteDisplayModeInsertForm {
  #[builder(!default)]
  pub local_user_id: LocalUserId,
  pub score: Option<bool>,
  pub upvotes: Option<bool>,
  pub downvotes: Option<bool>,
  pub upvote_percentage: Option<bool>,
}

#[derive(Clone, Default)]
#[cfg_attr(feature = "full", derive(AsChangeset))]
#[cfg_attr(feature = "full", diesel(table_name = local_user_vote_display_mode))]
pub struct LocalUserVoteDisplayModeUpdateForm {
  pub score: Option<bool>,
  pub upvotes: Option<bool>,
  pub downvotes: Option<bool>,
  pub upvote_percentage: Option<bool>,
}
