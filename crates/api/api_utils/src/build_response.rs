use crate::{context::LemmyContext, utils::is_mod_or_admin};
use actix_web::web::Json;
use lemmy_db_schema::{
  newtypes::{CommentId, CommunityId, InstanceId, PersonId, PostId},
  source::{
    actor_language::CommunityLanguage,
    comment::Comment,
    comment_reply::{CommentReply, CommentReplyInsertForm},
    community::{Community, CommunityActions},
    instance::InstanceActions,
    person::{Person, PersonActions},
    person_comment_mention::{PersonCommentMention, PersonCommentMentionInsertForm},
    person_post_mention::{PersonPostMention, PersonPostMentionInsertForm},
    post::{Post, PostActions},
  },
  traits::{Blockable, Crud},
};
use lemmy_db_schema_file::enums::PostNotifications;
use lemmy_db_views_comment::{api::CommentResponse, CommentView};
use lemmy_db_views_community::{api::CommunityResponse, CommunityView};
use lemmy_db_views_local_user::LocalUserView;
use lemmy_db_views_post::{api::PostResponse, PostView};
use lemmy_email::notifications::{send_mention_email, send_reply_email};
use lemmy_utils::{
  error::{LemmyErrorType, LemmyResult},
  utils::mention::scrape_text_for_mentions,
};
use url::Url;

pub async fn build_comment_response(
  context: &LemmyContext,
  comment_id: CommentId,
  local_user_view: Option<LocalUserView>,
  local_instance_id: InstanceId,
) -> LemmyResult<CommentResponse> {
  let local_user = local_user_view.map(|l| l.local_user);
  let comment_view = CommentView::read(
    &mut context.pool(),
    comment_id,
    local_user.as_ref(),
    local_instance_id,
  )
  .await?;
  Ok(CommentResponse { comment_view })
}

pub async fn build_community_response(
  context: &LemmyContext,
  local_user_view: LocalUserView,
  community_id: CommunityId,
) -> LemmyResult<Json<CommunityResponse>> {
  let is_mod_or_admin = is_mod_or_admin(&mut context.pool(), &local_user_view, community_id)
    .await
    .is_ok();
  let local_user = local_user_view.local_user;
  let community_view = CommunityView::read(
    &mut context.pool(),
    community_id,
    Some(&local_user),
    is_mod_or_admin,
  )
  .await?;
  let discussion_languages = CommunityLanguage::read(&mut context.pool(), community_id).await?;

  Ok(Json(CommunityResponse {
    community_view,
    discussion_languages,
  }))
}

pub async fn build_post_response(
  context: &LemmyContext,
  community_id: CommunityId,
  local_user_view: LocalUserView,
  post_id: PostId,
) -> LemmyResult<Json<PostResponse>> {
  let is_mod_or_admin = is_mod_or_admin(&mut context.pool(), &local_user_view, community_id)
    .await
    .is_ok();
  let local_user = local_user_view.local_user;
  let post_view = PostView::read(
    &mut context.pool(),
    post_id,
    Some(&local_user),
    local_user_view.person.instance_id,
    is_mod_or_admin,
  )
  .await?;
  Ok(Json(PostResponse { post_view }))
}

#[derive(derive_new::new)]
pub struct NotifyData<'a> {
  post: &'a Post,
  comment_opt: Option<&'a Comment>,
  community: &'a Community,
  creator: &'a Person,
  do_send_email: bool,
}

impl NotifyData<'_> {
  pub async fn send(self, context: &LemmyContext) -> LemmyResult<()> {
    let parent_creator = notify_parent_creator(&self, context).await?;

    send_local_mentions(&self, parent_creator, context).await?;

    send_subscriber_notifications(&self, context).await?;

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

/// Scans the post/comment content for mentions, then sends notifications via db and email
/// to mentioned users and parent creator.
pub async fn send_local_notifs(
  post: &Post,
  comment_opt: Option<&Comment>,
  person: &Person,
  community: &Community,
  do_send_email: bool,
  context: &LemmyContext,
) -> LemmyResult<()> {
  todo!()
}

async fn notify_parent_creator(
  data: &NotifyData<'_>,
  context: &LemmyContext,
) -> LemmyResult<Option<PersonId>> {
  let Some(comment) = data.comment_opt.as_ref() else {
    return Ok(None);
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
    return Ok(None);
  }

  let is_blocked = data
    .check_notifications_allowed(parent_creator_id, context)
    .await
    .is_err();
  if is_blocked {
    return Ok(None);
  }

  let Ok(user_view) = LocalUserView::read_person(&mut context.pool(), parent_creator_id).await
  else {
    return Ok(None);
  };

  let comment_reply_form = CommentReplyInsertForm {
    recipient_id: user_view.person.id,
    comment_id: comment.id,
    read: None,
  };

  // Allow this to fail softly, since comment edits might re-update or replace it
  // Let the uniqueness handle this fail
  CommentReply::create(&mut context.pool(), &comment_reply_form)
    .await
    .ok();

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
  Ok(Some(user_view.person.id))
}

async fn send_local_mentions(
  data: &NotifyData<'_>,
  parent_creator_id: Option<PersonId>,
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

    // Dont send any mention notification to parent creator nor to yourself
    if Some(user_view.person.id) == parent_creator_id || user_view.person.id == data.creator.id {
      continue;
    }

    let is_blocked = data
      .check_notifications_allowed(user_view.person.id, context)
      .await
      .is_err();
    if is_blocked {
      continue;
    };

    insert_post_or_comment_mention(&user_view, &data.post, data.comment_opt, context).await?;

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

async fn send_subscriber_notifications(
  data: &NotifyData<'_>,
  context: &LemmyContext,
) -> LemmyResult<()> {
  let subscribers = PostActions::list_subscribers(data.post.id, &mut context.pool()).await?;

  for subscriber in subscribers {
    let user_view = LocalUserView::read_person(&mut context.pool(), subscriber).await?;

    // TODO: need to check blocks and mentioned users, parent creator here?

    // TODO: would be easier if we use the same db table and email template here, eg with
    // `type` param
    insert_post_or_comment_mention(&user_view, &data.post, data.comment_opt, context).await?;

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

/// Make the correct reply form depending on whether its a post or comment mention
async fn insert_post_or_comment_mention(
  mention_user_view: &LocalUserView,
  post: &Post,
  comment_opt: Option<&Comment>,
  context: &LemmyContext,
) -> LemmyResult<()> {
  if let Some(comment) = &comment_opt {
    let person_comment_mention_form = PersonCommentMentionInsertForm {
      recipient_id: mention_user_view.person.id,
      comment_id: comment.id,
      read: None,
    };

    // Allow this to fail softly, since comment edits might re-update or replace it
    // Let the uniqueness handle this fail
    PersonCommentMention::create(&mut context.pool(), &person_comment_mention_form)
      .await
      .ok();
  } else {
    let person_post_mention_form = PersonPostMentionInsertForm {
      recipient_id: mention_user_view.person.id,
      post_id: post.id,
      read: None,
    };

    // Allow this to fail softly, since edits might re-update or replace it
    PersonPostMention::create(&mut context.pool(), &person_post_mention_form)
      .await
      .ok();
  }
  Ok(())
}
