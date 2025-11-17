use crate::{CommentView, NotificationData, NotificationView, NotificationViewInternal};
use diesel::{
  BoolExpressionMethods,
  ExpressionMethods,
  PgExpressionMethods,
  QueryDsl,
  SelectableHelper,
};
use diesel_async::RunQueryDsl;
use i_love_jesus::SortDirection;
use lemmy_db_schema::{
  NotificationDataType,
  newtypes::{NotificationId, PaginationCursor},
  source::{
    notification::{Notification, notification_keys},
    person::Person,
  },
  traits::PaginationCursorBuilder,
  utils::{limit_fetch, queries::filters::filter_blocked},
};
use lemmy_db_schema_file::{
  enums::NotificationType,
  schema::{notification, person},
};
use lemmy_db_views_modlog::ModlogView;
use lemmy_db_views_notification_sql::notification_joins;
use lemmy_db_views_post::PostView;
use lemmy_db_views_private_message::PrivateMessageView;
use lemmy_diesel_utils::{
  connection::{DbPool, get_conn},
  utils::paginate,
};
use lemmy_utils::error::{LemmyErrorExt, LemmyErrorType, LemmyResult};

impl NotificationView {
  /// Gets the number of unread mentions
  pub async fn get_unread_count(
    pool: &mut DbPool<'_>,
    my_person: &Person,
    show_bot_accounts: bool,
  ) -> LemmyResult<i64> {
    use diesel::dsl::count;
    let conn = &mut get_conn(pool).await?;

    let unread_filter = notification::read.eq(false);

    let mut query = notification_joins(my_person.id, my_person.instance_id)
      // Filter for your user
      .filter(notification::recipient_id.eq(my_person.id))
      // Filter unreads
      .filter(unread_filter)
      // Don't count replies from blocked users
      .filter(filter_blocked())
      .select(count(notification::id))
      .into_boxed();

    // These filters need to be kept in sync with the filters in queries().list()
    if !show_bot_accounts {
      query = query.filter(person::bot_account.is_distinct_from(true));
    }

    query
      .first::<i64>(conn)
      .await
      .with_lemmy_type(LemmyErrorType::NotFound)
  }

  pub async fn read(
    pool: &mut DbPool<'_>,
    id: NotificationId,
    my_person: &Person,
  ) -> LemmyResult<Self> {
    let conn = &mut get_conn(pool).await?;

    let res = notification_joins(my_person.id, my_person.instance_id)
      .filter(notification::id.eq(id))
      .select(NotificationViewInternal::as_select())
      .get_result::<NotificationViewInternal>(conn)
      .await
      .with_lemmy_type(LemmyErrorType::NotFound)?;
    // TODO: should pass this in as param
    let hide_modlog_names = true;
    map_to_enum(res, hide_modlog_names).ok_or(LemmyErrorType::NotFound.into())
  }
}

impl PaginationCursorBuilder for NotificationView {
  type CursorData = Notification;

  fn to_cursor(&self) -> PaginationCursor {
    PaginationCursor(self.notification.id.0.to_string())
  }

  async fn from_cursor(
    cursor: &PaginationCursor,
    pool: &mut DbPool<'_>,
  ) -> LemmyResult<Self::CursorData> {
    let conn = &mut get_conn(pool).await?;
    let id: i32 = cursor.0.parse()?;
    let query = notification::table
      .select(Self::CursorData::as_select())
      .filter(notification::id.eq(id));
    let token = query.first(conn).await?;

    Ok(token)
  }
}

#[derive(Default)]
pub struct NotificationQuery {
  pub type_: Option<NotificationDataType>,
  pub unread_only: Option<bool>,
  pub show_bot_accounts: Option<bool>,
  pub hide_modlog_names: Option<bool>,
  pub cursor_data: Option<Notification>,
  pub page_back: Option<bool>,
  pub limit: Option<i64>,
  pub no_limit: Option<bool>,
}

impl NotificationQuery {
  pub async fn list(
    self,
    pool: &mut DbPool<'_>,
    my_person: &Person,
  ) -> LemmyResult<Vec<NotificationView>> {
    let conn = &mut get_conn(pool).await?;

    let mut query = notification_joins(my_person.id, my_person.instance_id)
      .select(NotificationViewInternal::as_select())
      .into_boxed();

    if !self.no_limit.unwrap_or_default() {
      let limit = limit_fetch(self.limit)?;
      query = query.limit(limit);
    }

    // Filters
    if self.unread_only.unwrap_or_default() {
      query = query
        // The recipient filter (IE only show replies to you)
        .filter(notification::recipient_id.eq(my_person.id))
        .filter(notification::read.eq(false));
    } else {
      // A special case for private messages: show messages FROM you also.
      // Use a not-null checks to catch the others
      query = query.filter(
        notification::recipient_id.eq(my_person.id).or(
          notification::private_message_id.is_not_null().and(
            notification::recipient_id
              .eq(my_person.id)
              .or(person::id.eq(my_person.id)),
          ),
        ),
      );
    }

    if !self.show_bot_accounts.unwrap_or_default() {
      query = query.filter(person::bot_account.is_distinct_from(true));
    };

    // Dont show replies from blocked users or instances
    query = query.filter(filter_blocked());

    if let Some(type_) = self.type_ {
      query = match type_ {
        NotificationDataType::All => query,
        NotificationDataType::Reply => query.filter(notification::kind.eq(NotificationType::Reply)),
        NotificationDataType::Mention => {
          query.filter(notification::kind.eq(NotificationType::Mention))
        }
        NotificationDataType::PrivateMessage => {
          query.filter(notification::kind.eq(NotificationType::PrivateMessage))
        }
        NotificationDataType::Subscribed => {
          query.filter(notification::kind.eq(NotificationType::Subscribed))
        }
        NotificationDataType::ModAction => {
          query.filter(notification::kind.eq(NotificationType::ModAction))
        }
      }
    }

    // Sorting by published
    let paginated_query = paginate(
      query,
      SortDirection::Desc,
      self.cursor_data,
      None,
      self.page_back,
    )
    .then_order_by(notification_keys::published_at)
    // Tie breaker
    .then_order_by(notification_keys::id);

    let res = paginated_query
      .load::<NotificationViewInternal>(conn)
      .await?;

    let hide_modlog_names = self.hide_modlog_names.unwrap_or_default();
    Ok(
      res
        .into_iter()
        .filter_map(|r| map_to_enum(r, hide_modlog_names))
        .collect(),
    )
  }
}

fn map_to_enum(v: NotificationViewInternal, hide_modlog_name: bool) -> Option<NotificationView> {
  let data = if let (Some(modlog), Some(creator)) = (v.modlog.clone(), v.creator.clone()) {
    let m = ModlogView {
      modlog,
      moderator: Some(creator),
      target_person: Some(v.recipient),
      target_community: v.community,
      target_post: v.post,
      target_comment: v.comment,
      target_instance: v.instance,
    };
    let m = m.hide_mod_name(hide_modlog_name);
    NotificationData::ModAction(m)
  } else if let (Some(comment), Some(post), Some(community), Some(creator)) = (
    v.comment.clone(),
    v.post.clone(),
    v.community.clone(),
    v.creator.clone(),
  ) {
    NotificationData::Comment(CommentView {
      comment,
      post,
      community,
      creator,
      community_actions: v.community_actions,
      person_actions: v.person_actions,
      comment_actions: v.comment_actions,
      post_tags: v.post_tags,
      creator_banned_from_community: v.creator_banned_from_community,
      creator_community_ban_expires_at: v.creator_community_ban_expires_at,
      creator_is_admin: v.creator_is_admin,
      can_mod: v.can_mod,
      creator_banned: v.creator_banned,
      creator_ban_expires_at: v.creator_ban_expires_at,
      creator_is_moderator: v.creator_is_moderator,
    })
  } else if let (Some(post), Some(community), Some(creator)) =
    (v.post.clone(), v.community.clone(), v.creator.clone())
  {
    NotificationData::Post(PostView {
      post,
      community,
      creator,
      image_details: v.image_details,
      community_actions: v.community_actions,
      post_actions: v.post_actions,
      person_actions: v.person_actions,
      tags: v.post_tags,
      creator_banned_from_community: v.creator_banned_from_community,
      creator_community_ban_expires_at: v.creator_community_ban_expires_at,
      creator_is_admin: v.creator_is_admin,
      can_mod: v.can_mod,
      creator_banned: v.creator_banned,
      creator_ban_expires_at: v.creator_ban_expires_at,
      creator_is_moderator: v.creator_is_moderator,
    })
  } else if let (Some(private_message), Some(creator)) =
    (v.private_message.clone(), v.creator.clone())
  {
    NotificationData::PrivateMessage(PrivateMessageView {
      private_message,
      creator,
      recipient: v.recipient,
    })
  } else {
    return None;
  };
  Some(NotificationView {
    notification: v.notification,
    data,
  })
}
