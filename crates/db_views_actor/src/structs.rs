use lemmy_db_schema::{
  aggregates::structs::{CommentAggregates, CommunityAggregates, PersonAggregates},
  source::{
    comment::Comment,
    comment_reply::CommentReply,
    community::CommunitySafe,
    person::{PersonSafe, PersonSafeAlias1},
    person_mention::PersonMention,
    post::Post,
  },
  SubscribedType,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CommunityBlockView {
  pub person: PersonSafe,
  pub community: CommunitySafe,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CommunityFollowerView {
  pub community: CommunitySafe,
  pub follower: PersonSafe,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CommunityModeratorView {
  pub community: CommunitySafe,
  pub moderator: PersonSafe,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CommunityPersonBanView {
  pub community: CommunitySafe,
  pub person: PersonSafe,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CommunityView {
  pub community: CommunitySafe,
  pub subscribed: SubscribedType,
  pub blocked: bool,
  pub counts: CommunityAggregates,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PersonBlockView {
  pub person: PersonSafe,
  pub target: PersonSafeAlias1,
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct PersonMentionView {
  pub person_mention: PersonMention,
  pub comment: Comment,
  pub creator: PersonSafe,
  pub post: Post,
  pub community: CommunitySafe,
  pub recipient: PersonSafeAlias1,
  pub counts: CommentAggregates,
  pub creator_banned_from_community: bool, // Left Join to CommunityPersonBan
  pub subscribed: SubscribedType,          // Left join to CommunityFollower
  pub saved: bool,                         // Left join to CommentSaved
  pub creator_blocked: bool,               // Left join to PersonBlock
  pub my_vote: Option<i16>,                // Left join to CommentLike
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct CommentReplyView {
  pub comment_reply: CommentReply,
  pub comment: Comment,
  pub creator: PersonSafe,
  pub post: Post,
  pub community: CommunitySafe,
  pub recipient: PersonSafeAlias1,
  pub counts: CommentAggregates,
  pub creator_banned_from_community: bool, // Left Join to CommunityPersonBan
  pub subscribed: SubscribedType,          // Left join to CommunityFollower
  pub saved: bool,                         // Left join to CommentSaved
  pub creator_blocked: bool,               // Left join to PersonBlock
  pub my_vote: Option<i16>,                // Left join to CommentLike
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PersonViewSafe {
  pub person: PersonSafe,
  pub counts: PersonAggregates,
}
