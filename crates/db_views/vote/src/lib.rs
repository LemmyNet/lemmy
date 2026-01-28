use lemmy_db_schema::{
  newtypes::{CommentId, PostId},
  source::person::Person,
};
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
#[cfg(feature = "full")]
use {
  diesel::{ExpressionMethods, NullableExpressionMethods, Queryable, Selectable},
  lemmy_db_schema::utils::queries::selects::creator_local_home_banned,
  lemmy_db_schema_file::{
    aliases::creator_community_actions,
    schema::{comment, comment_actions, community_actions, post, post_actions},
  },
};

#[cfg(feature = "full")]
pub mod impls;

/// Only used internally so no ts(export)
#[derive(Serialize, Deserialize)]
#[cfg_attr(feature = "full", derive(Queryable, Selectable))]
#[cfg_attr(feature = "full", diesel(check_for_backend(diesel::pg::Pg)))]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
struct VoteViewPost {
  #[cfg_attr(feature = "full", diesel(embed))]
  pub creator: Person,
  #[cfg_attr(feature = "full", diesel(select_expression = creator_local_home_banned()))]
  pub creator_banned: bool,
  #[cfg_attr(feature = "full", diesel(select_expression = creator_community_actions
          .field(community_actions::received_ban_at)
          .nullable()
          .is_not_null()))]
  pub creator_banned_from_community: bool,
  #[cfg_attr(feature = "full", diesel(select_expression = post_actions::vote_is_upvote.assume_not_null()))]
  pub is_upvote: bool,
  #[cfg_attr(feature = "full", diesel(select_expression = post::id))]
  post_id: PostId,
}

/// Only used internally so no ts(export)
#[derive(Serialize, Deserialize)]
#[cfg_attr(feature = "full", derive(Queryable, Selectable))]
#[cfg_attr(feature = "full", diesel(check_for_backend(diesel::pg::Pg)))]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
struct VoteViewComment {
  #[cfg_attr(feature = "full", diesel(embed))]
  pub creator: Person,
  #[cfg_attr(feature = "full", diesel(select_expression = creator_local_home_banned()))]
  pub creator_banned: bool,
  #[cfg_attr(feature = "full", diesel(select_expression = creator_community_actions
          .field(community_actions::received_ban_at)
          .nullable()
          .is_not_null()))]
  pub creator_banned_from_community: bool,
  #[cfg_attr(feature = "full", diesel(select_expression = comment_actions::vote_is_upvote.assume_not_null()))]
  pub is_upvote: bool,
  #[cfg_attr(feature = "full", diesel(select_expression = comment::id))]
  comment_id: CommentId,
}

#[skip_serializing_none]
#[derive(Debug, PartialEq, Eq, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
/// A vote view for checking a post or comments votes.
pub struct VoteView {
  pub creator: Person,
  pub creator_banned: bool,
  pub creator_banned_from_community: bool,
  /// True means Upvote, False means Downvote.
  pub is_upvote: bool,
}

impl From<VoteViewComment> for VoteView {
  fn from(v: VoteViewComment) -> Self {
    VoteView {
      creator: v.creator,
      creator_banned: v.creator_banned,
      creator_banned_from_community: v.creator_banned_from_community,
      is_upvote: v.is_upvote,
    }
  }
}

impl From<VoteViewPost> for VoteView {
  fn from(v: VoteViewPost) -> Self {
    VoteView {
      creator: v.creator,
      creator_banned: v.creator_banned,
      creator_banned_from_community: v.creator_banned_from_community,
      is_upvote: v.is_upvote,
    }
  }
}
