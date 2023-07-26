use crate::PerformCrud;
use actix_web::web::Data;
use lemmy_api_common::{
  build_response::{build_comment_response, send_local_notifs},
  comment::{CommentResponse, CreateComment},
  context::LemmyContext,
  utils::{
    check_community_ban,
    check_community_deleted_or_removed,
    check_post_deleted_or_removed,
    generate_local_apub_endpoint,
    get_post,
    local_site_to_slur_regex,
    local_user_view_from_jwt,
    sanitize_html,
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
use lemmy_utils::{
  error::LemmyError,
  utils::{
    mention::scrape_text_for_mentions,
    slurs::remove_slurs,
    validation::is_valid_body_field,
  },
};
const MAX_COMMENT_DEPTH_LIMIT: usize = 100;

#[async_trait::async_trait(?Send)]
impl PerformCrud for CreateComment {
  type Response = CommentResponse;

  #[tracing::instrument(skip(context))]
  async fn perform(&self, context: &Data<LemmyContext>) -> Result<CommentResponse, LemmyError> {
    let data: &CreateComment = self;
    let local_user_view = local_user_view_from_jwt(&data.auth, context).await?;
    let local_site = LocalSite::read(context.pool()).await?;

    let content = remove_slurs(
      &data.content.clone(),
      &local_site_to_slur_regex(&local_site),
    );
    is_valid_body_field(&Some(content.clone()), false)?;
    let content = sanitize_html(&content);

    // Check for a community ban
    let post_id = data.post_id;
    let post = get_post(post_id, context.pool()).await?;
    let community_id = post.community_id;

    check_community_ban(local_user_view.person.id, community_id, context.pool()).await?;
    check_community_deleted_or_removed(community_id, context.pool()).await?;
    check_post_deleted_or_removed(&post)?;

    // Check if post is locked, no new comments
    if post.locked {
      return Err(LemmyError::from_message("locked"));
    }

    // Fetch the parent, if it exists
    let parent_opt = if let Some(parent_id) = data.parent_id {
      Comment::read(context.pool(), parent_id).await.ok()
    } else {
      None
    };

    // If there's a parent_id, check to make sure that comment is in that post
    // Strange issue where sometimes the post ID of the parent comment is incorrect
    if let Some(parent) = parent_opt.as_ref() {
      if parent.post_id != post_id {
        return Err(LemmyError::from_message("couldnt_create_comment"));
      }
      check_comment_depth(parent)?;
    }

    CommunityLanguage::is_allowed_community_language(
      context.pool(),
      data.language_id,
      community_id,
    )
    .await?;

    // attempt to set default language if none was provided
    let language_id = match data.language_id {
      Some(lid) => Some(lid),
      None => {
        default_post_language(context.pool(), community_id, local_user_view.local_user.id).await?
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
    let inserted_comment = Comment::create(context.pool(), &comment_form, parent_path.as_ref())
      .await
      .map_err(|e| LemmyError::from_error_message(e, "couldnt_create_comment"))?;

    // Necessary to update the ap_id
    let inserted_comment_id = inserted_comment.id;
    let protocol_and_hostname = context.settings().get_protocol_and_hostname();

    let apub_id = generate_local_apub_endpoint(
      EndpointType::Comment,
      &inserted_comment_id.to_string(),
      &protocol_and_hostname,
    )?;
    let updated_comment = Comment::update(
      context.pool(),
      inserted_comment_id,
      &CommentUpdateForm::builder().ap_id(Some(apub_id)).build(),
    )
    .await
    .map_err(|e| LemmyError::from_error_message(e, "couldnt_create_comment"))?;

    // Scan the comment for user mentions, add those rows
    let mentions = scrape_text_for_mentions(&content);
    let recipient_ids = send_local_notifs(
      mentions,
      &updated_comment,
      &local_user_view.person,
      &post,
      true,
      context,
    )
    .await?;

    // You like your own comment by default
    let like_form = CommentLikeForm {
      comment_id: inserted_comment.id,
      post_id: post.id,
      person_id: local_user_view.person.id,
      score: 1,
    };

    CommentLike::like(context.pool(), &like_form)
      .await
      .map_err(|e| LemmyError::from_error_message(e, "couldnt_like_comment"))?;

    // If its a reply, mark the parent as read
    if let Some(parent) = parent_opt {
      let parent_id = parent.id;
      let comment_reply = CommentReply::read_by_comment(context.pool(), parent_id).await;
      if let Ok(reply) = comment_reply {
        CommentReply::update(
          context.pool(),
          reply.id,
          &CommentReplyUpdateForm { read: Some(true) },
        )
        .await
        .map_err(|e| LemmyError::from_error_message(e, "couldnt_update_replies"))?;
      }

      // If the parent has PersonMentions mark them as read too
      let person_id = local_user_view.person.id;
      let person_mention =
        PersonMention::read_by_comment_and_person(context.pool(), parent_id, person_id).await;
      if let Ok(mention) = person_mention {
        PersonMention::update(
          context.pool(),
          mention.id,
          &PersonMentionUpdateForm { read: Some(true) },
        )
        .await
        .map_err(|e| LemmyError::from_error_message(e, "couldnt_update_person_mentions"))?;
      }
    }

    build_comment_response(
      context,
      inserted_comment.id,
      Some(local_user_view),
      self.form_id.clone(),
      recipient_ids,
    )
    .await
  }
}

pub fn check_comment_depth(comment: &Comment) -> Result<(), LemmyError> {
  let path = &comment.path.0;
  let length = path.split('.').count();
  if length > MAX_COMMENT_DEPTH_LIMIT {
    Err(LemmyError::from_message("max_comment_depth_reached"))
  } else {
    Ok(())
  }
}
