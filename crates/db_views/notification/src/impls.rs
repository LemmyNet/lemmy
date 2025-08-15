use crate::{CommentView, NotificationData, NotificationView, NotificationViewInternal};
use diesel::{
  dsl::not,
  BoolExpressionMethods,
  ExpressionMethods,
  JoinOnDsl,
  NullableExpressionMethods,
  QueryDsl,
  SelectableHelper,
};
use diesel_async::RunQueryDsl;
use i_love_jesus::SortDirection;
use lemmy_db_schema::{
  aliases,
  newtypes::PaginationCursor,
  source::{
    notification::{notification_keys, Notification},
    person::Person,
  },
  traits::PaginationCursorBuilder,
  utils::{
    get_conn,
    limit_fetch,
    paginate,
    queries::{
      filters::filter_blocked,
      joins::{
        community_join,
        creator_community_actions_join,
        creator_home_instance_actions_join,
        creator_local_instance_actions_join,
        creator_local_user_admin_join,
        image_details_join,
        my_comment_actions_join,
        my_community_actions_join,
        my_instance_communities_actions_join,
        my_instance_persons_actions_join_1,
        my_local_user_admin_join,
        my_person_actions_join,
        my_post_actions_join,
      },
    },
    DbPool,
  },
  NotificationDataType,
};
use lemmy_db_schema_file::{
  enums::NotificationTypes,
  schema::{comment, notification, person, post, private_message},
};
use lemmy_db_views_post::PostView;
use lemmy_db_views_private_message::PrivateMessageView;
use lemmy_utils::error::{LemmyErrorExt, LemmyErrorType, LemmyResult};

impl NotificationView {
  #[diesel::dsl::auto_type(no_type_alias)]
  fn joins(my_person: &Person) -> _ {
    let item_creator = person::id;
    let recipient_person = aliases::person1.field(person::id);

    let item_creator_join = person::table.on(
      comment::creator_id
        .eq(item_creator)
        .or(
          notification::post_id
            .is_not_null()
            .and(post::creator_id.eq(item_creator)),
        )
        .or(private_message::creator_id.eq(item_creator)),
    );

    let recipient_join = aliases::person1.on(notification::recipient_id.eq(recipient_person));

    let comment_join = comment::table.on(
      notification::comment_id
        .eq(comment::id.nullable())
        // Filter out the deleted / removed
        .and(not(comment::deleted))
        .and(not(comment::removed)),
    );

    let post_join = post::table.on(
      notification::post_id
        .eq(post::id.nullable())
        .or(comment::post_id.eq(post::id))
        // Filter out the deleted / removed
        .and(not(post::deleted))
        .and(not(post::removed)),
    );

    // This could be a simple join, but you need to check for deleted here
    let private_message_join = private_message::table.on(
      notification::private_message_id
        .eq(private_message::id.nullable())
        .and(not(private_message::deleted))
        .and(not(private_message::removed)),
    );

    let my_community_actions_join: my_community_actions_join =
      my_community_actions_join(Some(my_person.id));
    let my_post_actions_join: my_post_actions_join = my_post_actions_join(Some(my_person.id));
    let my_comment_actions_join: my_comment_actions_join =
      my_comment_actions_join(Some(my_person.id));
    let my_local_user_admin_join: my_local_user_admin_join =
      my_local_user_admin_join(Some(my_person.id));
    let my_instance_communities_actions_join: my_instance_communities_actions_join =
      my_instance_communities_actions_join(Some(my_person.id));
    let my_instance_persons_actions_join_1: my_instance_persons_actions_join_1 =
      my_instance_persons_actions_join_1(Some(my_person.id));
    let my_person_actions_join: my_person_actions_join = my_person_actions_join(Some(my_person.id));
    let creator_local_instance_actions_join: creator_local_instance_actions_join =
      creator_local_instance_actions_join(my_person.instance_id);

    notification::table
      .left_join(private_message_join)
      .left_join(comment_join)
      .left_join(post_join)
      .left_join(community_join())
      .inner_join(item_creator_join)
      .inner_join(recipient_join)
      .left_join(image_details_join())
      .left_join(creator_community_actions_join())
      .left_join(creator_local_user_admin_join())
      .left_join(creator_home_instance_actions_join())
      .left_join(creator_local_instance_actions_join)
      .left_join(my_local_user_admin_join)
      .left_join(my_community_actions_join)
      .left_join(my_instance_communities_actions_join)
      .left_join(my_instance_persons_actions_join_1)
      .left_join(my_post_actions_join)
      .left_join(my_person_actions_join)
      .left_join(my_comment_actions_join)
  }

  /// Gets the number of unread mentions
  pub async fn get_unread_count(
    pool: &mut DbPool<'_>,
    my_person: &Person,
    show_bot_accounts: bool,
  ) -> LemmyResult<i64> {
    use diesel::dsl::count;
    let conn = &mut get_conn(pool).await?;

    let unread_filter = notification::read.eq(false);

    let mut query = Self::joins(my_person)
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
      query = query.filter(not(person::bot_account));
    }

    query
      .first::<i64>(conn)
      .await
      .with_lemmy_type(LemmyErrorType::NotFound)
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

    let mut query = NotificationView::joins(my_person)
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

    if !(self.show_bot_accounts.unwrap_or_default()) {
      query = query.filter(not(person::bot_account));
    };

    // Dont show replies from blocked users or instances
    query = query.filter(filter_blocked());

    if let Some(type_) = self.type_ {
      query = match type_ {
        NotificationDataType::All => query,
        NotificationDataType::Reply => {
          query.filter(notification::kind.eq(NotificationTypes::Reply))
        }
        NotificationDataType::Mention => {
          query.filter(notification::kind.eq(NotificationTypes::Mention))
        }
        NotificationDataType::PrivateMessage => {
          query.filter(notification::kind.eq(NotificationTypes::PrivateMessage))
        }
        NotificationDataType::Subscribed => {
          query.filter(notification::kind.eq(NotificationTypes::Subscribed))
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

    Ok(res.into_iter().filter_map(map_to_enum).collect())
  }
}

fn map_to_enum(v: NotificationViewInternal) -> Option<NotificationView> {
  let data = if let (Some(comment), Some(post), Some(community)) =
    (v.comment, v.post.clone(), v.community.clone())
  {
    NotificationData::Comment(CommentView {
      comment,
      post,
      community,
      creator: v.creator,
      community_actions: v.community_actions,
      person_actions: v.person_actions,
      comment_actions: v.comment_actions,
      creator_is_admin: v.creator_is_admin,
      post_tags: v.post_tags,
      can_mod: v.can_mod,
      creator_banned: v.creator_banned,
      creator_ban_expires_at: v.creator_ban_expires_at,
      creator_is_moderator: v.creator_is_moderator,
      creator_banned_from_community: v.creator_banned_from_community,
      creator_community_ban_expires_at: v.creator_community_ban_expires_at,
    })
  } else if let (Some(post), Some(community)) = (v.post, v.community) {
    NotificationData::Post(PostView {
      post,
      community,
      creator: v.creator,
      image_details: v.image_details,
      community_actions: v.community_actions,
      post_actions: v.post_actions,
      person_actions: v.person_actions,
      creator_is_admin: v.creator_is_admin,
      tags: v.post_tags,
      can_mod: v.can_mod,
      creator_banned: v.creator_banned,
      creator_ban_expires_at: v.creator_ban_expires_at,
      creator_is_moderator: v.creator_is_moderator,
      creator_banned_from_community: v.creator_banned_from_community,
      creator_community_ban_expires_at: v.creator_community_ban_expires_at,
    })
  } else if let Some(private_message) = v.private_message {
    NotificationData::PrivateMessage(PrivateMessageView {
      private_message,
      creator: v.creator,
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
