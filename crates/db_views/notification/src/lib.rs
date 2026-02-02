use chrono::{DateTime, Utc};
use lemmy_db_schema::{
  NotificationTypeFilter,
  source::{
    comment::{Comment, CommentActions},
    community::{Community, CommunityActions},
    community_tag::CommunityTagsView,
    images::ImageDetails,
    instance::Instance,
    modlog::Modlog,
    notification::Notification,
    person::{Person, PersonActions},
    post::{Post, PostActions},
    private_message::PrivateMessage,
  },
};
use lemmy_db_views_comment::CommentView;
use lemmy_db_views_modlog::ModlogView;
use lemmy_db_views_post::PostView;
use lemmy_db_views_private_message::PrivateMessageView;
use lemmy_diesel_utils::pagination::PaginationCursor;
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
      post_community_tags_fragment,
    },
  },
};

pub mod api;
#[cfg(feature = "full")]
pub mod impls;
#[cfg(test)]
#[expect(clippy::indexing_slicing)]
pub mod tests;

#[cfg(feature = "full")]
#[derive(Clone, Debug, Queryable, Selectable)]
#[diesel(check_for_backend(diesel::pg::Pg))]
struct NotificationViewInternal {
  #[diesel(embed)]
  notification: Notification,
  #[diesel(embed)]
  private_message: Option<PrivateMessage>,
  #[diesel(embed)]
  comment: Option<Comment>,
  #[diesel(embed)]
  post: Option<Post>,
  #[diesel(embed)]
  community: Option<Community>,
  #[diesel(embed)]
  instance: Option<Instance>,
  #[diesel(embed)]
  creator: Option<Person>,
  #[diesel(
    select_expression_type = Person1AliasAllColumnsTuple,
    select_expression = person1_select()
  )]
  recipient: Person,
  #[diesel(embed)]
  image_details: Option<ImageDetails>,
  #[diesel(embed)]
  community_actions: Option<CommunityActions>,
  #[diesel(embed)]
  post_actions: Option<PostActions>,
  #[diesel(embed)]
  person_actions: Option<PersonActions>,
  #[diesel(embed)]
  comment_actions: Option<CommentActions>,
  #[diesel(embed)]
  modlog: Option<Modlog>,
  #[diesel(select_expression = post_community_tags_fragment())]
  tags: CommunityTagsView,
  #[diesel(select_expression = creator_is_admin())]
  creator_is_admin: bool,
  #[diesel(select_expression = local_user_can_mod())]
  can_mod: bool,
  #[diesel(select_expression = creator_local_home_banned())]
  creator_banned: bool,
  #[diesel(
    select_expression_type = CreatorLocalHomeBanExpiresType,
    select_expression = creator_local_home_ban_expires()
  )]
  pub creator_ban_expires_at: Option<DateTime<Utc>>,
  #[diesel(select_expression = creator_is_moderator())]
  creator_is_moderator: bool,
  #[diesel(select_expression = creator_banned_from_community())]
  creator_banned_from_community: bool,
  #[diesel(select_expression = creator_ban_expires_from_community())]
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
  pub type_: Option<NotificationTypeFilter>,
  pub unread_only: Option<bool>,
  pub page_cursor: Option<PaginationCursor>,
  pub limit: Option<i64>,
}
