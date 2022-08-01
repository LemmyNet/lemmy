use crate::PerformCrud;
use actix_web::web::Data;
use lemmy_api_common::{
  comment::{CommentResponse, CreateComment},
  utils::{
    blocking,
    check_community_ban,
    check_community_deleted_or_removed,
    check_post_deleted_or_removed,
    get_local_user_view_from_jwt,
    get_post,
  },
};
use lemmy_apub::{
  generate_local_apub_endpoint,
  objects::comment::ApubComment,
  protocol::activities::{create_or_update::comment::CreateOrUpdateComment, CreateOrUpdateType},
  EndpointType,
};
use lemmy_db_schema::{
  source::{
    comment::{Comment, CommentForm, CommentLike, CommentLikeForm},
    comment_reply::CommentReply,
    person_mention::PersonMention,
  },
  traits::{Crud, Likeable},
};
use lemmy_utils::{
  error::LemmyError,
  utils::{remove_slurs, scrape_text_for_mentions},
  ConnectionId,
};
use lemmy_websocket::{
  send::{send_comment_ws_message, send_local_notifs},
  LemmyContext,
  UserOperationCrud,
};

#[async_trait::async_trait(?Send)]
impl PerformCrud for CreateComment {
  type Response = CommentResponse;

  #[tracing::instrument(skip(context, websocket_id))]
  async fn perform(
    &self,
    context: &Data<LemmyContext>,
    websocket_id: Option<ConnectionId>,
  ) -> Result<CommentResponse, LemmyError> {
    let data: &CreateComment = self;
    let local_user_view =
      get_local_user_view_from_jwt(&data.auth, context.pool(), context.secret()).await?;

    let content_slurs_removed =
      remove_slurs(&data.content.to_owned(), &context.settings().slur_regex());

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
      blocking(context.pool(), move |conn| Comment::read(conn, parent_id))
        .await?
        .ok()
    } else {
      None
    };

    // If there's a parent_id, check to make sure that comment is in that post
    // Strange issue where sometimes the post ID of the parent comment is incorrect
    if let Some(parent) = parent_opt.as_ref() {
      if parent.post_id != post_id {
        return Err(LemmyError::from_message("couldnt_create_comment"));
      }
    }

    let comment_form = CommentForm {
      content: content_slurs_removed,
      post_id: data.post_id,
      creator_id: local_user_view.person.id,
      ..CommentForm::default()
    };

    // Create the comment
    let comment_form2 = comment_form.clone();
    let parent_path = parent_opt.to_owned().map(|t| t.path);
    let inserted_comment = blocking(context.pool(), move |conn| {
      Comment::create(conn, &comment_form2, parent_path.as_ref())
    })
    .await?
    .map_err(|e| LemmyError::from_error_message(e, "couldnt_create_comment"))?;

    // Necessary to update the ap_id
    let inserted_comment_id = inserted_comment.id;
    let protocol_and_hostname = context.settings().get_protocol_and_hostname();

    let updated_comment: Comment =
      blocking(context.pool(), move |conn| -> Result<Comment, LemmyError> {
        let apub_id = generate_local_apub_endpoint(
          EndpointType::Comment,
          &inserted_comment_id.to_string(),
          &protocol_and_hostname,
        )?;
        Ok(Comment::update_ap_id(conn, inserted_comment_id, apub_id)?)
      })
      .await?
      .map_err(|e| e.with_message("couldnt_create_comment"))?;

    // Scan the comment for user mentions, add those rows
    let post_id = post.id;
    let mentions = scrape_text_for_mentions(&comment_form.content);
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
      post_id,
      person_id: local_user_view.person.id,
      score: 1,
    };

    let like = move |conn: &'_ _| CommentLike::like(conn, &like_form);
    blocking(context.pool(), like)
      .await?
      .map_err(|e| LemmyError::from_error_message(e, "couldnt_like_comment"))?;

    let apub_comment: ApubComment = updated_comment.into();
    CreateOrUpdateComment::send(
      apub_comment.clone(),
      &local_user_view.person.clone().into(),
      CreateOrUpdateType::Create,
      context,
      &mut 0,
    )
    .await?;

    // If its a reply, mark the parent as read
    if let Some(parent) = parent_opt {
      let parent_id = parent.id;
      let comment_reply = blocking(context.pool(), move |conn| {
        CommentReply::read_by_comment(conn, parent_id)
      })
      .await?;
      if let Ok(reply) = comment_reply {
        blocking(context.pool(), move |conn| {
          CommentReply::update_read(conn, reply.id, true)
        })
        .await?
        .map_err(|e| LemmyError::from_error_message(e, "couldnt_update_replies"))?;
      }

      // If the parent has PersonMentions mark them as read too
      let person_id = local_user_view.person.id;
      let person_mention = blocking(context.pool(), move |conn| {
        PersonMention::read_by_comment_and_person(conn, parent_id, person_id)
      })
      .await?;
      if let Ok(mention) = person_mention {
        blocking(context.pool(), move |conn| {
          PersonMention::update_read(conn, mention.id, true)
        })
        .await?
        .map_err(|e| LemmyError::from_error_message(e, "couldnt_update_person_mentions"))?;
      }
    }

    send_comment_ws_message(
      inserted_comment.id,
      UserOperationCrud::CreateComment,
      websocket_id,
      data.form_id.to_owned(),
      Some(local_user_view.person.id),
      recipient_ids,
      context,
    )
    .await
  }
}
