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

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CommunityBlockView {
  pub person: Person,
  pub community: Community,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CommunityFollowerView {
  pub community: Community,
  pub follower: Person,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CommunityModeratorView {
  pub community: Community,
  pub moderator: Person,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CommunityPersonBanView {
  pub community: Community,
  pub person: Person,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CommunityView {
  pub community: Community,
  pub subscribed: SubscribedType,
  pub blocked: bool,
  pub counts: CommunityAggregates,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PersonBlockView {
  pub person: Person,
  pub target: Person,
}

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize, Clone)]
pub struct PersonMentionView {
  pub person_mention: PersonMention,
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

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize, Clone)]
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
pub struct PersonView {
  pub person: Person,
  pub counts: PersonAggregates,
}
