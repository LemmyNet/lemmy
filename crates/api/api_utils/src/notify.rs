use crate::context::LemmyContext;
use lemmy_db_schema::{
  newtypes::{NotificationId, PersonId},
  source::{
    comment::Comment,
    community::{Community, CommunityActions},
    instance::InstanceActions,
    notification::{
      Notification,
      NotificationInsertForm,
      PersonNotification,
      PersonNotificationInsertForm,
    },
    person::{Person, PersonActions},
    post::{Post, PostActions},
  },
  traits::{Blockable, Crud},
};
use lemmy_db_schema_file::enums::{NotificationTypes, NotificationsMode};
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

#[derive(derive_new::new)]
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
    let form = if let Some(comment) = self.comment_opt {
      NotificationInsertForm::new_comment(comment.id)
    } else {
      NotificationInsertForm::new_post(self.post.id)
    };
    let notif = Notification::create(&mut context.pool(), &form).await?;

    notify_parent_creator(&self, notif.id, context).await?;

    notify_mentions(&self, notif.id, context).await?;

    notify_subscribers(&self, notif.id, context).await?;

    Ok(())
  }

  async fn check_notifications_allowed(
    &self,
    potential_blocker_id: PersonId,
    context: &LemmyContext,
  ) -> LemmyResult<()> {
    let pool = &mut context.pool();
    PersonActions::read_block(pool, potential_blocker_id, self.post.creator_id).await?;
    InstanceActions::read_block(pool, potential_blocker_id, self.community.instance_id).await?;
    CommunityActions::read_block(pool, potential_blocker_id, self.post.community_id).await?;
    let post_notifications = PostActions::read(pool, self.post.id, potential_blocker_id)
      .await
      .ok()
      .and_then(|a| a.notifications)
      .unwrap_or_default();
    if post_notifications == NotificationsMode::Mute {
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

  let form = NotificationInsertForm::new_private_message(view.private_message.id);
  let notif = Notification::create(&mut context.pool(), &form).await?;

  let form = PersonNotificationInsertForm::new(
    notif.id,
    local_recipient.person.id,
    NotificationTypes::PrivateMessage,
  );
  PersonNotification::create(&mut context.pool(), &form).await?;

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

async fn notify_parent_creator(
  data: &NotifyData<'_>,
  notif_id: NotificationId,
  context: &LemmyContext,
) -> LemmyResult<()> {
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

  let form =
    PersonNotificationInsertForm::new(notif_id, user_view.person.id, NotificationTypes::Reply);
  PersonNotification::create(&mut context.pool(), &form).await?;

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

async fn notify_mentions(
  data: &NotifyData<'_>,
  notif_id: NotificationId,
  context: &LemmyContext,
) -> LemmyResult<()> {
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

    let form =
      PersonNotificationInsertForm::new(notif_id, user_view.person.id, NotificationTypes::Mention);
    PersonNotification::create(&mut context.pool(), &form).await?;

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

async fn notify_subscribers(
  data: &NotifyData<'_>,
  notif_id: NotificationId,
  context: &LemmyContext,
) -> LemmyResult<()> {
  let subscribers = if data.comment_opt.is_some() {
    PostActions::list_subscribers(data.post.id, &mut context.pool()).await?
  } else {
    CommunityActions::list_subscribers(data.post.community_id, &mut context.pool()).await?
  };

  for subscriber in subscribers {
    if data
      .check_notifications_allowed(subscriber, context)
      .await
      .is_err()
    {
      continue;
    };

    let form =
      PersonNotificationInsertForm::new(notif_id, subscriber, NotificationTypes::Subscribed);
    PersonNotification::create(&mut context.pool(), &form).await?;

    if data.do_send_email {
      let user_view = LocalUserView::read_person(&mut context.pool(), subscriber).await?;
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
#[expect(clippy::unwrap_used)]
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
  use lemmy_db_views_notification::{impls::NotificationQuery, NotificationView};
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

    // Sara replied to timmys comment, but lets create the row now
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
      NotificationView::get_unread_count(pool, data.timmy.person.id, data.instance.id, true)
        .await?;
    // TODO: fails because the same notification gets returned twice
    assert_eq!(1, timmy_unread_replies);

    let timmy_inbox = NotificationQuery::default()
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
    PersonNotification::mark_read_by_id_and_person(
      pool,
      timmy_inbox[0].notification.id,
      data.timmy.person.id,
    )
    .await?;

    let timmy_unread_replies =
      NotificationView::get_unread_count(pool, data.timmy.person.id, data.instance.id, true)
        .await?;
    assert_eq!(0, timmy_unread_replies);

    let timmy_inbox_unread = NotificationQuery {
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
    let form =
      PersonNotificationInsertForm::new(notif.id, data.sara.person.id, NotificationTypes::Mention);
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

    let sara_inbox = NotificationQuery::default()
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

    let sara_inbox_after_block = NotificationQuery::default()
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
    let sara_inbox_mentions_only = NotificationQuery {
      type_: Some(InboxDataType::Mention),
      ..Default::default()
    }
    .list(pool, data.sara.person.id, data.instance.id)
    .await?;
    assert_length!(2, sara_inbox_mentions_only);

    assert_eq!(
      NotificationTypes::Mention,
      sara_inbox_mentions_only[0].person_notification.kind
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

    let sara_inbox_after_hide_bots = NotificationQuery::default()
      .list(pool, data.sara.person.id, data.instance.id)
      .await?;
    assert_length!(1, sara_inbox_after_hide_bots);

    // Make sure the post mention which jessica made is the hidden one
    assert_eq!(
      NotificationTypes::Mention,
      sara_inbox_after_hide_bots[0].person_notification.kind
    );

    // Mark them all as read
    PersonNotification::mark_all_as_read(pool, data.sara.person.id).await?;

    // Make sure none come back
    let sara_unread_mentions =
      NotificationView::get_unread_count(pool, data.sara.person.id, data.instance.id, false)
        .await?;
    assert_eq!(0, sara_unread_mentions);

    let sara_inbox_unread = NotificationQuery {
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
    let context = LemmyContext::init_test_context().await;
    let pool = &mut context.pool();
    let data = init_data(pool).await?;
    setup_private_messages(&data, &context).await?;

    let timmy_messages = filter_pm(
      NotificationQuery::default()
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
      NotificationQuery {
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

    let timmy_messages = filter_pm(
      NotificationQuery {
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
    let context = LemmyContext::init_test_context().await;
    let pool = &mut context.pool();
    let data = init_data(pool).await?;
    setup_private_messages(&data, &context).await?;

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
      NotificationQuery {
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
