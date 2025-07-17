use crate::context::LemmyContext;
use lemmy_db_schema::{
  newtypes::PersonId,
  source::{
    comment::Comment,
    community::{Community, CommunityActions},
    instance::InstanceActions,
    notification::{Notification, NotificationInsertForm},
    person::{Person, PersonActions},
    post::{Post, PostActions},
  },
  traits::{Blockable, Crud},
};
use lemmy_db_schema_file::enums::{
  CommunityNotificationsMode,
  NotificationTypes,
  PostNotificationsMode,
};
use lemmy_db_views_local_user::LocalUserView;
use lemmy_db_views_private_message::PrivateMessageView;
use lemmy_db_views_site::SiteView;
use lemmy_email::notifications::{
  send_community_subscribed_email,
  send_mention_email,
  send_post_subscribed_email,
  send_private_message_email,
  send_reply_email,
};
use lemmy_utils::{
  error::{LemmyErrorType, LemmyResult},
  utils::mention::scrape_text_for_mentions,
};
use url::Url;

#[derive(derive_new::new, Debug)]
pub struct NotifyData<'a> {
  post: &'a Post,
  comment_opt: Option<&'a Comment>,
  creator: &'a Person,
  community: &'a Community,
  do_send_email: bool,
}

impl NotifyData<'_> {
  /// Scans the post/comment content for mentions, then sends notifications via db and email
  /// to mentioned users and parent creator.
  pub async fn send(self, context: &LemmyContext) -> LemmyResult<()> {
    notify_parent_creator(&self, context).await?;

    notify_mentions(&self, context).await?;

    notify_subscribers(&self, context).await?;

    Ok(())
  }

  async fn check_notifications_allowed(
    &self,
    potential_blocker_id: PersonId,
    context: &LemmyContext,
  ) -> LemmyResult<()> {
    let pool = &mut context.pool();
    // TODO: this needs too many queries for each user
    PersonActions::read_block(pool, potential_blocker_id, self.post.creator_id).await?;
    InstanceActions::read_block(pool, potential_blocker_id, self.community.instance_id).await?;
    CommunityActions::read_block(pool, potential_blocker_id, self.post.community_id).await?;
    let post_notifications = PostActions::read(pool, self.post.id, potential_blocker_id)
      .await
      .ok()
      .and_then(|a| a.notifications)
      .unwrap_or_default();
    let community_notifications =
      CommunityActions::read(pool, self.community.id, potential_blocker_id)
        .await
        .ok()
        .and_then(|a| a.notifications)
        .unwrap_or_default();
    if post_notifications == PostNotificationsMode::Mute
      || community_notifications == CommunityNotificationsMode::Mute
    {
      // The specific error type is irrelevant
      return Err(LemmyErrorType::NotFound.into());
    }

    Ok(())
  }

  fn content(&self) -> String {
    if let Some(comment) = self.comment_opt.as_ref() {
      comment.content.clone()
    } else {
      self.post.body.clone().unwrap_or_default()
    }
  }

  fn link(&self, context: &LemmyContext) -> LemmyResult<Url> {
    if let Some(comment) = self.comment_opt.as_ref() {
      Ok(comment.local_url(context.settings())?)
    } else {
      Ok(self.post.local_url(context.settings())?)
    }
  }

  async fn insert(
    &self,
    recipient_id: PersonId,
    kind: NotificationTypes,
    context: &LemmyContext,
  ) -> LemmyResult<Notification> {
    let form = if let Some(comment) = self.comment_opt {
      NotificationInsertForm::new_comment(comment.id, recipient_id, kind)
    } else {
      NotificationInsertForm::new_post(self.post.id, recipient_id, kind)
    };
    Notification::create(&mut context.pool(), &form).await
  }
}

pub async fn notify_private_message(
  view: &PrivateMessageView,
  is_create: bool,
  context: &LemmyContext,
) -> LemmyResult<()> {
  let Ok(local_recipient) =
    LocalUserView::read_person(&mut context.pool(), view.recipient.id).await
  else {
    return Ok(());
  };

  let form = NotificationInsertForm::new_private_message(
    view.private_message.id,
    local_recipient.person.id,
    NotificationTypes::PrivateMessage,
  );
  Notification::create(&mut context.pool(), &form).await?;

  if is_create {
    let site_view = SiteView::read_local(&mut context.pool()).await?;
    if !site_view.local_site.disable_email_notifications {
      send_private_message_email(
        &view.creator,
        &local_recipient,
        &view.private_message.content,
        context.settings(),
      )
      .await;
    }
  }
  Ok(())
}

async fn notify_parent_creator(data: &NotifyData<'_>, context: &LemmyContext) -> LemmyResult<()> {
  let Some(comment) = data.comment_opt.as_ref() else {
    return Ok(());
  };

  // Get the parent data
  let (parent_creator_id, parent_comment) =
    if let Some(parent_comment_id) = comment.parent_comment_id() {
      let parent_comment = Comment::read(&mut context.pool(), parent_comment_id).await?;
      (parent_comment.creator_id, Some(parent_comment))
    } else {
      (data.post.creator_id, None)
    };

  // Dont send notification to yourself
  if parent_creator_id == data.creator.id {
    return Ok(());
  }

  let is_blocked = data
    .check_notifications_allowed(parent_creator_id, context)
    .await
    .is_err();
  if is_blocked {
    return Ok(());
  }

  let Ok(user_view) = LocalUserView::read_person(&mut context.pool(), parent_creator_id).await
  else {
    return Ok(());
  };

  data
    .insert(user_view.person.id, NotificationTypes::Reply, context)
    .await?;

  if data.do_send_email {
    send_reply_email(
      &user_view,
      comment,
      data.creator,
      &parent_comment,
      data.post,
      context.settings(),
    )
    .await?;
  }
  Ok(())
}

async fn notify_mentions(data: &NotifyData<'_>, context: &LemmyContext) -> LemmyResult<()> {
  let mentions = scrape_text_for_mentions(&data.content())
    .into_iter()
    .filter(|m| m.is_local(&context.settings().hostname) && m.name.ne(&data.creator.name));
  for mention in mentions {
    // Ignore error if user is remote
    let Ok(user_view) = LocalUserView::read_from_name(&mut context.pool(), &mention.name).await
    else {
      continue;
    };

    let is_blocked = data
      .check_notifications_allowed(user_view.person.id, context)
      .await
      .is_err();
    if is_blocked {
      continue;
    };

    data
      .insert(user_view.person.id, NotificationTypes::Mention, context)
      .await?;

    // Send an email to those local users that have notifications on
    if data.do_send_email {
      send_mention_email(
        &user_view,
        &data.content(),
        data.creator,
        data.link(context)?.into(),
        context.settings(),
      )
      .await;
    }
  }
  Ok(())
}

async fn notify_subscribers(data: &NotifyData<'_>, context: &LemmyContext) -> LemmyResult<()> {
  let is_post = data.comment_opt.is_none();
  let subscribers = vec![
    PostActions::list_subscribers(data.post.id, &mut context.pool()).await?,
    CommunityActions::list_subscribers(data.post.community_id, is_post, &mut context.pool())
      .await?,
  ]
  .into_iter()
  .flatten()
  .collect::<Vec<_>>();

  for person_id in subscribers {
    if data
      .check_notifications_allowed(person_id, context)
      .await
      .is_err()
    {
      continue;
    };

    data
      .insert(person_id, NotificationTypes::Subscribed, context)
      .await?;

    if data.do_send_email {
      let user_view = LocalUserView::read_person(&mut context.pool(), person_id).await?;
      if let Some(comment) = data.comment_opt {
        send_post_subscribed_email(
          &user_view,
          data.post,
          comment,
          data.link(context)?.into(),
          context.settings(),
        )
        .await;
      } else {
        send_community_subscribed_email(
          &user_view,
          data.post,
          data.community,
          data.link(context)?.into(),
          context.settings(),
        )
        .await;
      }
    }
  }

  Ok(())
}

#[cfg(test)]
#[expect(clippy::indexing_slicing)]
mod tests {
  use crate::{
    context::LemmyContext,
    notify::{notify_private_message, NotifyData},
  };
  use lemmy_db_schema::{
    assert_length,
    source::{
      comment::{Comment, CommentInsertForm},
      community::{Community, CommunityInsertForm},
      instance::{Instance, InstanceActions, InstanceBlockForm},
      notification::{Notification, NotificationInsertForm},
      person::{Person, PersonActions, PersonBlockForm, PersonInsertForm, PersonUpdateForm},
      post::{Post, PostInsertForm},
      private_message::{PrivateMessage, PrivateMessageInsertForm},
    },
    traits::{Blockable, Crud},
    utils::{build_db_pool_for_tests, DbPool},
    NotificationDataType,
  };
  use lemmy_db_schema_file::enums::NotificationTypes;
  use lemmy_db_views_local_user::LocalUserView;
  use lemmy_db_views_notification::{impls::NotificationQuery, NotificationData, NotificationView};
  use lemmy_db_views_private_message::PrivateMessageView;
  use lemmy_utils::error::LemmyResult;
  use pretty_assertions::assert_eq;
  use serial_test::serial;

  struct Data {
    instance: Instance,
    timmy: LocalUserView,
    sara: LocalUserView,
    jessica: Person,
    community: Community,
    timmy_post: Post,
    jessica_post: Post,
    timmy_comment: Comment,
    sara_comment: Comment,
  }

  async fn init_data(pool: &mut DbPool<'_>) -> LemmyResult<Data> {
    let instance = Instance::read_or_create(pool, "example.com".to_string()).await?;

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
      community,
      timmy_post,
      jessica_post,
      timmy_comment,
      sara_comment,
    })
  }

  async fn insert_private_message(
    form: PrivateMessageInsertForm,
    context: &LemmyContext,
  ) -> LemmyResult<()> {
    let pool = &mut context.pool();
    let pm = PrivateMessage::create(pool, &form).await?;
    let view = PrivateMessageView::read(pool, pm.id).await?;
    notify_private_message(&view, false, context).await?;
    Ok(())
  }
  async fn setup_private_messages(data: &Data, context: &LemmyContext) -> LemmyResult<()> {
    let sara_timmy_message_form = PrivateMessageInsertForm::new(
      data.sara.person.id,
      data.timmy.person.id,
      "sara to timmy".into(),
    );
    insert_private_message(sara_timmy_message_form, context).await?;

    let sara_jessica_message_form = PrivateMessageInsertForm::new(
      data.sara.person.id,
      data.jessica.id,
      "sara to jessica".into(),
    );
    insert_private_message(sara_jessica_message_form, context).await?;

    let timmy_sara_message_form = PrivateMessageInsertForm::new(
      data.timmy.person.id,
      data.sara.person.id,
      "timmy to sara".into(),
    );
    insert_private_message(timmy_sara_message_form, context).await?;

    let jessica_timmy_message_form = PrivateMessageInsertForm::new(
      data.jessica.id,
      data.timmy.person.id,
      "jessica to timmy".into(),
    );
    insert_private_message(jessica_timmy_message_form, context).await?;

    Ok(())
  }

  async fn cleanup(data: Data, pool: &mut DbPool<'_>) -> LemmyResult<()> {
    Instance::delete(pool, data.instance.id).await?;

    Ok(())
  }

  #[tokio::test]
  #[serial]
  async fn replies() -> LemmyResult<()> {
    let context = LemmyContext::init_test_context().await;
    let pool = &mut context.pool();
    let data = init_data(pool).await?;

    // Sara replied to timmys comment
    NotifyData {
      post: &data.timmy_post,
      comment_opt: Some(&data.sara_comment),
      creator: &data.sara.person,
      community: &data.community,
      do_send_email: false,
    }
    .send(&context)
    .await?;

    let timmy_unread_replies =
      NotificationView::get_unread_count(pool, &data.timmy.person, true).await?;
    assert_eq!(1, timmy_unread_replies);

    let timmy_inbox = NotificationQuery::default()
      .list(pool, &data.timmy.person)
      .await?;
    assert_length!(1, timmy_inbox);

    if let NotificationData::Comment(comment) = &timmy_inbox[0].data {
      assert_eq!(data.sara_comment.id, comment.comment.id);
      assert_eq!(data.timmy_post.id, comment.post.id);
      assert_eq!(data.sara.person.id, comment.creator.id);
      assert_eq!(
        data.timmy.person.id,
        timmy_inbox[0].notification.recipient_id
      );
      assert_eq!(NotificationTypes::Reply, timmy_inbox[0].notification.kind);
    } else {
      panic!("wrong type")
    };

    // Mark it as read
    Notification::mark_read_by_id_and_person(
      pool,
      timmy_inbox[0].notification.id,
      data.timmy.person.id,
    )
    .await?;

    let timmy_unread_replies =
      NotificationView::get_unread_count(pool, &data.timmy.person, true).await?;
    assert_eq!(0, timmy_unread_replies);

    let timmy_inbox_unread = NotificationQuery {
      unread_only: Some(true),
      ..Default::default()
    }
    .list(pool, &data.timmy.person)
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
    let timmy_mention_sara_comment_form = NotificationInsertForm::new_comment(
      data.timmy_comment.id,
      data.sara.person.id,
      NotificationTypes::Mention,
    );
    Notification::create(pool, &timmy_mention_sara_comment_form).await?;

    // Jessica mentions sara in a post
    let jessica_mention_sara_post_form = NotificationInsertForm::new_post(
      data.jessica_post.id,
      data.sara.person.id,
      NotificationTypes::Mention,
    );
    Notification::create(pool, &jessica_mention_sara_post_form).await?;

    // Test to make sure counts and blocks work correctly
    let sara_unread_mentions =
      NotificationView::get_unread_count(pool, &data.sara.person, true).await?;
    assert_eq!(2, sara_unread_mentions);

    let sara_inbox = NotificationQuery::default()
      .list(pool, &data.sara.person)
      .await?;
    assert_length!(2, sara_inbox);

    if let NotificationData::Post(post) = &sara_inbox[0].data {
      assert_eq!(data.jessica_post.id, post.post.id);
      assert_eq!(data.jessica.id, post.creator.id);
    } else {
      panic!("wrong type")
    }
    assert_eq!(data.sara.person.id, sara_inbox[0].notification.recipient_id);
    assert_eq!(NotificationTypes::Mention, sara_inbox[0].notification.kind);

    if let NotificationData::Comment(comment) = &sara_inbox[1].data {
      assert_eq!(data.timmy_comment.id, comment.comment.id);
      assert_eq!(data.timmy_post.id, comment.post.id);
      assert_eq!(data.timmy.person.id, comment.creator.id);
    } else {
      panic!("wrong type");
    }
    assert_eq!(data.sara.person.id, sara_inbox[1].notification.recipient_id);
    assert_eq!(NotificationTypes::Mention, sara_inbox[1].notification.kind);

    // Sara blocks timmy, and make sure these counts are now empty
    let sara_blocks_timmy_form = PersonBlockForm::new(data.sara.person.id, data.timmy.person.id);
    PersonActions::block(pool, &sara_blocks_timmy_form).await?;

    let sara_unread_mentions_after_block =
      NotificationView::get_unread_count(pool, &data.sara.person, true).await?;
    assert_eq!(1, sara_unread_mentions_after_block);

    let sara_inbox_after_block = NotificationQuery::default()
      .list(pool, &data.sara.person)
      .await?;
    assert_length!(1, sara_inbox_after_block);

    // Make sure the comment mention which timmy made is the hidden one
    assert_eq!(
      NotificationTypes::Mention,
      sara_inbox_after_block[0].notification.kind
    );

    // Unblock user so we can reuse the same person
    PersonActions::unblock(pool, &sara_blocks_timmy_form).await?;

    // Test the type filter
    let sara_inbox_mentions_only = NotificationQuery {
      type_: Some(NotificationDataType::Mention),
      ..Default::default()
    }
    .list(pool, &data.sara.person)
    .await?;
    assert_length!(2, sara_inbox_mentions_only);

    assert_eq!(
      NotificationTypes::Mention,
      sara_inbox_mentions_only[0].notification.kind
    );

    // Turn Jessica into a bot account
    let person_update_form = PersonUpdateForm {
      bot_account: Some(true),
      ..Default::default()
    };
    Person::update(pool, data.jessica.id, &person_update_form).await?;

    // Make sure sara hides bot
    let sara_unread_mentions_after_hide_bots =
      NotificationView::get_unread_count(pool, &data.sara.person, false).await?;
    assert_eq!(1, sara_unread_mentions_after_hide_bots);

    let sara_inbox_after_hide_bots = NotificationQuery::default()
      .list(pool, &data.sara.person)
      .await?;
    assert_length!(1, sara_inbox_after_hide_bots);

    // Make sure the post mention which jessica made is the hidden one
    assert_eq!(
      NotificationTypes::Mention,
      sara_inbox_after_hide_bots[0].notification.kind
    );

    // Mark them all as read
    Notification::mark_all_as_read(pool, data.sara.person.id).await?;

    // Make sure none come back
    let sara_unread_mentions =
      NotificationView::get_unread_count(pool, &data.sara.person, true).await?;
    assert_eq!(0, sara_unread_mentions);

    let sara_inbox_unread = NotificationQuery {
      unread_only: Some(true),
      ..Default::default()
    }
    .list(pool, &data.sara.person)
    .await?;
    assert_length!(0, sara_inbox_unread);

    cleanup(data, pool).await?;

    Ok(())
  }

  /// Useful in combination with filter_map
  fn to_pm(x: NotificationView) -> Option<PrivateMessageView> {
    if let NotificationData::PrivateMessage(v) = x.data {
      Some(v)
    } else {
      None
    }
  }

  #[tokio::test]
  #[serial]
  async fn read_private_messages() -> LemmyResult<()> {
    let context = LemmyContext::init_test_context().await;
    let pool = &mut context.pool();
    let data = init_data(pool).await?;
    setup_private_messages(&data, &context).await?;

    let timmy_messages: Vec<_> = NotificationQuery::default()
      .list(pool, &data.timmy.person)
      .await?
      .into_iter()
      .filter_map(to_pm)
      .collect();

    // The read even shows timmy's sent messages
    assert_length!(3, &timmy_messages);
    assert_eq!(timmy_messages[0].creator.id, data.jessica.id);
    assert_eq!(timmy_messages[0].recipient.id, data.timmy.person.id);
    assert_eq!(timmy_messages[1].creator.id, data.timmy.person.id);
    assert_eq!(timmy_messages[1].recipient.id, data.sara.person.id);
    assert_eq!(timmy_messages[2].creator.id, data.sara.person.id);
    assert_eq!(timmy_messages[2].recipient.id, data.timmy.person.id);

    let timmy_unread = NotificationView::get_unread_count(pool, &data.timmy.person, true).await?;
    assert_eq!(2, timmy_unread);

    let timmy_unread_messages: Vec<_> = NotificationQuery {
      unread_only: Some(true),
      ..Default::default()
    }
    .list(pool, &data.timmy.person)
    .await?
    .into_iter()
    .filter_map(to_pm)
    .collect();

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
    let context = LemmyContext::init_test_context().await;
    let pool = &mut context.pool();
    let data = init_data(pool).await?;
    setup_private_messages(&data, &context).await?;

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

    let timmy_messages: Vec<_> = NotificationQuery {
      unread_only: Some(true),
      ..Default::default()
    }
    .list(pool, &data.timmy.person)
    .await?
    .into_iter()
    .filter_map(to_pm)
    .collect();

    assert_length!(1, &timmy_messages);

    let timmy_unread = NotificationView::get_unread_count(pool, &data.timmy.person, true).await?;
    assert_eq!(1, timmy_unread);

    cleanup(data, pool).await?;

    Ok(())
  }

  #[tokio::test]
  #[serial]
  async fn ensure_private_message_instance_block() -> LemmyResult<()> {
    let context = LemmyContext::init_test_context().await;
    let pool = &mut context.pool();
    let data = init_data(pool).await?;
    setup_private_messages(&data, &context).await?;

    // Make sure instance_blocks are working
    let timmy_blocks_instance_form =
      InstanceBlockForm::new(data.timmy.person.id, data.sara.person.instance_id);

    let inserted_instance_block = InstanceActions::block(pool, &timmy_blocks_instance_form).await?;

    assert_eq!(data.timmy.person.id, inserted_instance_block.person_id);
    assert_eq!(
      data.sara.person.instance_id,
      inserted_instance_block.instance_id
    );
    assert!(inserted_instance_block.blocked_at.is_some());

    let timmy_messages: Vec<_> = NotificationQuery {
      unread_only: Some(true),
      ..Default::default()
    }
    .list(pool, &data.timmy.person)
    .await?
    .into_iter()
    .filter_map(to_pm)
    .collect();

    assert_length!(0, &timmy_messages);

    let timmy_unread = NotificationView::get_unread_count(pool, &data.timmy.person, true).await?;
    assert_eq!(0, timmy_unread);

    cleanup(data, pool).await?;

    Ok(())
  }
}
