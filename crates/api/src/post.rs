use crate::{
  check_community_ban,
  check_downvotes_enabled,
  check_optional_url,
  collect_moderated_communities,
  get_user_from_jwt,
  get_user_from_jwt_opt,
  is_mod_or_admin,
  Perform,
};
use actix_web::web::Data;
use lemmy_apub::{generate_apub_endpoint, ApubLikeableType, ApubObjectType, EndpointType};
use lemmy_db_queries::{
  source::post::Post_,
  Crud,
  Likeable,
  ListingType,
  Reportable,
  Saveable,
  SortType,
};
use lemmy_db_schema::{
  naive_now,
  source::{
    moderator::*,
    post::*,
    post_report::{PostReport, PostReportForm},
  },
};
use lemmy_db_views::{
  comment_view::CommentQueryBuilder,
  post_report_view::{PostReportQueryBuilder, PostReportView},
  post_view::{PostQueryBuilder, PostView},
};
use lemmy_db_views_actor::{
  community_moderator_view::CommunityModeratorView,
  community_view::CommunityView,
};
use lemmy_structs::{blocking, post::*};
use lemmy_utils::{
  request::fetch_iframely_and_pictrs_data,
  utils::{check_slurs, check_slurs_opt, is_valid_post_title},
  ApiError,
  ConnectionId,
  LemmyError,
};
use lemmy_websocket::{
  messages::{GetPostUsersOnline, SendModRoomMessage, SendPost, SendUserRoomMessage},
  LemmyContext,
  UserOperation,
};
use std::str::FromStr;

#[async_trait::async_trait(?Send)]
impl Perform for CreatePost {
  type Response = PostResponse;

  async fn perform(
    &self,
    context: &Data<LemmyContext>,
    websocket_id: Option<ConnectionId>,
  ) -> Result<PostResponse, LemmyError> {
    let data: &CreatePost = &self;
    let user = get_user_from_jwt(&data.auth, context.pool()).await?;

    check_slurs(&data.name)?;
    check_slurs_opt(&data.body)?;

    if !is_valid_post_title(&data.name) {
      return Err(ApiError::err("invalid_post_title").into());
    }

    check_community_ban(user.id, data.community_id, context.pool()).await?;

    check_optional_url(&Some(data.url.to_owned()))?;

    // Fetch Iframely and pictrs cached image
    let (iframely_title, iframely_description, iframely_html, pictrs_thumbnail) =
      fetch_iframely_and_pictrs_data(context.client(), data.url.to_owned()).await;

    let post_form = PostForm {
      name: data.name.trim().to_owned(),
      url: data.url.to_owned(),
      body: data.body.to_owned(),
      community_id: data.community_id,
      creator_id: user.id,
      removed: None,
      deleted: None,
      nsfw: data.nsfw,
      locked: None,
      stickied: None,
      updated: None,
      embed_title: iframely_title,
      embed_description: iframely_description,
      embed_html: iframely_html,
      thumbnail_url: pictrs_thumbnail,
      ap_id: None,
      local: true,
      published: None,
    };

    let inserted_post =
      match blocking(context.pool(), move |conn| Post::create(conn, &post_form)).await? {
        Ok(post) => post,
        Err(e) => {
          let err_type = if e.to_string() == "value too long for type character varying(200)" {
            "post_title_too_long"
          } else {
            "couldnt_create_post"
          };

          return Err(ApiError::err(err_type).into());
        }
      };

    let inserted_post_id = inserted_post.id;
    let updated_post = match blocking(context.pool(), move |conn| -> Result<Post, LemmyError> {
      let apub_id = generate_apub_endpoint(EndpointType::Post, &inserted_post_id.to_string())?;
      Ok(Post::update_ap_id(conn, inserted_post_id, apub_id)?)
    })
    .await?
    {
      Ok(post) => post,
      Err(_e) => return Err(ApiError::err("couldnt_create_post").into()),
    };

    updated_post.send_create(&user, context).await?;

    // They like their own post by default
    let like_form = PostLikeForm {
      post_id: inserted_post.id,
      user_id: user.id,
      score: 1,
    };

    let like = move |conn: &'_ _| PostLike::like(conn, &like_form);
    if blocking(context.pool(), like).await?.is_err() {
      return Err(ApiError::err("couldnt_like_post").into());
    }

    updated_post.send_like(&user, context).await?;

    // Refetch the view
    let inserted_post_id = inserted_post.id;
    let post_view = match blocking(context.pool(), move |conn| {
      PostView::read(conn, inserted_post_id, Some(user.id))
    })
    .await?
    {
      Ok(post) => post,
      Err(_e) => return Err(ApiError::err("couldnt_find_post").into()),
    };

    let res = PostResponse { post_view };

    context.chat_server().do_send(SendPost {
      op: UserOperation::CreatePost,
      post: res.clone(),
      websocket_id,
    });

    Ok(res)
  }
}

#[async_trait::async_trait(?Send)]
impl Perform for GetPost {
  type Response = GetPostResponse;

  async fn perform(
    &self,
    context: &Data<LemmyContext>,
    _websocket_id: Option<ConnectionId>,
  ) -> Result<GetPostResponse, LemmyError> {
    let data: &GetPost = &self;
    let user = get_user_from_jwt_opt(&data.auth, context.pool()).await?;
    let user_id = user.map(|u| u.id);

    let id = data.id;
    let post_view = match blocking(context.pool(), move |conn| {
      PostView::read(conn, id, user_id)
    })
    .await?
    {
      Ok(post) => post,
      Err(_e) => return Err(ApiError::err("couldnt_find_post").into()),
    };

    let id = data.id;
    let comments = blocking(context.pool(), move |conn| {
      CommentQueryBuilder::create(conn)
        .my_user_id(user_id)
        .post_id(id)
        .limit(9999)
        .list()
    })
    .await??;

    let community_id = post_view.community.id;
    let moderators = blocking(context.pool(), move |conn| {
      CommunityModeratorView::for_community(conn, community_id)
    })
    .await??;

    // Necessary for the sidebar
    let community_view = match blocking(context.pool(), move |conn| {
      CommunityView::read(conn, community_id, user_id)
    })
    .await?
    {
      Ok(community) => community,
      Err(_e) => return Err(ApiError::err("couldnt_find_community").into()),
    };

    let online = context
      .chat_server()
      .send(GetPostUsersOnline { post_id: data.id })
      .await
      .unwrap_or(1);

    // Return the jwt
    Ok(GetPostResponse {
      post_view,
      community_view,
      comments,
      moderators,
      online,
    })
  }
}

#[async_trait::async_trait(?Send)]
impl Perform for GetPosts {
  type Response = GetPostsResponse;

  async fn perform(
    &self,
    context: &Data<LemmyContext>,
    _websocket_id: Option<ConnectionId>,
  ) -> Result<GetPostsResponse, LemmyError> {
    let data: &GetPosts = &self;
    let user = get_user_from_jwt_opt(&data.auth, context.pool()).await?;

    let user_id = match &user {
      Some(user) => Some(user.id),
      None => None,
    };

    let show_nsfw = match &user {
      Some(user) => user.show_nsfw,
      None => false,
    };

    let type_ = ListingType::from_str(&data.type_)?;
    let sort = SortType::from_str(&data.sort)?;

    let page = data.page;
    let limit = data.limit;
    let community_id = data.community_id;
    let community_name = data.community_name.to_owned();
    let posts = match blocking(context.pool(), move |conn| {
      PostQueryBuilder::create(conn)
        .listing_type(&type_)
        .sort(&sort)
        .show_nsfw(show_nsfw)
        .community_id(community_id)
        .community_name(community_name)
        .my_user_id(user_id)
        .page(page)
        .limit(limit)
        .list()
    })
    .await?
    {
      Ok(posts) => posts,
      Err(_e) => return Err(ApiError::err("couldnt_get_posts").into()),
    };

    Ok(GetPostsResponse { posts })
  }
}

#[async_trait::async_trait(?Send)]
impl Perform for CreatePostLike {
  type Response = PostResponse;

  async fn perform(
    &self,
    context: &Data<LemmyContext>,
    websocket_id: Option<ConnectionId>,
  ) -> Result<PostResponse, LemmyError> {
    let data: &CreatePostLike = &self;
    let user = get_user_from_jwt(&data.auth, context.pool()).await?;

    // Don't do a downvote if site has downvotes disabled
    check_downvotes_enabled(data.score, context.pool()).await?;

    // Check for a community ban
    let post_id = data.post_id;
    let post = blocking(context.pool(), move |conn| Post::read(conn, post_id)).await??;

    check_community_ban(user.id, post.community_id, context.pool()).await?;

    let like_form = PostLikeForm {
      post_id: data.post_id,
      user_id: user.id,
      score: data.score,
    };

    // Remove any likes first
    let user_id = user.id;
    blocking(context.pool(), move |conn| {
      PostLike::remove(conn, user_id, post_id)
    })
    .await??;

    // Only add the like if the score isnt 0
    let do_add = like_form.score != 0 && (like_form.score == 1 || like_form.score == -1);
    if do_add {
      let like_form2 = like_form.clone();
      let like = move |conn: &'_ _| PostLike::like(conn, &like_form2);
      if blocking(context.pool(), like).await?.is_err() {
        return Err(ApiError::err("couldnt_like_post").into());
      }

      if like_form.score == 1 {
        post.send_like(&user, context).await?;
      } else if like_form.score == -1 {
        post.send_dislike(&user, context).await?;
      }
    } else {
      post.send_undo_like(&user, context).await?;
    }

    let post_id = data.post_id;
    let user_id = user.id;
    let post_view = match blocking(context.pool(), move |conn| {
      PostView::read(conn, post_id, Some(user_id))
    })
    .await?
    {
      Ok(post) => post,
      Err(_e) => return Err(ApiError::err("couldnt_find_post").into()),
    };

    let res = PostResponse { post_view };

    context.chat_server().do_send(SendPost {
      op: UserOperation::CreatePostLike,
      post: res.clone(),
      websocket_id,
    });

    Ok(res)
  }
}

#[async_trait::async_trait(?Send)]
impl Perform for EditPost {
  type Response = PostResponse;

  async fn perform(
    &self,
    context: &Data<LemmyContext>,
    websocket_id: Option<ConnectionId>,
  ) -> Result<PostResponse, LemmyError> {
    let data: &EditPost = &self;
    let user = get_user_from_jwt(&data.auth, context.pool()).await?;

    check_slurs(&data.name)?;
    check_slurs_opt(&data.body)?;

    if !is_valid_post_title(&data.name) {
      return Err(ApiError::err("invalid_post_title").into());
    }

    let post_id = data.post_id;
    let orig_post = blocking(context.pool(), move |conn| Post::read(conn, post_id)).await??;

    check_community_ban(user.id, orig_post.community_id, context.pool()).await?;

    // Verify that only the creator can edit
    if !Post::is_post_creator(user.id, orig_post.creator_id) {
      return Err(ApiError::err("no_post_edit_allowed").into());
    }

    // Fetch Iframely and Pictrs cached image
    let (iframely_title, iframely_description, iframely_html, pictrs_thumbnail) =
      fetch_iframely_and_pictrs_data(context.client(), data.url.to_owned()).await;

    let post_form = PostForm {
      name: data.name.trim().to_owned(),
      url: data.url.to_owned(),
      body: data.body.to_owned(),
      nsfw: data.nsfw,
      creator_id: orig_post.creator_id.to_owned(),
      community_id: orig_post.community_id,
      removed: Some(orig_post.removed),
      deleted: Some(orig_post.deleted),
      locked: Some(orig_post.locked),
      stickied: Some(orig_post.stickied),
      updated: Some(naive_now()),
      embed_title: iframely_title,
      embed_description: iframely_description,
      embed_html: iframely_html,
      thumbnail_url: pictrs_thumbnail,
      ap_id: Some(orig_post.ap_id),
      local: orig_post.local,
      published: None,
    };

    let post_id = data.post_id;
    let res = blocking(context.pool(), move |conn| {
      Post::update(conn, post_id, &post_form)
    })
    .await?;
    let updated_post: Post = match res {
      Ok(post) => post,
      Err(e) => {
        let err_type = if e.to_string() == "value too long for type character varying(200)" {
          "post_title_too_long"
        } else {
          "couldnt_update_post"
        };

        return Err(ApiError::err(err_type).into());
      }
    };

    // Send apub update
    updated_post.send_update(&user, context).await?;

    let post_id = data.post_id;
    let post_view = blocking(context.pool(), move |conn| {
      PostView::read(conn, post_id, Some(user.id))
    })
    .await??;

    let res = PostResponse { post_view };

    context.chat_server().do_send(SendPost {
      op: UserOperation::EditPost,
      post: res.clone(),
      websocket_id,
    });

    Ok(res)
  }
}

#[async_trait::async_trait(?Send)]
impl Perform for DeletePost {
  type Response = PostResponse;

  async fn perform(
    &self,
    context: &Data<LemmyContext>,
    websocket_id: Option<ConnectionId>,
  ) -> Result<PostResponse, LemmyError> {
    let data: &DeletePost = &self;
    let user = get_user_from_jwt(&data.auth, context.pool()).await?;

    let post_id = data.post_id;
    let orig_post = blocking(context.pool(), move |conn| Post::read(conn, post_id)).await??;

    check_community_ban(user.id, orig_post.community_id, context.pool()).await?;

    // Verify that only the creator can delete
    if !Post::is_post_creator(user.id, orig_post.creator_id) {
      return Err(ApiError::err("no_post_edit_allowed").into());
    }

    // Update the post
    let post_id = data.post_id;
    let deleted = data.deleted;
    let updated_post = blocking(context.pool(), move |conn| {
      Post::update_deleted(conn, post_id, deleted)
    })
    .await??;

    // apub updates
    if deleted {
      updated_post.send_delete(&user, context).await?;
    } else {
      updated_post.send_undo_delete(&user, context).await?;
    }

    // Refetch the post
    let post_id = data.post_id;
    let post_view = blocking(context.pool(), move |conn| {
      PostView::read(conn, post_id, Some(user.id))
    })
    .await??;

    let res = PostResponse { post_view };

    context.chat_server().do_send(SendPost {
      op: UserOperation::DeletePost,
      post: res.clone(),
      websocket_id,
    });

    Ok(res)
  }
}

#[async_trait::async_trait(?Send)]
impl Perform for RemovePost {
  type Response = PostResponse;

  async fn perform(
    &self,
    context: &Data<LemmyContext>,
    websocket_id: Option<ConnectionId>,
  ) -> Result<PostResponse, LemmyError> {
    let data: &RemovePost = &self;
    let user = get_user_from_jwt(&data.auth, context.pool()).await?;

    let post_id = data.post_id;
    let orig_post = blocking(context.pool(), move |conn| Post::read(conn, post_id)).await??;

    check_community_ban(user.id, orig_post.community_id, context.pool()).await?;

    // Verify that only the mods can remove
    is_mod_or_admin(context.pool(), user.id, orig_post.community_id).await?;

    // Update the post
    let post_id = data.post_id;
    let removed = data.removed;
    let updated_post = blocking(context.pool(), move |conn| {
      Post::update_removed(conn, post_id, removed)
    })
    .await??;

    // Mod tables
    let form = ModRemovePostForm {
      mod_user_id: user.id,
      post_id: data.post_id,
      removed: Some(removed),
      reason: data.reason.to_owned(),
    };
    blocking(context.pool(), move |conn| {
      ModRemovePost::create(conn, &form)
    })
    .await??;

    // apub updates
    if removed {
      updated_post.send_remove(&user, context).await?;
    } else {
      updated_post.send_undo_remove(&user, context).await?;
    }

    // Refetch the post
    let post_id = data.post_id;
    let user_id = user.id;
    let post_view = blocking(context.pool(), move |conn| {
      PostView::read(conn, post_id, Some(user_id))
    })
    .await??;

    let res = PostResponse { post_view };

    context.chat_server().do_send(SendPost {
      op: UserOperation::RemovePost,
      post: res.clone(),
      websocket_id,
    });

    Ok(res)
  }
}

#[async_trait::async_trait(?Send)]
impl Perform for LockPost {
  type Response = PostResponse;

  async fn perform(
    &self,
    context: &Data<LemmyContext>,
    websocket_id: Option<ConnectionId>,
  ) -> Result<PostResponse, LemmyError> {
    let data: &LockPost = &self;
    let user = get_user_from_jwt(&data.auth, context.pool()).await?;

    let post_id = data.post_id;
    let orig_post = blocking(context.pool(), move |conn| Post::read(conn, post_id)).await??;

    check_community_ban(user.id, orig_post.community_id, context.pool()).await?;

    // Verify that only the mods can lock
    is_mod_or_admin(context.pool(), user.id, orig_post.community_id).await?;

    // Update the post
    let post_id = data.post_id;
    let locked = data.locked;
    let updated_post = blocking(context.pool(), move |conn| {
      Post::update_locked(conn, post_id, locked)
    })
    .await??;

    // Mod tables
    let form = ModLockPostForm {
      mod_user_id: user.id,
      post_id: data.post_id,
      locked: Some(locked),
    };
    blocking(context.pool(), move |conn| ModLockPost::create(conn, &form)).await??;

    // apub updates
    updated_post.send_update(&user, context).await?;

    // Refetch the post
    let post_id = data.post_id;
    let post_view = blocking(context.pool(), move |conn| {
      PostView::read(conn, post_id, Some(user.id))
    })
    .await??;

    let res = PostResponse { post_view };

    context.chat_server().do_send(SendPost {
      op: UserOperation::LockPost,
      post: res.clone(),
      websocket_id,
    });

    Ok(res)
  }
}

#[async_trait::async_trait(?Send)]
impl Perform for StickyPost {
  type Response = PostResponse;

  async fn perform(
    &self,
    context: &Data<LemmyContext>,
    websocket_id: Option<ConnectionId>,
  ) -> Result<PostResponse, LemmyError> {
    let data: &StickyPost = &self;
    let user = get_user_from_jwt(&data.auth, context.pool()).await?;

    let post_id = data.post_id;
    let orig_post = blocking(context.pool(), move |conn| Post::read(conn, post_id)).await??;

    check_community_ban(user.id, orig_post.community_id, context.pool()).await?;

    // Verify that only the mods can sticky
    is_mod_or_admin(context.pool(), user.id, orig_post.community_id).await?;

    // Update the post
    let post_id = data.post_id;
    let stickied = data.stickied;
    let updated_post = blocking(context.pool(), move |conn| {
      Post::update_stickied(conn, post_id, stickied)
    })
    .await??;

    // Mod tables
    let form = ModStickyPostForm {
      mod_user_id: user.id,
      post_id: data.post_id,
      stickied: Some(stickied),
    };
    blocking(context.pool(), move |conn| {
      ModStickyPost::create(conn, &form)
    })
    .await??;

    // Apub updates
    // TODO stickied should pry work like locked for ease of use
    updated_post.send_update(&user, context).await?;

    // Refetch the post
    let post_id = data.post_id;
    let post_view = blocking(context.pool(), move |conn| {
      PostView::read(conn, post_id, Some(user.id))
    })
    .await??;

    let res = PostResponse { post_view };

    context.chat_server().do_send(SendPost {
      op: UserOperation::StickyPost,
      post: res.clone(),
      websocket_id,
    });

    Ok(res)
  }
}

#[async_trait::async_trait(?Send)]
impl Perform for SavePost {
  type Response = PostResponse;

  async fn perform(
    &self,
    context: &Data<LemmyContext>,
    _websocket_id: Option<ConnectionId>,
  ) -> Result<PostResponse, LemmyError> {
    let data: &SavePost = &self;
    let user = get_user_from_jwt(&data.auth, context.pool()).await?;

    let post_saved_form = PostSavedForm {
      post_id: data.post_id,
      user_id: user.id,
    };

    if data.save {
      let save = move |conn: &'_ _| PostSaved::save(conn, &post_saved_form);
      if blocking(context.pool(), save).await?.is_err() {
        return Err(ApiError::err("couldnt_save_post").into());
      }
    } else {
      let unsave = move |conn: &'_ _| PostSaved::unsave(conn, &post_saved_form);
      if blocking(context.pool(), unsave).await?.is_err() {
        return Err(ApiError::err("couldnt_save_post").into());
      }
    }

    let post_id = data.post_id;
    let user_id = user.id;
    let post_view = blocking(context.pool(), move |conn| {
      PostView::read(conn, post_id, Some(user_id))
    })
    .await??;

    Ok(PostResponse { post_view })
  }
}

/// Creates a post report and notifies the moderators of the community
#[async_trait::async_trait(?Send)]
impl Perform for CreatePostReport {
  type Response = CreatePostReportResponse;

  async fn perform(
    &self,
    context: &Data<LemmyContext>,
    websocket_id: Option<ConnectionId>,
  ) -> Result<CreatePostReportResponse, LemmyError> {
    let data: &CreatePostReport = &self;
    let user = get_user_from_jwt(&data.auth, context.pool()).await?;

    // check size of report and check for whitespace
    let reason = data.reason.trim();
    if reason.is_empty() {
      return Err(ApiError::err("report_reason_required").into());
    }
    if reason.chars().count() > 1000 {
      return Err(ApiError::err("report_too_long").into());
    }

    let user_id = user.id;
    let post_id = data.post_id;
    let post_view = blocking(context.pool(), move |conn| {
      PostView::read(&conn, post_id, None)
    })
    .await??;

    check_community_ban(user_id, post_view.community.id, context.pool()).await?;

    let report_form = PostReportForm {
      creator_id: user_id,
      post_id,
      original_post_name: post_view.post.name,
      original_post_url: post_view.post.url,
      original_post_body: post_view.post.body,
      reason: data.reason.to_owned(),
    };

    let report = match blocking(context.pool(), move |conn| {
      PostReport::report(conn, &report_form)
    })
    .await?
    {
      Ok(report) => report,
      Err(_e) => return Err(ApiError::err("couldnt_create_report").into()),
    };

    let res = CreatePostReportResponse { success: true };

    context.chat_server().do_send(SendUserRoomMessage {
      op: UserOperation::CreatePostReport,
      response: res.clone(),
      recipient_id: user.id,
      websocket_id,
    });

    context.chat_server().do_send(SendModRoomMessage {
      op: UserOperation::CreatePostReport,
      response: report,
      community_id: post_view.community.id,
      websocket_id,
    });

    Ok(res)
  }
}

/// Resolves or unresolves a post report and notifies the moderators of the community
#[async_trait::async_trait(?Send)]
impl Perform for ResolvePostReport {
  type Response = ResolvePostReportResponse;

  async fn perform(
    &self,
    context: &Data<LemmyContext>,
    websocket_id: Option<ConnectionId>,
  ) -> Result<ResolvePostReportResponse, LemmyError> {
    let data: &ResolvePostReport = &self;
    let user = get_user_from_jwt(&data.auth, context.pool()).await?;

    let report_id = data.report_id;
    let report = blocking(context.pool(), move |conn| {
      PostReportView::read(&conn, report_id)
    })
    .await??;

    let user_id = user.id;
    is_mod_or_admin(context.pool(), user_id, report.community.id).await?;

    let resolved = data.resolved;
    let resolve_fun = move |conn: &'_ _| {
      if resolved {
        PostReport::resolve(conn, report_id, user_id)
      } else {
        PostReport::unresolve(conn, report_id, user_id)
      }
    };

    let res = ResolvePostReportResponse {
      report_id,
      resolved: true,
    };

    if blocking(context.pool(), resolve_fun).await?.is_err() {
      return Err(ApiError::err("couldnt_resolve_report").into());
    };

    context.chat_server().do_send(SendModRoomMessage {
      op: UserOperation::ResolvePostReport,
      response: res.clone(),
      community_id: report.community.id,
      websocket_id,
    });

    Ok(res)
  }
}

/// Lists post reports for a community if an id is supplied
/// or returns all post reports for communities a user moderates
#[async_trait::async_trait(?Send)]
impl Perform for ListPostReports {
  type Response = ListPostReportsResponse;

  async fn perform(
    &self,
    context: &Data<LemmyContext>,
    websocket_id: Option<ConnectionId>,
  ) -> Result<ListPostReportsResponse, LemmyError> {
    let data: &ListPostReports = &self;
    let user = get_user_from_jwt(&data.auth, context.pool()).await?;

    let user_id = user.id;
    let community_id = data.community;
    let community_ids =
      collect_moderated_communities(user_id, community_id, context.pool()).await?;

    let page = data.page;
    let limit = data.limit;
    let posts = blocking(context.pool(), move |conn| {
      PostReportQueryBuilder::create(conn)
        .community_ids(community_ids)
        .page(page)
        .limit(limit)
        .list()
    })
    .await??;

    let res = ListPostReportsResponse { posts };

    context.chat_server().do_send(SendUserRoomMessage {
      op: UserOperation::ListPostReports,
      response: res.clone(),
      recipient_id: user.id,
      websocket_id,
    });

    Ok(res)
  }
}
