use lemmy_db_schema::{
  newtypes::PaginationCursor,
  source::{
    comment::{Comment, CommentActions},
    community::{Community, CommunityActions},
    images::ImageDetails,
    instance::InstanceActions,
    notification::Notification,
    person::{Person, PersonActions},
    post::{Post, PostActions},
    private_message::PrivateMessage,
    tag::TagsView,
  },
  InboxDataType,
};
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
#[cfg(feature = "full")]
use {
  diesel::{Queryable, Selectable},
  lemmy_db_schema::{
    utils::queries::{creator_banned, creator_is_admin, local_user_can_mod, post_tags_fragment},
    utils::queries::{creator_banned_from_community, creator_is_moderator},
  },
};

pub mod api;
#[cfg(feature = "full")]
pub mod impls;

#[derive(Clone)]
#[cfg_attr(feature = "full", derive(Queryable, Selectable))]
#[cfg_attr(feature = "full", diesel(check_for_backend(diesel::pg::Pg)))]
struct NotificationViewInternal {
  #[cfg_attr(feature = "full", diesel(embed))]
  pub notification: Notification,
  #[cfg_attr(feature = "full", diesel(embed))]
  pub private_message: Option<PrivateMessage>,
  #[cfg_attr(feature = "full", diesel(embed))]
  pub comment: Option<Comment>,
  #[cfg_attr(feature = "full", diesel(embed))]
  pub post: Option<Post>,
  #[cfg_attr(feature = "full", diesel(embed))]
  pub community: Option<Community>,
  #[cfg_attr(feature = "full", diesel(embed))]
  pub creator: Person,
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
  pub creator_is_admin: bool,
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
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
pub struct NotificationView {
  pub notification: Notification,
  pub creator: Person,
  pub image_details: Option<ImageDetails>,
  pub community_actions: Option<CommunityActions>,
  pub instance_actions: Option<InstanceActions>,
  pub post_actions: Option<PostActions>,
  pub person_actions: Option<PersonActions>,
  pub comment_actions: Option<CommentActions>,
  pub creator_is_admin: bool,
  pub post_tags: TagsView,
  pub can_mod: bool,
  pub creator_banned: bool,
  pub creator_is_moderator: bool,
  pub creator_banned_from_community: bool,
  pub data: NotificationData,
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(export))]
#[serde(tag = "type_")]
pub enum NotificationData {
  Post {
    post: Post,
    community: Community,
  },
  Comment {
    comment: Comment,
    post: Post,
    community: Community,
  },
  PrivateMessage {
    pm: PrivateMessage,
  },
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
  pub inbox: Vec<NotificationView>,
  /// the pagination cursor to use to fetch the next page
  pub next_page: Option<PaginationCursor>,
  pub prev_page: Option<PaginationCursor>,
}
