use chrono::{DateTime, Utc};
use lemmy_db_schema::{
  NotificationDataType,
  newtypes::PaginationCursor,
  source::{
    comment::{Comment, CommentActions},
    community::{Community, CommunityActions},
    images::ImageDetails,
    instance::Instance,
    modlog::Modlog,
    notification::Notification,
    person::{Person, PersonActions},
    post::{Post, PostActions},
    private_message::PrivateMessage,
    tag::TagsView,
  },
};
use lemmy_db_views_comment::CommentView;
use lemmy_db_views_modlog::ModlogView;
use lemmy_db_views_post::PostView;
use lemmy_db_views_private_message::PrivateMessageView;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
#[cfg(feature = "full")]
use {
  diesel::{Queryable, Selectable},
  lemmy_db_schema::{
    Person1AliasAllColumnsTuple,
    utils::queries::selects::{
      CreatorLocalHomeBanExpiresType,
      creator_is_admin,
      creator_is_moderator,
      creator_local_home_ban_expires,
      creator_local_home_banned,
      local_user_can_mod,
    },
    utils::queries::selects::{
      creator_ban_expires_from_community,
      creator_banned_from_community,
      person1_select,
      post_tags_fragment,
    },
  },
};

pub mod api;
#[cfg(feature = "full")]
pub mod impls;
#[cfg(test)]
#[expect(clippy::indexing_slicing)]
pub mod tests;

#[derive(Clone, Debug)]
#[cfg_attr(feature = "full", derive(Queryable, Selectable))]
#[cfg_attr(feature = "full", diesel(check_for_backend(diesel::pg::Pg)))]
struct NotificationViewInternal {
  #[cfg_attr(feature = "full", diesel(embed))]
  notification: Notification,
  #[cfg_attr(feature = "full", diesel(embed))]
  private_message: Option<PrivateMessage>,
  #[cfg_attr(feature = "full", diesel(embed))]
  comment: Option<Comment>,
  #[cfg_attr(feature = "full", diesel(embed))]
  post: Option<Post>,
  #[cfg_attr(feature = "full", diesel(embed))]
  community: Option<Community>,
  #[cfg_attr(feature = "full", diesel(embed))]
  instance: Option<Instance>,
  #[cfg_attr(feature = "full", diesel(embed))]
  creator: Option<Person>,
  #[cfg_attr(feature = "full",
    diesel(
      select_expression_type = Person1AliasAllColumnsTuple,
      select_expression = person1_select()
    )
  )]
  recipient: Person,
  #[cfg_attr(feature = "full", diesel(embed))]
  image_details: Option<ImageDetails>,
  #[cfg_attr(feature = "full", diesel(embed))]
  community_actions: Option<CommunityActions>,
  #[cfg_attr(feature = "full", diesel(embed))]
  post_actions: Option<PostActions>,
  #[cfg_attr(feature = "full", diesel(embed))]
  person_actions: Option<PersonActions>,
  #[cfg_attr(feature = "full", diesel(embed))]
  comment_actions: Option<CommentActions>,
  #[cfg_attr(feature = "full", diesel(embed))]
  modlog: Option<Modlog>,
  #[cfg_attr(feature = "full",
    diesel(
      select_expression = post_tags_fragment()
    )
  )]
  post_tags: TagsView,
  #[cfg_attr(feature = "full",
    diesel(
      select_expression = creator_is_admin()
    )
  )]
  creator_is_admin: bool,
  #[cfg_attr(feature = "full",
    diesel(
      select_expression = local_user_can_mod()
    )
  )]
  can_mod: bool,
  #[cfg_attr(feature = "full",
    diesel(
      select_expression = creator_local_home_banned()
    )
  )]
  creator_banned: bool,
  #[cfg_attr(feature = "full",
    diesel(
      select_expression_type = CreatorLocalHomeBanExpiresType,
      select_expression = creator_local_home_ban_expires()
     )
  )]
  pub creator_ban_expires_at: Option<DateTime<Utc>>,
  #[cfg_attr(feature = "full",
    diesel(
      select_expression = creator_is_moderator()
    )
  )]
  creator_is_moderator: bool,
  #[cfg_attr(feature = "full",
    diesel(
      select_expression = creator_banned_from_community()
    )
  )]
  creator_banned_from_community: bool,
  #[cfg_attr(feature = "full",
    diesel(
      select_expression = creator_ban_expires_from_community()
    )
  )]
  pub creator_community_ban_expires_at: Option<DateTime<Utc>>,
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(export))]
pub struct NotificationView {
  pub notification: Notification,
  pub data: NotificationData,
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(export))]
#[serde(tag = "type_", rename_all = "snake_case")]
pub enum NotificationData {
  Comment(CommentView),
  Post(PostView),
  PrivateMessage(PrivateMessageView),
  ModAction(ModlogView),
}

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
/// Get your inbox (replies, comment mentions, post mentions, and messages)
pub struct ListNotifications {
  pub type_: Option<NotificationDataType>,
  pub unread_only: Option<bool>,
  pub page_cursor: Option<PaginationCursor>,
  pub page_back: Option<bool>,
  pub limit: Option<i64>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
/// Get your inbox (replies, comment mentions, post mentions, and messages)
pub struct ListNotificationsResponse {
  pub notifications: Vec<NotificationView>,
  /// the pagination cursor to use to fetch the next page
  pub next_page: Option<PaginationCursor>,
  pub prev_page: Option<PaginationCursor>,
}
