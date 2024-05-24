use activitypub_federation::config::Data;
use actix_web::web::Json;
use lemmy_api_common::{
  build_response::{build_comment_response, send_local_notifs},
  comment::{CommentResponse, CreateComment},
  context::LemmyContext,
  send_activity::{ActivityChannel, SendActivityData},
  utils::{
    check_community_user_action,
    check_post_deleted_or_removed,
    generate_local_apub_endpoint,
    get_url_blocklist,
    is_mod_or_admin,
    local_site_to_slur_regex,
    process_markdown,
    update_read_comments,
    EndpointType,
  },
};
use lemmy_db_schema::{
  impls::actor_language::default_post_language,
  source::{
    actor_language::CommunityLanguage,
    comment::{Comment, CommentInsertForm, CommentLike, CommentLikeForm, CommentUpdateForm},
    comment_reply::{CommentReply, CommentReplyUpdateForm},
    local_site::LocalSite,
    person_mention::{PersonMention, PersonMentionUpdateForm},
  },
  traits::{Crud, Likeable},
};
use lemmy_db_views::structs::{LocalUserView, PostView};
use lemmy_utils::{
  error::{LemmyErrorExt, LemmyErrorType, LemmyResult},
  utils::{mention::scrape_text_for_mentions, validation::is_valid_body_field},
};

const MAX_COMMENT_DEPTH_LIMIT: usize = 100;

#[tracing::instrument(skip(context))]
pub async fn create_comment(
  data: Json<CreateComment>,
  context: Data<LemmyContext>,
  local_user_view: LocalUserView,
) -> LemmyResult<Json<CommentResponse>> {
  let local_site = LocalSite::read(&mut context.pool()).await?;

  let slur_regex = local_site_to_slur_regex(&local_site);
  let url_blocklist = get_url_blocklist(&context).await?;
  let content = process_markdown(&data.content, &slur_regex, &url_blocklist, &context).await?;
  is_valid_body_field(&Some(content.clone()), false)?;

  // Check for a community ban
  let post_id = data.post_id;

  // Read the full post view in order to get the comments count.
  let post_view = PostView::read(
    &mut context.pool(),
    post_id,
    Some(local_user_view.person.id),
    true,
  )
  .await?
  .ok_or(LemmyErrorType::CouldntFindPost)?;

  let post = post_view.post;
  let community_id = post_view.community.id;

  check_community_user_action(&local_user_view.person, community_id, &mut context.pool()).await?;
  check_post_deleted_or_removed(&post)?;

  // Check if post is locked, no new comments
  let is_mod_or_admin = is_mod_or_admin(&mut context.pool(), &local_user_view.person, community_id)
    .await
    .is_ok();
  if post.locked && !is_mod_or_admin {
    Err(LemmyErrorType::Locked)?
  }

  // Fetch the parent, if it exists
  let parent_opt = if let Some(parent_id) = data.parent_id {
    Comment::read(&mut context.pool(), parent_id).await.ok()
  } else {
    None
  }
  .flatten();

  // If there's a parent_id, check to make sure that comment is in that post
  // Strange issue where sometimes the post ID of the parent comment is incorrect
  if let Some(parent) = parent_opt.as_ref() {
    if parent.post_id != post_id {
      Err(LemmyErrorType::CouldntCreateComment)?
    }
    check_comment_depth(parent)?;
  }

  CommunityLanguage::is_allowed_community_language(
    &mut context.pool(),
    data.language_id,
    community_id,
  )
  .await?;

  // attempt to set default language if none was provided
  let language_id = match data.language_id {
    Some(lid) => Some(lid),
    None => {
      default_post_language(
        &mut context.pool(),
        community_id,
        local_user_view.local_user.id,
      )
      .await?
    }
  };

  let comment_form = CommentInsertForm::builder()
    .content(content.clone())
    .post_id(data.post_id)
    .creator_id(local_user_view.person.id)
    .language_id(language_id)
    .build();

  // Create the comment
  let parent_path = parent_opt.clone().map(|t| t.path);
  let inserted_comment = Comment::create(&mut context.pool(), &comment_form, parent_path.as_ref())
    .await
    .with_lemmy_type(LemmyErrorType::CouldntCreateComment)?;

  // Necessary to update the ap_id
  let inserted_comment_id = inserted_comment.id;
  let protocol_and_hostname = context.settings().get_protocol_and_hostname();

  let apub_id = generate_local_apub_endpoint(
    EndpointType::Comment,
    &inserted_comment_id.to_string(),
    &protocol_and_hostname,
  )?;
  let updated_comment = Comment::update(
    &mut context.pool(),
    inserted_comment_id,
    &CommentUpdateForm {
      ap_id: Some(apub_id),
      ..Default::default()
    },
  )
  .await
  .with_lemmy_type(LemmyErrorType::CouldntCreateComment)?;

  // Scan the comment for user mentions, add those rows
  let mentions = scrape_text_for_mentions(&content);
  let recipient_ids = send_local_notifs(
    mentions,
    inserted_comment_id,
    &local_user_view.person,
    true,
    &context,
  )
  .await?;

  // You like your own comment by default
  let like_form = CommentLikeForm {
    comment_id: inserted_comment.id,
    post_id: post.id,
    person_id: local_user_view.person.id,
    score: 1,
  };

  CommentLike::like(&mut context.pool(), &like_form)
    .await
    .with_lemmy_type(LemmyErrorType::CouldntLikeComment)?;

  ActivityChannel::submit_activity(
    SendActivityData::CreateComment(updated_comment.clone()),
    &context,
  )
  .await?;

  // Update the read comments, so your own new comment doesn't appear as a +1 unread
  update_read_comments(
    local_user_view.person.id,
    post_id,
    post_view.counts.comments + 1,
    &mut context.pool(),
  )
  .await?;

  // If we're responding to a comment where we're the recipient,
  // (ie we're the grandparent, or the recipient of the parent comment_reply),
  // then mark the parent as read.
  // Then we don't have to do it manually after we respond to a comment.
  if let Some(parent) = parent_opt {
    let person_id = local_user_view.person.id;
    let parent_id = parent.id;
    let comment_reply =
      CommentReply::read_by_comment_and_person(&mut context.pool(), parent_id, person_id).await;
    if let Ok(Some(reply)) = comment_reply {
      CommentReply::update(
        &mut context.pool(),
        reply.id,
        &CommentReplyUpdateForm { read: Some(true) },
      )
      .await
      .with_lemmy_type(LemmyErrorType::CouldntUpdateReplies)?;
    }

    // If the parent has PersonMentions mark them as read too
    let person_mention =
      PersonMention::read_by_comment_and_person(&mut context.pool(), parent_id, person_id).await;
    if let Ok(Some(mention)) = person_mention {
      PersonMention::update(
        &mut context.pool(),
        mention.id,
        &PersonMentionUpdateForm { read: Some(true) },
      )
      .await
      .with_lemmy_type(LemmyErrorType::CouldntUpdatePersonMentions)?;
    }
  }

  Ok(Json(
    build_comment_response(
      &context,
      inserted_comment.id,
      Some(local_user_view),
      recipient_ids,
    )
    .await?,
  ))
}

pub fn check_comment_depth(comment: &Comment) -> LemmyResult<()> {
  let path = &comment.path.0;
  let length = path.split('.').count();
  if length > MAX_COMMENT_DEPTH_LIMIT {
    Err(LemmyErrorType::MaxCommentDepthReached)?
  } else {
    Ok(())
  }
}
