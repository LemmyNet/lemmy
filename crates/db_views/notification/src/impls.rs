use crate::NotificationView;
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
  aliases::{self},
  newtypes::{InstanceId, PaginationCursor, PersonId},
  source::notification::{notification_keys, Notification},
  traits::PaginationCursorBuilder,
  utils::{
    get_conn,
    limit_fetch,
    paginate,
    queries::{
      community_join,
      creator_community_actions_join,
      creator_home_instance_actions_join,
      creator_local_instance_actions_join,
      creator_local_user_admin_join,
      image_details_join,
      my_comment_actions_join,
      my_community_actions_join,
      my_instance_actions_person_join,
      my_local_user_admin_join,
      my_person_actions_join,
      my_post_actions_join,
    },
    DbPool,
  },
  InboxDataType,
};
use lemmy_db_schema_file::{
  enums::NotificationTypes,
  schema::{
    comment,
    instance_actions,
    notification,
    person,
    person_actions,
    person_notification,
    post,
    private_message,
  },
};
use lemmy_utils::error::{LemmyErrorExt, LemmyErrorType, LemmyResult};

impl NotificationView {
  #[diesel::dsl::auto_type(no_type_alias)]
  fn joins(my_person_id: PersonId, local_instance_id: InstanceId) -> _ {
    let item_creator = person::id;
    let recipient_person = aliases::person1.field(person::id);

    let item_creator_join = person::table.on(
      comment::creator_id
        .eq(item_creator)
        .or(post::creator_id.eq(item_creator))
        .or(private_message::creator_id.eq(item_creator)),
    );

    let recipient_join = aliases::person1.on(
      person_notification::recipient_id
        .eq(recipient_person)
        .or(private_message::recipient_id.eq(recipient_person)),
    );

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
      my_community_actions_join(Some(my_person_id));
    let my_post_actions_join: my_post_actions_join = my_post_actions_join(Some(my_person_id));
    let my_comment_actions_join: my_comment_actions_join =
      my_comment_actions_join(Some(my_person_id));
    let my_local_user_admin_join: my_local_user_admin_join =
      my_local_user_admin_join(Some(my_person_id));
    let my_instance_actions_person_join: my_instance_actions_person_join =
      my_instance_actions_person_join(Some(my_person_id));
    let my_person_actions_join: my_person_actions_join = my_person_actions_join(Some(my_person_id));
    let creator_local_instance_actions_join: creator_local_instance_actions_join =
      creator_local_instance_actions_join(local_instance_id);

    notification::table
      .inner_join(person_notification::table)
      .left_join(private_message_join)
      .left_join(comment_join)
      .left_join(post_join)
      .left_join(community_join())
      .inner_join(item_creator_join)
      .inner_join(recipient_join)
      .left_join(image_details_join())
      .left_join(creator_community_actions_join())
      .left_join(my_local_user_admin_join)
      .left_join(creator_local_user_admin_join())
      .left_join(my_community_actions_join)
      .left_join(my_instance_actions_person_join)
      .left_join(creator_home_instance_actions_join())
      .left_join(creator_local_instance_actions_join)
      .left_join(my_post_actions_join)
      .left_join(my_person_actions_join)
      .left_join(my_comment_actions_join)
  }

  /// Gets the number of unread mentions
  pub async fn get_unread_count(
    pool: &mut DbPool<'_>,
    my_person_id: PersonId,
    local_instance_id: InstanceId,
    show_bot_accounts: bool,
  ) -> LemmyResult<i64> {
    use diesel::dsl::count;
    let conn = &mut get_conn(pool).await?;

    let recipient_person = aliases::person1.field(person::id);

    let unread_filter = person_notification::read
      .eq(false)
      // If its unread, I only want the messages to me
      .or(
        private_message::read
          .eq(false)
          .and(private_message::recipient_id.eq(my_person_id)),
      );

    let mut query = Self::joins(my_person_id, local_instance_id)
      // Filter for your user
      .filter(recipient_person.eq(my_person_id))
      // Filter unreads
      .filter(unread_filter)
      // Don't count replies from blocked users
      .filter(person_actions::blocked_at.is_null())
      .filter(instance_actions::blocked_at.is_null())
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
pub struct InboxCombinedQuery {
  pub type_: Option<InboxDataType>,
  pub unread_only: Option<bool>,
  pub show_bot_accounts: Option<bool>,
  pub cursor_data: Option<Notification>,
  pub page_back: Option<bool>,
  pub limit: Option<i64>,
  pub no_limit: Option<bool>,
}

impl InboxCombinedQuery {
  pub async fn list(
    self,
    pool: &mut DbPool<'_>,
    my_person_id: PersonId,
    local_instance_id: InstanceId,
  ) -> LemmyResult<Vec<NotificationView>> {
    let conn = &mut get_conn(pool).await?;

    let item_creator = person::id;
    let recipient_person = aliases::person1.field(person::id);

    let mut query = NotificationView::joins(my_person_id, local_instance_id)
      .select(NotificationView::as_select())
      .into_boxed();

    if !self.no_limit.unwrap_or_default() {
      let limit = limit_fetch(self.limit)?;
      query = query.limit(limit);
    }

    // Filters
    if self.unread_only.unwrap_or_default() {
      query = query
        // The recipient filter (IE only show replies to you)
        .filter(recipient_person.eq(my_person_id))
        .filter(
          person_notification::read
            .eq(false)
            // If its unread, I only want the messages to me
            .or(private_message::read.eq(false)),
        );
    } else {
      // A special case for private messages: show messages FROM you also.
      // Use a not-null checks to catch the others
      query = query.filter(
        recipient_person.eq(my_person_id).or(
          notification::private_message_id.is_not_null().and(
            recipient_person
              .eq(my_person_id)
              .or(item_creator.eq(my_person_id)),
          ),
        ),
      );
    }

    if !(self.show_bot_accounts.unwrap_or_default()) {
      query = query.filter(not(person::bot_account));
    };

    // Dont show replies from blocked users or instances
    query = query
      .filter(person_actions::blocked_at.is_null())
      .filter(instance_actions::blocked_at.is_null());

    if let Some(type_) = self.type_ {
      query = match type_ {
        InboxDataType::All => query,
        InboxDataType::CommentReply => {
          query.filter(person_notification::kind.eq(NotificationTypes::Reply))
        }
        InboxDataType::Mention => {
          query.filter(person_notification::kind.eq(NotificationTypes::Mention))
        }
        InboxDataType::PrivateMessage => {
          query.filter(notification::private_message_id.is_not_null())
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

    let res = paginated_query.load::<NotificationView>(conn).await?;

    Ok(res)
  }
}

#[cfg(test)]
#[expect(clippy::indexing_slicing)]
#[expect(clippy::unwrap_used)]
mod tests {
  use crate::{impls::InboxCombinedQuery, NotificationView};
  use lemmy_db_schema::{
    assert_length,
    source::{
      comment::{Comment, CommentInsertForm},
      community::{Community, CommunityInsertForm},
      instance::{Instance, InstanceActions, InstanceBlockForm},
      notification::{
        Notification,
        NotificationInsertForm,
        PersonNotification,
        PersonNotificationInsertForm,
      },
      person::{Person, PersonActions, PersonBlockForm, PersonInsertForm, PersonUpdateForm},
      post::{Post, PostInsertForm},
      private_message::{PrivateMessage, PrivateMessageInsertForm},
    },
    traits::{Blockable, Crud},
    utils::{build_db_pool_for_tests, DbPool},
    InboxDataType,
  };
  use lemmy_db_schema_file::enums::NotificationTypes;
  use lemmy_db_views_local_user::LocalUserView;
  use lemmy_utils::error::LemmyResult;
  use pretty_assertions::assert_eq;
  use serial_test::serial;

  struct Data {
    instance: Instance,
    timmy: LocalUserView,
    sara: LocalUserView,
    jessica: Person,
    timmy_post: Post,
    jessica_post: Post,
    timmy_comment: Comment,
    sara_comment: Comment,
  }

  async fn init_data(pool: &mut DbPool<'_>) -> LemmyResult<Data> {
    let instance = Instance::read_or_create(pool, "my_domain.tld".to_string()).await?;

    let timmy = LocalUserView::create_test_user(pool, "timmy_pcv", "", false).await?;

    let sara = LocalUserView::create_test_user(pool, "sara_pcv", "", false).await?;

    let jessica_form = PersonInsertForm::test_form(instance.id, "jessica_mrv");
    let jessica = Person::create(pool, &jessica_form).await?;

    let community_form = CommunityInsertForm::new(
      instance.id,
      "test community pcv".to_string(),
      "nada".to_owned(),
      "pubkey".to_string(),
    );
    let community = Community::create(pool, &community_form).await?;

    let timmy_post_form =
      PostInsertForm::new("timmy post prv".into(), timmy.person.id, community.id);
    let timmy_post = Post::create(pool, &timmy_post_form).await?;

    let jessica_post_form =
      PostInsertForm::new("jessica post prv".into(), jessica.id, community.id);
    let jessica_post = Post::create(pool, &jessica_post_form).await?;

    let timmy_comment_form =
      CommentInsertForm::new(timmy.person.id, timmy_post.id, "timmy comment prv".into());
    let timmy_comment = Comment::create(pool, &timmy_comment_form, None).await?;

    let sara_comment_form =
      CommentInsertForm::new(sara.person.id, timmy_post.id, "sara comment prv".into());
    let sara_comment = Comment::create(pool, &sara_comment_form, Some(&timmy_comment.path)).await?;

    Ok(Data {
      instance,
      timmy,
      sara,
      jessica,
      timmy_post,
      jessica_post,
      timmy_comment,
      sara_comment,
    })
  }

  async fn setup_private_messages(data: &Data, pool: &mut DbPool<'_>) -> LemmyResult<()> {
    let sara_timmy_message_form = PrivateMessageInsertForm::new(
      data.sara.person.id,
      data.timmy.person.id,
      "sara to timmy".into(),
    );
    PrivateMessage::create(pool, &sara_timmy_message_form).await?;

    let sara_jessica_message_form = PrivateMessageInsertForm::new(
      data.sara.person.id,
      data.jessica.id,
      "sara to jessica".into(),
    );
    PrivateMessage::create(pool, &sara_jessica_message_form).await?;

    let timmy_sara_message_form = PrivateMessageInsertForm::new(
      data.timmy.person.id,
      data.sara.person.id,
      "timmy to sara".into(),
    );
    PrivateMessage::create(pool, &timmy_sara_message_form).await?;

    let jessica_timmy_message_form = PrivateMessageInsertForm::new(
      data.jessica.id,
      data.timmy.person.id,
      "jessica to timmy".into(),
    );
    PrivateMessage::create(pool, &jessica_timmy_message_form).await?;

    Ok(())
  }

  async fn cleanup(data: Data, pool: &mut DbPool<'_>) -> LemmyResult<()> {
    Instance::delete(pool, data.instance.id).await?;

    Ok(())
  }

  #[tokio::test]
  #[serial]
  async fn replies() -> LemmyResult<()> {
    let pool = &build_db_pool_for_tests();
    let pool = &mut pool.into();
    let data = init_data(pool).await?;

    // Sara replied to timmys comment, but lets create the row now
    let form = NotificationInsertForm::new_comment(data.sara_comment.id);
    let reply = Notification::create(pool, &form).await?;
    let form =
      PersonNotificationInsertForm::new(reply.id, data.timmy.person.id, NotificationTypes::Reply);
    PersonNotification::create(pool, &form).await?;

    let timmy_unread_replies =
      NotificationView::get_unread_count(pool, data.timmy.person.id, data.instance.id, true)
        .await?;
    assert_eq!(1, timmy_unread_replies);

    let timmy_inbox = InboxCombinedQuery::default()
      .list(pool, data.timmy.person.id, data.instance.id)
      .await?;
    assert_length!(1, timmy_inbox);

    assert_eq!(
      Some(data.sara_comment.id),
      timmy_inbox[0].notification.comment_id
    );
    assert_eq!(
      data.sara_comment.id,
      timmy_inbox[0].comment.as_ref().unwrap().id
    );
    assert_eq!(data.timmy_post.id, timmy_inbox[0].post.as_ref().unwrap().id);
    assert_eq!(data.sara.person.id, timmy_inbox[0].creator.id);
    assert_eq!(data.timmy.person.id, timmy_inbox[0].recipient.id);
    assert_eq!(
      NotificationTypes::Mention,
      timmy_inbox[0].person_notification.kind
    );

    // Mark it as read
    PersonNotification::mark_read_by_id_and_person(pool, reply.id, data.timmy.local_user.id)
      .await?;

    let timmy_unread_replies =
      NotificationView::get_unread_count(pool, data.timmy.person.id, data.instance.id, true)
        .await?;
    assert_eq!(0, timmy_unread_replies);

    let timmy_inbox_unread = InboxCombinedQuery {
      unread_only: Some(true),
      ..Default::default()
    }
    .list(pool, data.timmy.person.id, data.instance.id)
    .await?;
    assert_length!(0, timmy_inbox_unread);

    cleanup(data, pool).await?;

    Ok(())
  }

  #[tokio::test]
  #[serial]
  async fn mentions() -> LemmyResult<()> {
    let pool = &build_db_pool_for_tests();
    let pool = &mut pool.into();
    let data = init_data(pool).await?;

    // Timmy mentions sara in a comment
    let timmy_mention_sara_comment_form =
      NotificationInsertForm::new_comment(data.timmy_comment.id);
    let notif = Notification::create(pool, &timmy_mention_sara_comment_form).await?;
    let form = PersonNotificationInsertForm::new(
      notif.id,
      data.sara.local_user.id,
      NotificationTypes::Mention,
    );
    PersonNotification::create(pool, &form).await?;

    // Jessica mentions sara in a post
    let jessica_mention_sara_post_form = NotificationInsertForm::new_post(data.jessica_post.id);
    let notif = Notification::create(pool, &jessica_mention_sara_post_form).await?;
    let form =
      PersonNotificationInsertForm::new(notif.id, data.sara.person.id, NotificationTypes::Mention);
    PersonNotification::create(pool, &form).await?;

    // Test to make sure counts and blocks work correctly
    let sara_unread_mentions =
      NotificationView::get_unread_count(pool, data.sara.person.id, data.instance.id, true).await?;
    assert_eq!(2, sara_unread_mentions);

    let sara_inbox = InboxCombinedQuery::default()
      .list(pool, data.sara.person.id, data.instance.id)
      .await?;
    assert_length!(2, sara_inbox);

    assert_eq!(
      Some(data.jessica_post.id),
      sara_inbox[0].notification.post_id
    );
    assert_eq!(
      data.jessica_post.id,
      sara_inbox[0].post.as_ref().unwrap().id
    );
    assert_eq!(data.jessica.id, sara_inbox[0].creator.id);
    assert_eq!(data.sara.person.id, sara_inbox[0].recipient.id);
    assert_eq!(
      NotificationTypes::Mention,
      sara_inbox[0].person_notification.kind
    );

    assert_eq!(
      Some(data.timmy_comment.id),
      sara_inbox[1].notification.comment_id
    );
    assert_eq!(
      data.timmy_comment.id,
      sara_inbox[1].comment.as_ref().unwrap().id
    );
    assert_eq!(data.timmy_post.id, sara_inbox[1].post.as_ref().unwrap().id);
    assert_eq!(data.timmy.person.id, sara_inbox[1].creator.id);
    assert_eq!(data.sara.person.id, sara_inbox[1].recipient.id);
    assert_eq!(
      NotificationTypes::Mention,
      sara_inbox[1].person_notification.kind
    );

    // Sara blocks timmy, and make sure these counts are now empty
    let sara_blocks_timmy_form = PersonBlockForm::new(data.sara.person.id, data.timmy.person.id);
    PersonActions::block(pool, &sara_blocks_timmy_form).await?;

    let sara_unread_mentions_after_block =
      NotificationView::get_unread_count(pool, data.sara.person.id, data.instance.id, true).await?;
    assert_eq!(1, sara_unread_mentions_after_block);

    let sara_inbox_after_block = InboxCombinedQuery::default()
      .list(pool, data.sara.person.id, data.instance.id)
      .await?;
    assert_length!(1, sara_inbox_after_block);

    // Make sure the comment mention which timmy made is the hidden one
    assert_eq!(
      NotificationTypes::Mention,
      sara_inbox_after_block[0].person_notification.kind
    );

    // Unblock user so we can reuse the same person
    PersonActions::unblock(pool, &sara_blocks_timmy_form).await?;

    // Test the type filter
    let sara_inbox_post_mentions_only = InboxCombinedQuery {
      type_: Some(InboxDataType::Mention),
      ..Default::default()
    }
    .list(pool, data.sara.person.id, data.instance.id)
    .await?;
    assert_length!(1, sara_inbox_post_mentions_only);

    assert_eq!(
      NotificationTypes::Mention,
      sara_inbox_post_mentions_only[0].person_notification.kind
    );

    // Turn Jessica into a bot account
    let person_update_form = PersonUpdateForm {
      bot_account: Some(true),
      ..Default::default()
    };
    Person::update(pool, data.jessica.id, &person_update_form).await?;

    // Make sure sara hides bots
    let sara_unread_mentions_after_hide_bots =
      NotificationView::get_unread_count(pool, data.sara.person.id, data.instance.id, false)
        .await?;
    assert_eq!(1, sara_unread_mentions_after_hide_bots);

    let sara_inbox_after_hide_bots = InboxCombinedQuery::default()
      .list(pool, data.sara.person.id, data.instance.id)
      .await?;
    assert_length!(1, sara_inbox_after_hide_bots);

    // Make sure the post mention which jessica made is the hidden one
    assert_eq!(
      NotificationTypes::Mention,
      sara_inbox_after_hide_bots[0].person_notification.kind
    );

    // Mark them all as read
    PersonNotification::mark_all_as_read(pool, data.sara.local_user.id).await?;

    // Make sure none come back
    let sara_unread_mentions =
      NotificationView::get_unread_count(pool, data.sara.person.id, data.instance.id, false)
        .await?;
    assert_eq!(0, sara_unread_mentions);

    let sara_inbox_unread = InboxCombinedQuery {
      unread_only: Some(true),
      ..Default::default()
    }
    .list(pool, data.sara.person.id, data.instance.id)
    .await?;
    assert_length!(0, sara_inbox_unread);

    cleanup(data, pool).await?;

    Ok(())
  }

  fn filter_pm(inbox: Vec<NotificationView>) -> Vec<NotificationView> {
    inbox
      .into_iter()
      .filter(|f| f.private_message.is_some())
      .collect::<Vec<NotificationView>>()
  }

  #[tokio::test]
  #[serial]
  async fn read_private_messages() -> LemmyResult<()> {
    let pool = &build_db_pool_for_tests();
    let pool = &mut pool.into();
    let data = init_data(pool).await?;
    setup_private_messages(&data, pool).await?;

    let timmy_messages = filter_pm(
      InboxCombinedQuery::default()
        .list(pool, data.timmy.person.id, data.instance.id)
        .await?,
    );

    // The read even shows timmy's sent messages
    assert_length!(3, &timmy_messages);
    assert_eq!(timmy_messages[0].creator.id, data.jessica.id);
    assert_eq!(timmy_messages[0].recipient.id, data.timmy.person.id);
    assert_eq!(timmy_messages[1].creator.id, data.timmy.person.id);
    assert_eq!(timmy_messages[1].recipient.id, data.sara.person.id);
    assert_eq!(timmy_messages[2].creator.id, data.sara.person.id);
    assert_eq!(timmy_messages[2].recipient.id, data.timmy.person.id);

    let timmy_unread =
      NotificationView::get_unread_count(pool, data.timmy.person.id, data.instance.id, false)
        .await?;
    assert_eq!(2, timmy_unread);

    let timmy_unread_messages = filter_pm(
      InboxCombinedQuery {
        unread_only: Some(true),
        ..Default::default()
      }
      .list(pool, data.timmy.person.id, data.instance.id)
      .await?,
    );

    // The unread hides timmy's sent messages
    assert_length!(2, &timmy_unread_messages);
    assert_eq!(timmy_unread_messages[0].creator.id, data.jessica.id);
    assert_eq!(timmy_unread_messages[0].recipient.id, data.timmy.person.id);
    assert_eq!(timmy_unread_messages[1].creator.id, data.sara.person.id);
    assert_eq!(timmy_unread_messages[1].recipient.id, data.timmy.person.id);

    cleanup(data, pool).await?;

    Ok(())
  }

  #[tokio::test]
  #[serial]
  async fn ensure_private_message_person_block() -> LemmyResult<()> {
    let pool = &build_db_pool_for_tests();
    let pool = &mut pool.into();
    let data = init_data(pool).await?;
    setup_private_messages(&data, pool).await?;

    // Make sure blocks are working
    let timmy_blocks_sara_form = PersonBlockForm::new(data.timmy.person.id, data.sara.person.id);

    let inserted_block = PersonActions::block(pool, &timmy_blocks_sara_form).await?;

    assert_eq!(
      (data.timmy.person.id, data.sara.person.id, true),
      (
        inserted_block.person_id,
        inserted_block.target_id,
        inserted_block.blocked_at.is_some()
      )
    );

    let timmy_messages = filter_pm(
      InboxCombinedQuery {
        unread_only: Some(true),
        ..Default::default()
      }
      .list(pool, data.timmy.person.id, data.instance.id)
      .await?,
    );

    assert_length!(1, &timmy_messages);

    let timmy_unread =
      NotificationView::get_unread_count(pool, data.timmy.person.id, data.instance.id, false)
        .await?;
    assert_eq!(1, timmy_unread);

    cleanup(data, pool).await?;

    Ok(())
  }

  #[tokio::test]
  #[serial]
  async fn ensure_private_message_instance_block() -> LemmyResult<()> {
    let pool = &build_db_pool_for_tests();
    let pool = &mut pool.into();
    let data = init_data(pool).await?;
    setup_private_messages(&data, pool).await?;

    // Make sure instance_blocks are working
    let timmy_blocks_instance_form =
      InstanceBlockForm::new(data.timmy.person.id, data.sara.person.instance_id);

    let inserted_instance_block = InstanceActions::block(pool, &timmy_blocks_instance_form).await?;

    assert_eq!(
      (data.timmy.person.id, data.sara.person.instance_id, true),
      (
        inserted_instance_block.person_id,
        inserted_instance_block.instance_id,
        inserted_instance_block.blocked_at.is_some()
      )
    );

    let timmy_messages = filter_pm(
      InboxCombinedQuery {
        unread_only: Some(true),
        ..Default::default()
      }
      .list(pool, data.timmy.person.id, data.instance.id)
      .await?,
    );

    assert_length!(0, &timmy_messages);

    let timmy_unread =
      NotificationView::get_unread_count(pool, data.timmy.person.id, data.instance.id, false)
        .await?;
    assert_eq!(0, timmy_unread);

    cleanup(data, pool).await?;

    Ok(())
  }
}
