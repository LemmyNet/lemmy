#[cfg(feature = "full")]
use diesel::Queryable;
use lemmy_db_schema::{
  aggregates::structs::{CommentAggregates, CommunityAggregates, PersonAggregates, PostAggregates},
  source::{
    comment::Comment,
    comment_reply::CommentReply,
    community::Community,
    images::ImageDetails,
    person::Person,
    person_comment_mention::PersonCommentMention,
    person_post_mention::PersonPostMention,
    post::Post,
    private_message::PrivateMessage,
  },
  SubscribedType,
};
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
#[cfg(feature = "full")]
use ts_rs::TS;

#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "full", derive(TS, Queryable))]
#[cfg_attr(feature = "full", diesel(check_for_backend(diesel::pg::Pg)))]
#[cfg_attr(feature = "full", ts(export))]
/// A community follower.
pub struct CommunityFollowerView {
  pub community: Community,
  pub follower: Person,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "full", derive(TS, Queryable))]
#[cfg_attr(feature = "full", diesel(check_for_backend(diesel::pg::Pg)))]
#[cfg_attr(feature = "full", ts(export))]
/// A community moderator.
pub struct CommunityModeratorView {
  pub community: Community,
  pub moderator: Person,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "full", derive(Queryable))]
#[cfg_attr(feature = "full", diesel(check_for_backend(diesel::pg::Pg)))]
/// A community person ban.
pub struct CommunityPersonBanView {
  pub community: Community,
  pub person: Person,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "full", derive(TS, Queryable))]
#[cfg_attr(feature = "full", diesel(check_for_backend(diesel::pg::Pg)))]
#[cfg_attr(feature = "full", ts(export))]
/// A community view.
pub struct CommunityView {
  pub community: Community,
  pub subscribed: SubscribedType,
  pub blocked: bool,
  pub counts: CommunityAggregates,
  pub banned_from_community: bool,
}

/// The community sort types. See here for descriptions: https://join-lemmy.org/docs/en/users/03-votes-and-ranking.html
#[derive(Debug, Serialize, Deserialize, Clone, Copy, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "full", derive(TS))]
#[cfg_attr(feature = "full", ts(export))]
pub enum CommunitySortType {
  #[default]
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
  Scaled,
  NameAsc,
  NameDesc,
}

#[skip_serializing_none]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "full", derive(TS, Queryable))]
#[cfg_attr(feature = "full", diesel(check_for_backend(diesel::pg::Pg)))]
#[cfg_attr(feature = "full", ts(export))]
/// A person comment mention view.
pub struct PersonCommentMentionView {
  pub person_comment_mention: PersonCommentMention,
  pub comment: Comment,
  pub creator: Person,
  pub post: Post,
  pub community: Community,
  pub recipient: Person,
  pub counts: CommentAggregates,
  pub creator_banned_from_community: bool,
  pub banned_from_community: bool,
  pub creator_is_moderator: bool,
  pub creator_is_admin: bool,
  pub subscribed: SubscribedType,
  pub saved: bool,
  pub creator_blocked: bool,
  #[cfg_attr(feature = "full", ts(optional))]
  pub my_vote: Option<i16>,
}

#[skip_serializing_none]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "full", derive(TS, Queryable))]
#[cfg_attr(feature = "full", diesel(check_for_backend(diesel::pg::Pg)))]
#[cfg_attr(feature = "full", ts(export))]
/// A person post mention view.
pub struct PersonPostMentionView {
  pub person_post_mention: PersonPostMention,
  pub post: Post,
  pub creator: Person,
  pub community: Community,
  #[cfg_attr(feature = "full", ts(optional))]
  pub image_details: Option<ImageDetails>,
  pub recipient: Person,
  pub counts: PostAggregates,
  pub creator_banned_from_community: bool,
  pub banned_from_community: bool,
  pub creator_is_moderator: bool,
  pub creator_is_admin: bool,
  pub subscribed: SubscribedType,
  pub saved: bool,
  pub read: bool,
  pub hidden: bool,
  pub creator_blocked: bool,
  #[cfg_attr(feature = "full", ts(optional))]
  pub my_vote: Option<i16>,
  pub unread_comments: i64,
}

#[skip_serializing_none]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "full", derive(TS, Queryable))]
#[cfg_attr(feature = "full", diesel(check_for_backend(diesel::pg::Pg)))]
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
  pub creator_banned_from_community: bool,
  pub banned_from_community: bool,
  pub creator_is_moderator: bool,
  pub creator_is_admin: bool,
  pub subscribed: SubscribedType,
  pub saved: bool,
  pub creator_blocked: bool,
  #[cfg_attr(feature = "full", ts(optional))]
  pub my_vote: Option<i16>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "full", derive(TS, Queryable))]
#[cfg_attr(feature = "full", diesel(check_for_backend(diesel::pg::Pg)))]
#[cfg_attr(feature = "full", ts(export))]
/// A person view.
pub struct PersonView {
  pub person: Person,
  pub counts: PersonAggregates,
  pub is_admin: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "full", derive(TS, Queryable))]
#[cfg_attr(feature = "full", diesel(check_for_backend(diesel::pg::Pg)))]
#[cfg_attr(feature = "full", ts(export))]
pub struct PendingFollow {
  pub person: Person,
  pub community: Community,
  pub is_new_instance: bool,
  pub subscribed: SubscribedType,
}

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "full", derive(TS, Queryable))]
#[cfg_attr(feature = "full", diesel(check_for_backend(diesel::pg::Pg)))]
#[cfg_attr(feature = "full", ts(export))]
/// A private message view.
pub struct PrivateMessageView {
  pub private_message: PrivateMessage,
  pub creator: Person,
  pub recipient: Person,
}

/// like PaginationCursor but for the report_combined table
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "full", derive(TS))]
#[cfg_attr(feature = "full", ts(export))]
pub struct InboxCombinedPaginationCursor(pub String);

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "full", derive(Queryable))]
#[cfg_attr(feature = "full", diesel(check_for_backend(diesel::pg::Pg)))]
/// A combined inbox view
pub struct InboxCombinedViewInternal {
  // Comment reply
  pub comment_reply: Option<CommentReply>,
  // Person comment mention
  pub person_comment_mention: Option<PersonCommentMention>,
  // Person post mention
  pub person_post_mention: Option<PersonPostMention>,
  pub post_counts: Option<PostAggregates>,
  pub post_unread_comments: Option<i64>,
  pub post_saved: bool,
  pub post_read: bool,
  pub post_hidden: bool,
  pub my_post_vote: Option<i16>,
  pub image_details: Option<ImageDetails>,
  // Private message
  pub private_message: Option<PrivateMessage>,
  // Shared
  pub post: Option<Post>,
  pub community: Option<Community>,
  pub comment: Option<Comment>,
  pub comment_counts: Option<CommentAggregates>,
  pub comment_saved: bool,
  pub my_comment_vote: Option<i16>,
  pub subscribed: SubscribedType,
  pub item_creator: Person,
  pub item_recipient: Person,
  pub item_creator_is_admin: bool,
  pub item_creator_is_moderator: bool,
  pub item_creator_banned_from_community: bool,
  pub item_creator_blocked: bool,
  pub banned_from_community: bool,
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "full", derive(TS))]
#[cfg_attr(feature = "full", ts(export))]
// Use serde's internal tagging, to work easier with javascript libraries
#[serde(tag = "type_")]
pub enum InboxCombinedView {
  CommentReply(CommentReplyView),
  CommentMention(PersonCommentMentionView),
  PostMention(PersonPostMentionView),
  PrivateMessage(PrivateMessageView),
}
