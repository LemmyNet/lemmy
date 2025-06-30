use crate::context::LemmyContext;
use lemmy_db_schema::{
  newtypes::{NotificationId, PersonId},
  source::{
    comment::Comment,
    community::{Community, CommunityActions},
    instance::InstanceActions,
    notification::{
      LocalUserNotification,
      LocalUserNotificationInsertForm,
      Notification,
      NotificationInsertForm,
    },
    person::{Person, PersonActions},
    post::{Post, PostActions},
  },
  traits::{Blockable, Crud},
};
use lemmy_db_schema_file::enums::{NotificationTypes, PostNotifications};
use lemmy_db_views_local_user::LocalUserView;
use lemmy_email::notifications::{send_mention_email, send_reply_email};
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
    if post_notifications == PostNotifications::Mute {
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

  let form = LocalUserNotificationInsertForm::new(
    notif_id,
    user_view.local_user.id,
    NotificationTypes::Reply,
  );
  LocalUserNotification::create(&mut context.pool(), &form).await?;

  if data.do_send_email {
    send_reply_email(
      &user_view,
      &comment,
      &data.creator,
      &parent_comment,
      &data.post,
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

    let form = LocalUserNotificationInsertForm::new(
      notif_id,
      user_view.local_user.id,
      NotificationTypes::Mention,
    );
    LocalUserNotification::create(&mut context.pool(), &form).await?;

    // Send an email to those local users that have notifications on
    if data.do_send_email {
      send_mention_email(
        &user_view,
        &data.content(),
        &data.creator,
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
  let subscribers = PostActions::list_subscribers(data.post.id, &mut context.pool()).await?;

  for subscriber in subscribers {
    let user_view = LocalUserView::read_person(&mut context.pool(), subscriber).await?;

    // TODO: need to check blocks and mentioned users, parent creator here?

    // TODO: would be easier if we use the same db table and email template here, eg with
    // `type` param
    let form = LocalUserNotificationInsertForm::new(
      notif_id,
      user_view.local_user.id,
      NotificationTypes::Mention,
    );
    LocalUserNotification::create(&mut context.pool(), &form).await?;

    if data.do_send_email {
      send_mention_email(
        &user_view,
        &data.content(),
        &data.creator,
        data.link(context)?.into(),
        context.settings(),
      )
      .await;
    }
  }
  Ok(())
}
