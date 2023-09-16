#[cfg(feature = "full")]
use diesel::Queryable;
use lemmy_db_schema::{
  aggregates::structs::{CommentAggregates, CommunityAggregates, PersonAggregates},
  source::{
    comment::Comment,
    comment_reply::CommentReply,
    community::Community,
    person::Person,
    person_mention::PersonMention,
    post::Post,
  },
  SubscribedType,
};
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
#[cfg(feature = "full")]
use ts_rs::TS;

#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "full", derive(TS, Queryable))]
#[cfg_attr(feature = "full", ts(export))]
/// A community block.
pub struct CommunityBlockView {
  pub person: Person,
  pub community: Community,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "full", derive(TS, Queryable))]
#[cfg_attr(feature = "full", ts(export))]
/// A community follower.
pub struct CommunityFollowerView {
  pub community: Community,
  pub follower: Person,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "full", derive(TS, Queryable))]
#[cfg_attr(feature = "full", ts(export))]
/// A community moderator.
pub struct CommunityModeratorView {
  pub community: Community,
  pub moderator: Person,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "full", derive(Queryable))]
/// A community person ban.
pub struct CommunityPersonBanView {
  pub community: Community,
  pub person: Person,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "full", derive(TS, Queryable))]
#[cfg_attr(feature = "full", ts(export))]
/// A community view.
pub struct CommunityView {
  pub community: Community,
  pub subscribed: SubscribedType,
  pub blocked: bool,
  pub counts: CommunityAggregates,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "full", derive(TS, Queryable))]
#[cfg_attr(feature = "full", ts(export))]
/// A person block.
pub struct PersonBlockView {
  pub person: Person,
  pub target: Person,
}

#[skip_serializing_none]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "full", derive(TS, Queryable))]
#[cfg_attr(feature = "full", ts(export))]
/// A person mention view.
pub struct PersonMentionView {
  pub person_mention: PersonMention,
  pub comment: Comment,
  pub creator: Person,
  pub post: Post,
  pub community: Community,
  pub recipient: Person,
  pub counts: CommentAggregates,
  pub creator_banned_from_community: bool,
  pub subscribed: SubscribedType,
  pub saved: bool,
  pub creator_blocked: bool,
  pub my_vote: Option<i16>,
}

#[skip_serializing_none]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "full", derive(TS, Queryable))]
#[cfg_attr(feature = "full", ts(export))]
/// A comment reply view.
pub struct CommentReplyView {
  pub comment_reply: CommentReply,
  pub comment: Comment,
  pub creator: Person,
  pub post: Post,
  pub community: Community,
  pub recipient: Person,
  pub counts: CommentAggregates,
  pub creator_banned_from_community: bool, // Left Join to CommunityPersonBan
  pub subscribed: SubscribedType,          // Left join to CommunityFollower
  pub saved: bool,                         // Left join to CommentSaved
  pub creator_blocked: bool,               // Left join to PersonBlock
  pub my_vote: Option<i16>,                // Left join to CommentLike
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "full", derive(TS, Queryable))]
#[cfg_attr(feature = "full", ts(export))]
/// A person view.
pub struct PersonView {
  pub person: Person,
  pub counts: PersonAggregates,
}

#[skip_serializing_none]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "full", derive(TS, Queryable))]
#[cfg_attr(feature = "full", ts(export))]
/// Followed community posts view.
pub struct FollowedCommunityPostView {
  pub community: Community,
  pub pending: bool,
  pub notifications_enabled: bool,
  pub post: Post,
  pub post_read: bool,
}
