use lemmy_db_schema::{
  newtypes::PaginationCursor,
  source::{
    combined::inbox::InboxCombined,
    comment::{Comment, CommentActions},
    comment_reply::CommentReply,
    community::{Community, CommunityActions},
    images::ImageDetails,
    instance::InstanceActions,
    person::{Person, PersonActions},
    person_comment_mention::PersonCommentMention,
    person_post_mention::PersonPostMention,
    post::{Post, PostActions},
    private_message::PrivateMessage,
    tag::TagsView,
  },
  InboxDataType,
};
use lemmy_db_views_private_message::PrivateMessageView;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
#[cfg(feature = "full")]
use {
  diesel::{Queryable, Selectable},
  lemmy_db_schema::{
    utils::queries::{
      creator_banned,
      creator_is_admin,
      local_user_can_mod,
      person1_select,
      post_tags_fragment,
    },
    utils::queries::{creator_banned_from_community, creator_is_moderator},
    Person1AliasAllColumnsTuple,
  },
};

pub mod api;
#[cfg(feature = "full")]
pub mod impls;

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "full", derive(Queryable, Selectable))]
#[cfg_attr(feature = "full", diesel(check_for_backend(diesel::pg::Pg)))]
/// A combined inbox view
pub struct InboxCombinedViewInternal {
  #[cfg_attr(feature = "full", diesel(embed))]
  pub inbox_combined: InboxCombined,
  #[cfg_attr(feature = "full", diesel(embed))]
  pub comment_reply: Option<CommentReply>,
  #[cfg_attr(feature = "full", diesel(embed))]
  pub person_comment_mention: Option<PersonCommentMention>,
  #[cfg_attr(feature = "full", diesel(embed))]
  pub person_post_mention: Option<PersonPostMention>,
  #[cfg_attr(feature = "full", diesel(embed))]
  pub private_message: Option<PrivateMessage>,
  #[cfg_attr(feature = "full", diesel(embed))]
  pub comment: Option<Comment>,
  #[cfg_attr(feature = "full", diesel(embed))]
  pub post: Option<Post>,
  #[cfg_attr(feature = "full", diesel(embed))]
  pub community: Option<Community>,
  #[cfg_attr(feature = "full", diesel(embed))]
  pub item_creator: Person,
  #[cfg_attr(feature = "full",
    diesel(
      select_expression_type = Person1AliasAllColumnsTuple,
      select_expression = person1_select()
    )
  )]
  pub item_recipient: Person,
  #[cfg_attr(feature = "full", diesel(embed))]
  pub image_details: Option<ImageDetails>,
  #[cfg_attr(feature = "full", diesel(embed))]
  pub community_actions: Option<CommunityActions>,
  #[cfg_attr(feature = "full", diesel(embed))]
  pub instance_actions: Option<InstanceActions>,
  #[cfg_attr(feature = "full", diesel(embed))]
  pub post_actions: Option<PostActions>,
  #[cfg_attr(feature = "full", diesel(embed))]
  pub person_actions: Option<PersonActions>,
  #[cfg_attr(feature = "full", diesel(embed))]
  pub comment_actions: Option<CommentActions>,
  #[cfg_attr(feature = "full",
    diesel(
      select_expression = creator_is_admin()
    )
  )]
  pub item_creator_is_admin: bool,
  #[cfg_attr(feature = "full",
    diesel(
      select_expression = post_tags_fragment()
    )
  )]
  pub post_tags: TagsView,
  #[cfg_attr(feature = "full",
    diesel(
      select_expression = local_user_can_mod()
    )
  )]
  pub can_mod: bool,
  #[cfg_attr(feature = "full",
    diesel(
      select_expression = creator_banned()
    )
  )]
  pub creator_banned: bool,
  #[cfg_attr(feature = "full",
    diesel(
      select_expression = creator_is_moderator()
    )
  )]
  pub creator_is_moderator: bool,
  #[cfg_attr(feature = "full",
    diesel(
      select_expression = creator_banned_from_community()
    )
  )]
  pub creator_banned_from_community: bool,
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(export))]
// Use serde's internal tagging, to work easier with javascript libraries
#[serde(tag = "type_")]
pub enum InboxCombinedView {
  CommentReply(CommentReplyView),
  CommentMention(PersonCommentMentionView),
  PostMention(PersonPostMentionView),
  PrivateMessage(PrivateMessageView),
}
#[skip_serializing_none]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "full", derive(Queryable))]
#[cfg_attr(feature = "full", diesel(check_for_backend(diesel::pg::Pg)))]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
/// A person comment mention view.
pub struct PersonCommentMentionView {
  pub person_comment_mention: PersonCommentMention,
  pub recipient: Person,
  pub comment: Comment,
  pub creator: Person,
  pub post: Post,
  pub community: Community,
  pub community_actions: Option<CommunityActions>,
  pub comment_actions: Option<CommentActions>,
  pub person_actions: Option<PersonActions>,
  pub instance_actions: Option<InstanceActions>,
  pub creator_is_admin: bool,
  pub can_mod: bool,
  pub creator_banned: bool,
  pub creator_is_moderator: bool,
  pub creator_banned_from_community: bool,
}

#[skip_serializing_none]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "full", derive(Queryable))]
#[cfg_attr(feature = "full", diesel(check_for_backend(diesel::pg::Pg)))]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
/// A person post mention view.
pub struct PersonPostMentionView {
  pub person_post_mention: PersonPostMention,
  pub recipient: Person,
  pub post: Post,
  pub creator: Person,
  pub community: Community,
  pub image_details: Option<ImageDetails>,
  pub community_actions: Option<CommunityActions>,
  pub person_actions: Option<PersonActions>,
  pub post_actions: Option<PostActions>,
  pub instance_actions: Option<InstanceActions>,
  pub post_tags: TagsView,
  pub creator_is_admin: bool,
  pub can_mod: bool,
  pub creator_banned: bool,
  pub creator_is_moderator: bool,
  pub creator_banned_from_community: bool,
}

#[skip_serializing_none]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "full", derive(Queryable))]
#[cfg_attr(feature = "full", diesel(check_for_backend(diesel::pg::Pg)))]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
/// A comment reply view.
pub struct CommentReplyView {
  pub comment_reply: CommentReply,
  pub recipient: Person,
  pub comment: Comment,
  pub creator: Person,
  pub post: Post,
  pub community: Community,
  pub community_actions: Option<CommunityActions>,
  pub comment_actions: Option<CommentActions>,
  pub person_actions: Option<PersonActions>,
  #[cfg_attr(feature = "full", diesel(embed))]
  pub instance_actions: Option<InstanceActions>,
  pub creator_is_admin: bool,
  pub post_tags: TagsView,
  pub can_mod: bool,
  pub creator_banned: bool,
  pub creator_is_moderator: bool,
  pub creator_banned_from_community: bool,
}

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
/// Get your inbox (replies, comment mentions, post mentions, and messages)
pub struct ListInbox {
  pub type_: Option<InboxDataType>,
  pub unread_only: Option<bool>,
  pub page_cursor: Option<PaginationCursor>,
  pub page_back: Option<bool>,
  pub limit: Option<i64>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
/// Get your inbox (replies, comment mentions, post mentions, and messages)
pub struct ListInboxResponse {
  pub inbox: Vec<InboxCombinedView>,
  /// the pagination cursor to use to fetch the next page
  pub next_page: Option<PaginationCursor>,
  pub prev_page: Option<PaginationCursor>,
}
