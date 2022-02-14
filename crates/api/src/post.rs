use crate::Perform;
use actix_web::web::Data;
use lemmy_api_common::{
  blocking,
  check_community_ban,
  check_community_deleted_or_removed,
  check_downvotes_enabled,
  get_local_user_view_from_jwt,
  is_mod_or_admin,
  mark_post_as_read,
  mark_post_as_unread,
  post::*,
};
use lemmy_apub::{
  fetcher::post_or_comment::PostOrComment,
  objects::post::ApubPost,
  protocol::activities::{
    create_or_update::post::CreateOrUpdatePost,
    voting::{
      undo_vote::UndoVote,
      vote::{Vote, VoteType},
    },
    CreateOrUpdateType,
  },
};
use lemmy_db_schema::{
  source::{moderator::*, post::*},
  traits::{Crud, Likeable, Saveable},
};
use lemmy_db_views::post_view::PostView;
use lemmy_utils::{request::fetch_site_metadata, ConnectionId, LemmyError};
use lemmy_websocket::{send::send_post_ws_message, LemmyContext, UserOperation};
use std::convert::TryInto;

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
      let like = move |conn: &'_ _| PostLike::like(conn, &like_form2);
      blocking(context.pool(), like)
        .await?
        .map_err(LemmyError::from)
        .map_err(|e| e.with_message("couldnt_like_post"))?;

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

#[async_trait::async_trait(?Send)]
impl Perform for MarkPostAsRead {
  type Response = PostResponse;

  #[tracing::instrument(skip(context, _websocket_id))]
  async fn perform(
    &self,
    context: &Data<LemmyContext>,
    _websocket_id: Option<ConnectionId>,
  ) -> Result<Self::Response, LemmyError> {
    let data = self;
    let local_user_view =
      get_local_user_view_from_jwt(&data.auth, context.pool(), context.secret()).await?;

    let post_id = data.post_id;
    let person_id = local_user_view.person.id;

    // Mark the post as read / unread
    if data.read {
      mark_post_as_read(person_id, post_id, context.pool()).await?;
    } else {
      mark_post_as_unread(person_id, post_id, context.pool()).await?;
    }

    // Fetch it
    let post_view = blocking(context.pool(), move |conn| {
      PostView::read(conn, post_id, Some(person_id))
    })
    .await??;

    let res = Self::Response { post_view };

    Ok(res)
  }
}

#[async_trait::async_trait(?Send)]
impl Perform for LockPost {
  type Response = PostResponse;

  #[tracing::instrument(skip(context, websocket_id))]
  async fn perform(
    &self,
    context: &Data<LemmyContext>,
    websocket_id: Option<ConnectionId>,
  ) -> Result<PostResponse, LemmyError> {
    let data: &LockPost = self;
    let local_user_view =
      get_local_user_view_from_jwt(&data.auth, context.pool(), context.secret()).await?;

    let post_id = data.post_id;
    let orig_post = blocking(context.pool(), move |conn| Post::read(conn, post_id)).await??;

    check_community_ban(
      local_user_view.person.id,
      orig_post.community_id,
      context.pool(),
    )
    .await?;
    check_community_deleted_or_removed(orig_post.community_id, context.pool()).await?;

    // Verify that only the mods can lock
    is_mod_or_admin(
      context.pool(),
      local_user_view.person.id,
      orig_post.community_id,
    )
    .await?;

    // Update the post
    let post_id = data.post_id;
    let locked = data.locked;
    let updated_post: ApubPost = blocking(context.pool(), move |conn| {
      Post::update_locked(conn, post_id, locked)
    })
    .await??
    .into();

    // Mod tables
    let form = ModLockPostForm {
      mod_person_id: local_user_view.person.id,
      post_id: data.post_id,
      locked: Some(locked),
    };
    blocking(context.pool(), move |conn| ModLockPost::create(conn, &form)).await??;

    // apub updates
    CreateOrUpdatePost::send(
      updated_post,
      &local_user_view.person.clone().into(),
      CreateOrUpdateType::Update,
      context,
    )
    .await?;

    send_post_ws_message(
      data.post_id,
      UserOperation::LockPost,
      websocket_id,
      Some(local_user_view.person.id),
      context,
    )
    .await
  }
}

#[async_trait::async_trait(?Send)]
impl Perform for StickyPost {
  type Response = PostResponse;

  #[tracing::instrument(skip(context, websocket_id))]
  async fn perform(
    &self,
    context: &Data<LemmyContext>,
    websocket_id: Option<ConnectionId>,
  ) -> Result<PostResponse, LemmyError> {
    let data: &StickyPost = self;
    let local_user_view =
      get_local_user_view_from_jwt(&data.auth, context.pool(), context.secret()).await?;

    let post_id = data.post_id;
    let orig_post = blocking(context.pool(), move |conn| Post::read(conn, post_id)).await??;

    check_community_ban(
      local_user_view.person.id,
      orig_post.community_id,
      context.pool(),
    )
    .await?;
    check_community_deleted_or_removed(orig_post.community_id, context.pool()).await?;

    // Verify that only the mods can sticky
    is_mod_or_admin(
      context.pool(),
      local_user_view.person.id,
      orig_post.community_id,
    )
    .await?;

    // Update the post
    let post_id = data.post_id;
    let stickied = data.stickied;
    let updated_post: ApubPost = blocking(context.pool(), move |conn| {
      Post::update_stickied(conn, post_id, stickied)
    })
    .await??
    .into();

    // Mod tables
    let form = ModStickyPostForm {
      mod_person_id: local_user_view.person.id,
      post_id: data.post_id,
      stickied: Some(stickied),
    };
    blocking(context.pool(), move |conn| {
      ModStickyPost::create(conn, &form)
    })
    .await??;

    // Apub updates
    // TODO stickied should pry work like locked for ease of use
    CreateOrUpdatePost::send(
      updated_post,
      &local_user_view.person.clone().into(),
      CreateOrUpdateType::Update,
      context,
    )
    .await?;

    send_post_ws_message(
      data.post_id,
      UserOperation::StickyPost,
      websocket_id,
      Some(local_user_view.person.id),
      context,
    )
    .await
  }
}

#[async_trait::async_trait(?Send)]
impl Perform for SavePost {
  type Response = PostResponse;

  #[tracing::instrument(skip(context, _websocket_id))]
  async fn perform(
    &self,
    context: &Data<LemmyContext>,
    _websocket_id: Option<ConnectionId>,
  ) -> Result<PostResponse, LemmyError> {
    let data: &SavePost = self;
    let local_user_view =
      get_local_user_view_from_jwt(&data.auth, context.pool(), context.secret()).await?;

    let post_saved_form = PostSavedForm {
      post_id: data.post_id,
      person_id: local_user_view.person.id,
    };

    if data.save {
      let save = move |conn: &'_ _| PostSaved::save(conn, &post_saved_form);
      blocking(context.pool(), save)
        .await?
        .map_err(LemmyError::from)
        .map_err(|e| e.with_message("couldnt_save_post"))?;
    } else {
      let unsave = move |conn: &'_ _| PostSaved::unsave(conn, &post_saved_form);
      blocking(context.pool(), unsave)
        .await?
        .map_err(LemmyError::from)
        .map_err(|e| e.with_message("couldnt_save_post"))?;
    }

    let post_id = data.post_id;
    let person_id = local_user_view.person.id;
    let post_view = blocking(context.pool(), move |conn| {
      PostView::read(conn, post_id, Some(person_id))
    })
    .await??;

    // Mark the post as read
    mark_post_as_read(person_id, post_id, context.pool()).await?;

    Ok(PostResponse { post_view })
  }
}

#[async_trait::async_trait(?Send)]
impl Perform for GetSiteMetadata {
  type Response = GetSiteMetadataResponse;

  #[tracing::instrument(skip(context, _websocket_id))]
  async fn perform(
    &self,
    context: &Data<LemmyContext>,
    _websocket_id: Option<ConnectionId>,
  ) -> Result<GetSiteMetadataResponse, LemmyError> {
    let data: &Self = self;

    let metadata = fetch_site_metadata(context.client(), &data.url).await?;

    Ok(GetSiteMetadataResponse { metadata })
  }
}
