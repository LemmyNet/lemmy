use crate::{
  comment::CommentResponse,
  community::CommunityResponse,
  context::LemmyContext,
  post::PostResponse,
  utils::{check_person_block, get_interface_language, is_mod_or_admin, send_email_to_user},
};
use actix_web::web::Data;
use lemmy_db_schema::{
  newtypes::{CommentId, CommunityId, LocalUserId, PersonId, PostId},
  source::{
    actor_language::CommunityLanguage,
    comment::Comment,
    comment_reply::{CommentReply, CommentReplyInsertForm},
    person::Person,
    person_mention::{PersonMention, PersonMentionInsertForm},
    post::Post,
  },
  traits::Crud,
};
use lemmy_db_views::structs::{CommentView, LocalUserView, PostView};
use lemmy_db_views_actor::structs::CommunityView;
use lemmy_utils::{error::LemmyError, utils::mention::MentionData};

pub async fn build_comment_response(
  context: &Data<LemmyContext>,
  comment_id: CommentId,
  local_user_view: Option<LocalUserView>,
  form_id: Option<String>,
  recipient_ids: Vec<LocalUserId>,
) -> Result<CommentResponse, LemmyError> {
  let person_id = local_user_view.map(|l| l.person.id);
  let comment_view = CommentView::read(context.pool(), comment_id, person_id).await?;
  Ok(CommentResponse {
    comment_view,
    recipient_ids,
    form_id,
  })
}

pub async fn build_community_response(
  context: &Data<LemmyContext>,
  local_user_view: LocalUserView,
  community_id: CommunityId,
) -> Result<CommunityResponse, LemmyError> {
  let is_mod_or_admin = is_mod_or_admin(context.pool(), local_user_view.person.id, community_id)
    .await
    .is_ok();
  let person_id = local_user_view.person.id;
  let community_view = CommunityView::read(
    context.pool(),
    community_id,
    Some(person_id),
    Some(is_mod_or_admin),
  )
  .await?;
  let discussion_languages = CommunityLanguage::read(context.pool(), community_id).await?;

  Ok(CommunityResponse {
    community_view,
    discussion_languages,
  })
}

pub async fn build_post_response(
  context: &Data<LemmyContext>,
  community_id: CommunityId,
  person_id: PersonId,
  post_id: PostId,
) -> Result<PostResponse, LemmyError> {
  let is_mod_or_admin = is_mod_or_admin(context.pool(), person_id, community_id)
    .await
    .is_ok();
  let post_view = PostView::read(
    context.pool(),
    post_id,
    Some(person_id),
    Some(is_mod_or_admin),
  )
  .await?;
  Ok(PostResponse { post_view })
}

// TODO: this function is a mess and should be split up to handle email seperately
#[tracing::instrument(skip_all)]
pub async fn send_local_notifs(
  mentions: Vec<MentionData>,
  comment: &Comment,
  person: &Person,
  post: &Post,
  do_send_email: bool,
  context: &LemmyContext,
) -> Result<Vec<LocalUserId>, LemmyError> {
  let mut recipient_ids = Vec::new();
  let inbox_link = format!("{}/inbox", context.settings().get_protocol_and_hostname());

  // Send the local mentions
  for mention in mentions
    .iter()
    .filter(|m| m.is_local(&context.settings().hostname) && m.name.ne(&person.name))
    .collect::<Vec<&MentionData>>()
  {
    let mention_name = mention.name.clone();
    let user_view = LocalUserView::read_from_name(context.pool(), &mention_name).await;
    if let Ok(mention_user_view) = user_view {
      // TODO
      // At some point, make it so you can't tag the parent creator either
      // This can cause two notifications, one for reply and the other for mention
      recipient_ids.push(mention_user_view.local_user.id);

      let user_mention_form = PersonMentionInsertForm {
        recipient_id: mention_user_view.person.id,
        comment_id: comment.id,
        read: None,
      };

      // Allow this to fail softly, since comment edits might re-update or replace it
      // Let the uniqueness handle this fail
      PersonMention::create(context.pool(), &user_mention_form)
        .await
        .ok();

      // Send an email to those local users that have notifications on
      if do_send_email {
        let lang = get_interface_language(&mention_user_view);
        send_email_to_user(
          &mention_user_view,
          &lang.notification_mentioned_by_subject(&person.name),
          &lang.notification_mentioned_by_body(&comment.content, &inbox_link, &person.name),
          context.settings(),
        )
      }
    }
  }

  // Send comment_reply to the parent commenter / poster
  if let Some(parent_comment_id) = comment.parent_comment_id() {
    let parent_comment = Comment::read(context.pool(), parent_comment_id).await?;

    // Get the parent commenter local_user
    let parent_creator_id = parent_comment.creator_id;

    // Only add to recipients if that person isn't blocked
    let creator_blocked = check_person_block(person.id, parent_creator_id, context.pool())
      .await
      .is_err();

    // Don't send a notif to yourself
    if parent_comment.creator_id != person.id && !creator_blocked {
      let user_view = LocalUserView::read_person(context.pool(), parent_creator_id).await;
      if let Ok(parent_user_view) = user_view {
        recipient_ids.push(parent_user_view.local_user.id);

        let comment_reply_form = CommentReplyInsertForm {
          recipient_id: parent_user_view.person.id,
          comment_id: comment.id,
          read: None,
        };

        // Allow this to fail softly, since comment edits might re-update or replace it
        // Let the uniqueness handle this fail
        CommentReply::create(context.pool(), &comment_reply_form)
          .await
          .ok();

        if do_send_email {
          let lang = get_interface_language(&parent_user_view);
          send_email_to_user(
            &parent_user_view,
            &lang.notification_comment_reply_subject(&person.name),
            &lang.notification_comment_reply_body(&comment.content, &inbox_link, &person.name),
            context.settings(),
          )
        }
      }
    }
  } else {
    // If there's no parent, its the post creator
    // Only add to recipients if that person isn't blocked
    let creator_blocked = check_person_block(person.id, post.creator_id, context.pool())
      .await
      .is_err();

    if post.creator_id != person.id && !creator_blocked {
      let creator_id = post.creator_id;
      let parent_user = LocalUserView::read_person(context.pool(), creator_id).await;
      if let Ok(parent_user_view) = parent_user {
        recipient_ids.push(parent_user_view.local_user.id);

        let comment_reply_form = CommentReplyInsertForm {
          recipient_id: parent_user_view.person.id,
          comment_id: comment.id,
          read: None,
        };

        // Allow this to fail softly, since comment edits might re-update or replace it
        // Let the uniqueness handle this fail
        CommentReply::create(context.pool(), &comment_reply_form)
          .await
          .ok();

        if do_send_email {
          let lang = get_interface_language(&parent_user_view);
          send_email_to_user(
            &parent_user_view,
            &lang.notification_post_reply_subject(&person.name),
            &lang.notification_post_reply_body(&comment.content, &inbox_link, &person.name),
            context.settings(),
          )
        }
      }
    }
  }

  Ok(recipient_ids)
}
