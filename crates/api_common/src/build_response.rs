use crate::{
  comment::CommentResponse,
  community::CommunityResponse,
  context::LemmyContext,
  post::PostResponse,
  utils::{
    check_person_instance_community_block,
    get_interface_language,
    is_mod_or_admin,
    send_email_to_user,
  },
};
use actix_web::web::Json;
use lemmy_db_schema::{
  newtypes::{CommentId, CommunityId, LocalUserId, PostId},
  source::{
    actor_language::CommunityLanguage,
    comment::Comment,
    comment_reply::{CommentReply, CommentReplyInsertForm},
    person::Person,
    person_mention::{PersonMention, PersonMentionInsertForm},
  },
  traits::Crud,
};
use lemmy_db_views::structs::{CommentView, LocalUserView, PostView};
use lemmy_db_views_actor::structs::CommunityView;
use lemmy_utils::{
  error::LemmyResult,
  utils::{markdown::markdown_to_html, mention::MentionData},
  LemmyErrorType,
};

pub async fn build_comment_response(
  context: &LemmyContext,
  comment_id: CommentId,
  local_user_view: Option<LocalUserView>,
  recipient_ids: Vec<LocalUserId>,
) -> LemmyResult<CommentResponse> {
  let person_id = local_user_view.map(|l| l.person.id);
  let comment_view = CommentView::read(&mut context.pool(), comment_id, person_id)
    .await?
    .ok_or(LemmyErrorType::CouldntFindComment)?;
  Ok(CommentResponse {
    comment_view,
    recipient_ids,
  })
}

pub async fn build_community_response(
  context: &LemmyContext,
  local_user_view: LocalUserView,
  community_id: CommunityId,
) -> LemmyResult<Json<CommunityResponse>> {
  let is_mod_or_admin = is_mod_or_admin(&mut context.pool(), &local_user_view.person, community_id)
    .await
    .is_ok();
  let person_id = local_user_view.person.id;
  let community_view = CommunityView::read(
    &mut context.pool(),
    community_id,
    Some(person_id),
    is_mod_or_admin,
  )
  .await?
  .ok_or(LemmyErrorType::CouldntFindCommunity)?;
  let discussion_languages = CommunityLanguage::read(&mut context.pool(), community_id).await?;

  Ok(Json(CommunityResponse {
    community_view,
    discussion_languages,
  }))
}

pub async fn build_post_response(
  context: &LemmyContext,
  community_id: CommunityId,
  person: &Person,
  post_id: PostId,
) -> LemmyResult<Json<PostResponse>> {
  let is_mod_or_admin = is_mod_or_admin(&mut context.pool(), person, community_id)
    .await
    .is_ok();
  let post_view = PostView::read(
    &mut context.pool(),
    post_id,
    Some(person.id),
    is_mod_or_admin,
  )
  .await?
  .ok_or(LemmyErrorType::CouldntFindPost)?;
  Ok(Json(PostResponse { post_view }))
}

// TODO: this function is a mess and should be split up to handle email separately
#[tracing::instrument(skip_all)]
pub async fn send_local_notifs(
  mentions: Vec<MentionData>,
  comment_id: CommentId,
  person: &Person,
  do_send_email: bool,
  context: &LemmyContext,
) -> LemmyResult<Vec<LocalUserId>> {
  let mut recipient_ids = Vec::new();
  let inbox_link = format!("{}/inbox", context.settings().get_protocol_and_hostname());

  // Read the comment view to get extra info
  let comment_view = CommentView::read(&mut context.pool(), comment_id, None)
    .await?
    .ok_or(LemmyErrorType::CouldntFindComment)?;
  let comment = comment_view.comment;
  let post = comment_view.post;
  let community = comment_view.community;

  // Send the local mentions
  for mention in mentions
    .iter()
    .filter(|m| m.is_local(&context.settings().hostname) && m.name.ne(&person.name))
  {
    let mention_name = mention.name.clone();
    let user_view = LocalUserView::read_from_name(&mut context.pool(), &mention_name).await;
    if let Ok(Some(mention_user_view)) = user_view {
      // TODO
      // At some point, make it so you can't tag the parent creator either
      // Potential duplication of notifications, one for reply and the other for mention, is handled
      // below by checking recipient ids
      recipient_ids.push(mention_user_view.local_user.id);

      let user_mention_form = PersonMentionInsertForm {
        recipient_id: mention_user_view.person.id,
        comment_id,
        read: None,
      };

      // Allow this to fail softly, since comment edits might re-update or replace it
      // Let the uniqueness handle this fail
      PersonMention::create(&mut context.pool(), &user_mention_form)
        .await
        .ok();

      // Send an email to those local users that have notifications on
      if do_send_email {
        let lang = get_interface_language(&mention_user_view);
        let content = markdown_to_html(&comment.content);
        send_email_to_user(
          &mention_user_view,
          &lang.notification_mentioned_by_subject(&person.name),
          &lang.notification_mentioned_by_body(&content, &inbox_link, &person.name),
          context.settings(),
        )
        .await
      }
    }
  }

  // Send comment_reply to the parent commenter / poster
  if let Some(parent_comment_id) = comment.parent_comment_id() {
    let parent_comment = Comment::read(&mut context.pool(), parent_comment_id)
      .await?
      .ok_or(LemmyErrorType::CouldntFindComment)?;

    // Get the parent commenter local_user
    let parent_creator_id = parent_comment.creator_id;

    let check_blocks = check_person_instance_community_block(
      person.id,
      parent_creator_id,
      // Only block from the community's instance_id
      community.instance_id,
      community.id,
      &mut context.pool(),
    )
    .await
    .is_err();

    // Don't send a notif to yourself
    if parent_comment.creator_id != person.id && !check_blocks {
      let user_view = LocalUserView::read_person(&mut context.pool(), parent_creator_id).await;
      if let Ok(Some(parent_user_view)) = user_view {
        // Don't duplicate notif if already mentioned by checking recipient ids
        if !recipient_ids.contains(&parent_user_view.local_user.id) {
          recipient_ids.push(parent_user_view.local_user.id);

          let comment_reply_form = CommentReplyInsertForm {
            recipient_id: parent_user_view.person.id,
            comment_id: comment.id,
            read: None,
          };

          // Allow this to fail softly, since comment edits might re-update or replace it
          // Let the uniqueness handle this fail
          CommentReply::create(&mut context.pool(), &comment_reply_form)
            .await
            .ok();

          if do_send_email {
            let lang = get_interface_language(&parent_user_view);
            let content = markdown_to_html(&comment.content);
            send_email_to_user(
              &parent_user_view,
              &lang.notification_comment_reply_subject(&person.name),
              &lang.notification_comment_reply_body(&content, &inbox_link, &person.name),
              context.settings(),
            )
            .await
          }
        }
      }
    }
  } else {
    // Use the post creator to check blocks
    let check_blocks = check_person_instance_community_block(
      person.id,
      post.creator_id,
      // Only block from the community's instance_id
      community.instance_id,
      community.id,
      &mut context.pool(),
    )
    .await
    .is_err();

    if post.creator_id != person.id && !check_blocks {
      let creator_id = post.creator_id;
      let parent_user = LocalUserView::read_person(&mut context.pool(), creator_id).await;
      if let Ok(Some(parent_user_view)) = parent_user {
        if !recipient_ids.contains(&parent_user_view.local_user.id) {
          recipient_ids.push(parent_user_view.local_user.id);

          let comment_reply_form = CommentReplyInsertForm {
            recipient_id: parent_user_view.person.id,
            comment_id: comment.id,
            read: None,
          };

          // Allow this to fail softly, since comment edits might re-update or replace it
          // Let the uniqueness handle this fail
          CommentReply::create(&mut context.pool(), &comment_reply_form)
            .await
            .ok();

          if do_send_email {
            let lang = get_interface_language(&parent_user_view);
            let content = markdown_to_html(&comment.content);
            send_email_to_user(
              &parent_user_view,
              &lang.notification_post_reply_subject(&person.name),
              &lang.notification_post_reply_body(&content, &inbox_link, &person.name),
              context.settings(),
            )
            .await
          }
        }
      }
    }
  }

  Ok(recipient_ids)
}
