use crate::Perform;
use actix_web::web::Data;
use lemmy_api_common::{
  post::{CreatePostLike, PostResponse},
  utils::{
    blocking,
    check_community_ban,
    check_community_deleted_or_removed,
    check_downvotes_enabled,
    get_local_user_view_from_jwt,
    mark_post_as_read,
  },
};
use lemmy_apub::{
  fetcher::post_or_comment::PostOrComment,
  objects::post::ApubPost,
  protocol::activities::voting::{
    undo_vote::UndoVote,
    vote::{Vote, VoteType},
  },
};
use lemmy_db_schema::{
  source::post::{Post, PostLike, PostLikeForm},
  traits::{Crud, Likeable},
};
use lemmy_utils::{error::LemmyError, ConnectionId};
use lemmy_websocket::{send::send_post_ws_message, LemmyContext, UserOperation};

#[async_trait::async_trait(?Send)]
impl Perform for CreatePostLike {
  type Response = PostResponse;

  #[tracing::instrument(skip(context, websocket_id))]
  async fn perform(
    &self,
    context: &Data<LemmyContext>,
    websocket_id: Option<ConnectionId>,
  ) -> Result<PostResponse, LemmyError> {
    let data: &CreatePostLike = self;
    let local_user_view =
      get_local_user_view_from_jwt(&data.auth, context.pool(), context.secret()).await?;

    // Don't do a downvote if site has downvotes disabled
    check_downvotes_enabled(data.score, context.pool()).await?;

    // Check for a community ban
    let post_id = data.post_id;
    let post: ApubPost = blocking(context.pool(), move |conn| Post::read(conn, post_id))
      .await??
      .into();

    check_community_ban(local_user_view.person.id, post.community_id, context.pool()).await?;
    check_community_deleted_or_removed(post.community_id, context.pool()).await?;

    let like_form = PostLikeForm {
      post_id: data.post_id,
      person_id: local_user_view.person.id,
      score: data.score,
    };

    // Remove any likes first
    let person_id = local_user_view.person.id;
    blocking(context.pool(), move |conn| {
      PostLike::remove(conn, person_id, post_id)
    })
    .await??;

    let community_id = post.community_id;
    let object = PostOrComment::Post(Box::new(post));

    // Only add the like if the score isnt 0
    let do_add = like_form.score != 0 && (like_form.score == 1 || like_form.score == -1);
    if do_add {
      let like_form2 = like_form.clone();
      let like = move |conn: &mut _| PostLike::like(conn, &like_form2);
      blocking(context.pool(), like)
        .await?
        .map_err(|e| LemmyError::from_error_message(e, "couldnt_like_post"))?;

      Vote::send(
        &object,
        &local_user_view.person.clone().into(),
        community_id,
        like_form.score.try_into()?,
        context,
      )
      .await?;
    } else {
      // API doesn't distinguish between Undo/Like and Undo/Dislike
      UndoVote::send(
        &object,
        &local_user_view.person.clone().into(),
        community_id,
        VoteType::Like,
        context,
      )
      .await?;
    }

    // Mark the post as read
    mark_post_as_read(person_id, post_id, context.pool()).await?;

    send_post_ws_message(
      data.post_id,
      UserOperation::CreatePostLike,
      websocket_id,
      Some(local_user_view.person.id),
      context,
    )
    .await
  }
}
