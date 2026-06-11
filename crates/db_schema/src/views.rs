use crate::{
  Person1AliasAllColumnsTuple,
  diesel::{ExpressionMethods, QueryDsl, SelectableHelper},
  newtypes::{CommentId, PostId},
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
  utils::queries::selects::{
    CreatorLocalHomeBanExpiresType,
    CreatorLocalHomeCommunityBanExpiresType,
    comment_creator_is_admin,
    comment_select_remove_deletes,
    creator_ban_expires_from_community,
    creator_banned_from_community,
    creator_is_moderator,
    creator_local_home_ban_expires,
    creator_local_home_community_ban_expires,
    creator_local_home_community_banned,
    local_user_can_mod_comment,
    local_user_can_mod_post,
    person1_select,
    post_community_tags_fragment,
    post_creator_is_admin,
    post_select_remove_deletes,
  },
};
use chrono::{DateTime, Utc};
use diesel::{NullableExpressionMethods, dsl::Nullable};
use diesel_async::RunQueryDsl;
use lemmy_db_schema_file::schema::{modlog, notification};
use lemmy_diesel_utils::{
  connection::{DbPool, get_conn},
  pagination::{CursorData, PaginationCursorConversion},
  traits::Crud,
};
use lemmy_utils::error::LemmyResult;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

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
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "full", derive(Queryable, Selectable))]
#[cfg_attr(feature = "full", diesel(check_for_backend(diesel::pg::Pg)))]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
/// A comment view.
pub struct CommentView {
  #[cfg_attr(feature = "full",
    diesel(
      select_expression = comment_select_remove_deletes()
    )
  )]
  pub comment: Comment,
  #[cfg_attr(feature = "full", diesel(embed))]
  pub creator: Person,
  #[cfg_attr(feature = "full", diesel(embed))]
  pub post: Post,
  #[cfg_attr(feature = "full", diesel(embed))]
  pub community: Community,
  #[cfg_attr(feature = "full", diesel(embed))]
  pub community_actions: Option<CommunityActions>,
  #[cfg_attr(feature = "full", diesel(embed))]
  pub comment_actions: Option<CommentActions>,
  #[cfg_attr(feature = "full", diesel(embed))]
  pub person_actions: Option<PersonActions>,
  #[cfg_attr(feature = "full",
    diesel(
      select_expression = comment_creator_is_admin()
    )
  )]
  pub creator_is_admin: bool,
  #[cfg_attr(feature = "full",
    diesel(
      select_expression = post_community_tags_fragment()
    )
  )]
  pub tags: CommunityTagsView,
  #[cfg_attr(feature = "full",
    diesel(
      select_expression = local_user_can_mod_comment()
    )
  )]
  pub can_mod: bool,
  #[cfg_attr(feature = "full",
    diesel(
      select_expression = creator_local_home_community_banned()
    )
  )]
  pub creator_banned: bool,
  #[cfg_attr(feature = "full",
    diesel(
      select_expression_type = CreatorLocalHomeCommunityBanExpiresType,
      select_expression = creator_local_home_community_ban_expires()
     )
  )]
  pub creator_ban_expires_at: Option<DateTime<Utc>>,
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
  #[cfg_attr(feature = "full",
    diesel(
      select_expression = creator_ban_expires_from_community()
    )
  )]
  pub creator_community_ban_expires_at: Option<DateTime<Utc>>,
}

#[skip_serializing_none]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "full", derive(Queryable, Selectable))]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
/// A post view.
pub struct PostView {
  #[cfg_attr(feature = "full",
    diesel(
      select_expression = post_select_remove_deletes()
    )
  )]
  pub post: Post,
  #[cfg_attr(feature = "full", diesel(embed))]
  pub creator: Person,
  #[cfg_attr(feature = "full", diesel(embed))]
  pub community: Community,
  #[cfg_attr(feature = "full", diesel(embed))]
  pub image_details: Option<ImageDetails>,
  #[cfg_attr(feature = "full", diesel(embed))]
  pub community_actions: Option<CommunityActions>,
  #[cfg_attr(feature = "full", diesel(embed))]
  pub person_actions: Option<PersonActions>,
  #[cfg_attr(feature = "full", diesel(embed))]
  pub post_actions: Option<PostActions>,
  #[cfg_attr(feature = "full",
    diesel(
      select_expression = post_creator_is_admin()
    )
  )]
  pub creator_is_admin: bool,
  #[cfg_attr(feature = "full",
    diesel(
      select_expression = post_community_tags_fragment()
    )
  )]
  pub tags: CommunityTagsView,
  #[cfg_attr(feature = "full",
    diesel(
      select_expression = local_user_can_mod_post()
    )
  )]
  pub can_mod: bool,
  #[cfg_attr(feature = "full",
    diesel(
      select_expression = creator_local_home_community_banned()
    )
  )]
  pub creator_banned: bool,
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
  pub creator_is_moderator: bool,
  #[cfg_attr(feature = "full",
    diesel(
      select_expression = creator_banned_from_community()
    )
  )]
  pub creator_banned_from_community: bool,
  #[cfg_attr(feature = "full",
    diesel(
      select_expression = creator_ban_expires_from_community()
    )
  )]
  pub creator_community_ban_expires_at: Option<DateTime<Utc>>,
}

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "full", derive(Queryable, Selectable))]
#[cfg_attr(feature = "full", diesel(check_for_backend(diesel::pg::Pg)))]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
/// A private message view.
pub struct PrivateMessageView {
  #[cfg_attr(feature = "full", diesel(embed))]
  pub private_message: PrivateMessage,
  #[cfg_attr(feature = "full", diesel(embed))]
  pub creator: Person,
  #[cfg_attr(feature = "full",
    diesel(
      select_expression_type = Person1AliasAllColumnsTuple,
      select_expression = person1_select()
    )
  )]
  pub recipient: Person,
}

#[skip_serializing_none]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "full", derive(Queryable, Selectable))]
#[cfg_attr(feature = "full", diesel(check_for_backend(diesel::pg::Pg)))]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(export, optional_fields))]
#[skip_serializing_none]
pub struct ModlogView {
  #[cfg_attr(feature = "full", diesel(embed))]
  pub modlog: Modlog,
  #[cfg_attr(feature = "full", diesel(embed))]
  pub moderator: Option<Person>,
  #[cfg_attr(feature = "full",
    diesel(
      select_expression_type = Nullable<Person1AliasAllColumnsTuple>,
      select_expression = person1_select().nullable()
    )
  )]
  pub target_person: Option<Person>,
  #[cfg_attr(feature = "full", diesel(embed))]
  pub target_instance: Option<Instance>,
  #[cfg_attr(feature = "full", diesel(embed))]
  pub target_community: Option<Community>,
  #[cfg_attr(feature = "full", diesel(embed))]
  pub target_post: Option<Post>,
  #[cfg_attr(feature = "full", diesel(embed))]
  pub target_comment: Option<Comment>,
}

impl PaginationCursorConversion for CommentView {
  type PaginatedType = Comment;
  fn to_cursor(&self) -> CursorData {
    CursorData::new_id(self.comment.id.0)
  }

  async fn from_cursor(
    data: CursorData,
    pool: &mut DbPool<'_>,
  ) -> LemmyResult<Self::PaginatedType> {
    Comment::read(pool, CommentId(data.id()?)).await
  }
}

impl PaginationCursorConversion for ModlogView {
  type PaginatedType = Modlog;
  fn to_cursor(&self) -> CursorData {
    CursorData::new_id(self.modlog.id.0)
  }

  async fn from_cursor(
    cursor: CursorData,
    pool: &mut DbPool<'_>,
  ) -> LemmyResult<Self::PaginatedType> {
    let conn = &mut get_conn(pool).await?;
    let query = modlog::table
      .select(Self::PaginatedType::as_select())
      .filter(modlog::id.eq(cursor.id()?));
    let token = query.first(conn).await?;

    Ok(token)
  }
}

impl PaginationCursorConversion for PostView {
  type PaginatedType = Post;
  fn to_cursor(&self) -> CursorData {
    CursorData::new_id(self.post.id.0)
  }

  async fn from_cursor(
    cursor: CursorData,
    pool: &mut DbPool<'_>,
  ) -> LemmyResult<Self::PaginatedType> {
    Post::read(pool, PostId(cursor.id()?)).await
  }
}

impl PaginationCursorConversion for NotificationView {
  type PaginatedType = Notification;

  fn to_cursor(&self) -> CursorData {
    CursorData::new_id(self.notification.id.0)
  }

  async fn from_cursor(
    cursor: CursorData,
    pool: &mut DbPool<'_>,
  ) -> LemmyResult<Self::PaginatedType> {
    let conn = &mut get_conn(pool).await?;
    let query = notification::table
      .select(Self::PaginatedType::as_select())
      .filter(notification::id.eq(cursor.id()?));
    let token = query.first(conn).await?;

    Ok(token)
  }
}
