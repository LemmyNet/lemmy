use crate::{
  check_community_ban,
  get_user_from_jwt,
  get_user_from_jwt_opt,
  is_mod_or_admin,
  Perform,
};
use actix_web::web::Data;
use lemmy_apub::{ApubLikeableType, ApubObjectType};
use lemmy_db::{
  comment_view::*,
  community_view::*,
  moderator::*,
  naive_now,
  post::*,
  post_view::*,
  site_view::*,
  Crud,
  Likeable,
  ListingType,
  Saveable,
  SortType,
};
use lemmy_structs::{blocking, post::*};
use lemmy_utils::{
  apub::{make_apub_endpoint, EndpointType},
  request::fetch_iframely_and_pictrs_data,
  utils::{check_slurs, check_slurs_opt, is_valid_post_title},
  APIError,
  ConnectionId,
  LemmyError,
};
use lemmy_websocket::{
  messages::{GetPostUsersOnline, JoinPostRoom, SendPost},
  LemmyContext,
  UserOperation,
};
use std::str::FromStr;
use url::Url;

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
      return Err(APIError::err("invalid_post_title").into());
    }

    check_community_ban(user.id, data.community_id, context.pool()).await?;

    if let Some(url) = data.url.as_ref() {
      match Url::parse(url) {
        Ok(_t) => (),
        Err(_e) => return Err(APIError::err("invalid_url").into()),
      }
    }

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

          return Err(APIError::err(err_type).into());
        }
      };

    let inserted_post_id = inserted_post.id;
    let updated_post = match blocking(context.pool(), move |conn| {
      let apub_id =
        make_apub_endpoint(EndpointType::Post, &inserted_post_id.to_string()).to_string();
      Post::update_ap_id(conn, inserted_post_id, apub_id)
    })
    .await?
    {
      Ok(post) => post,
      Err(_e) => return Err(APIError::err("couldnt_create_post").into()),
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
      return Err(APIError::err("couldnt_like_post").into());
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
      Err(_e) => return Err(APIError::err("couldnt_find_post").into()),
    };

    let res = PostResponse { post: post_view };

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
      Err(_e) => return Err(APIError::err("couldnt_find_post").into()),
    };

    let id = data.id;
    let comments = blocking(context.pool(), move |conn| {
      CommentQueryBuilder::create(conn)
        .for_post_id(id)
        .my_user_id(user_id)
        .limit(9999)
        .list()
    })
    .await??;

    let community_id = post_view.community_id;
    let community = blocking(context.pool(), move |conn| {
      CommunityView::read(conn, community_id, user_id)
    })
    .await??;

    let community_id = post_view.community_id;
    let moderators = blocking(context.pool(), move |conn| {
      CommunityModeratorView::for_community(conn, community_id)
    })
    .await??;

    let online = context
      .chat_server()
      .send(GetPostUsersOnline { post_id: data.id })
      .await
      .unwrap_or(1);

    // Return the jwt
    Ok(GetPostResponse {
      post: post_view,
      comments,
      community,
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
        .listing_type(type_)
        .sort(&sort)
        .show_nsfw(show_nsfw)
        .for_community_id(community_id)
        .for_community_name(community_name)
        .my_user_id(user_id)
        .page(page)
        .limit(limit)
        .list()
    })
    .await?
    {
      Ok(posts) => posts,
      Err(_e) => return Err(APIError::err("couldnt_get_posts").into()),
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
    if data.score == -1 {
      let site = blocking(context.pool(), move |conn| SiteView::read(conn)).await??;
      if !site.enable_downvotes {
        return Err(APIError::err("downvotes_disabled").into());
      }
    }

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
        return Err(APIError::err("couldnt_like_post").into());
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
      Err(_e) => return Err(APIError::err("couldnt_find_post").into()),
    };

    let res = PostResponse { post: post_view };

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
      return Err(APIError::err("invalid_post_title").into());
    }

    let edit_id = data.edit_id;
    let orig_post = blocking(context.pool(), move |conn| Post::read(conn, edit_id)).await??;

    check_community_ban(user.id, orig_post.community_id, context.pool()).await?;

    // Verify that only the creator can edit
    if !Post::is_post_creator(user.id, orig_post.creator_id) {
      return Err(APIError::err("no_post_edit_allowed").into());
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

    let edit_id = data.edit_id;
    let res = blocking(context.pool(), move |conn| {
      Post::update(conn, edit_id, &post_form)
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

        return Err(APIError::err(err_type).into());
      }
    };

    // Send apub update
    updated_post.send_update(&user, context).await?;

    let edit_id = data.edit_id;
    let post_view = blocking(context.pool(), move |conn| {
      PostView::read(conn, edit_id, Some(user.id))
    })
    .await??;

    let res = PostResponse { post: post_view };

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

    let edit_id = data.edit_id;
    let orig_post = blocking(context.pool(), move |conn| Post::read(conn, edit_id)).await??;

    check_community_ban(user.id, orig_post.community_id, context.pool()).await?;

    // Verify that only the creator can delete
    if !Post::is_post_creator(user.id, orig_post.creator_id) {
      return Err(APIError::err("no_post_edit_allowed").into());
    }

    // Update the post
    let edit_id = data.edit_id;
    let deleted = data.deleted;
    let updated_post = blocking(context.pool(), move |conn| {
      Post::update_deleted(conn, edit_id, deleted)
    })
    .await??;

    // apub updates
    if deleted {
      updated_post.send_delete(&user, context).await?;
    } else {
      updated_post.send_undo_delete(&user, context).await?;
    }

    // Refetch the post
    let edit_id = data.edit_id;
    let post_view = blocking(context.pool(), move |conn| {
      PostView::read(conn, edit_id, Some(user.id))
    })
    .await??;

    let res = PostResponse { post: post_view };

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

    let edit_id = data.edit_id;
    let orig_post = blocking(context.pool(), move |conn| Post::read(conn, edit_id)).await??;

    check_community_ban(user.id, orig_post.community_id, context.pool()).await?;

    // Verify that only the mods can remove
    is_mod_or_admin(context.pool(), user.id, orig_post.community_id).await?;

    // Update the post
    let edit_id = data.edit_id;
    let removed = data.removed;
    let updated_post = blocking(context.pool(), move |conn| {
      Post::update_removed(conn, edit_id, removed)
    })
    .await??;

    // Mod tables
    let form = ModRemovePostForm {
      mod_user_id: user.id,
      post_id: data.edit_id,
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
    let edit_id = data.edit_id;
    let user_id = user.id;
    let post_view = blocking(context.pool(), move |conn| {
      PostView::read(conn, edit_id, Some(user_id))
    })
    .await??;

    let res = PostResponse { post: post_view };

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

    let edit_id = data.edit_id;
    let orig_post = blocking(context.pool(), move |conn| Post::read(conn, edit_id)).await??;

    check_community_ban(user.id, orig_post.community_id, context.pool()).await?;

    // Verify that only the mods can lock
    is_mod_or_admin(context.pool(), user.id, orig_post.community_id).await?;

    // Update the post
    let edit_id = data.edit_id;
    let locked = data.locked;
    let updated_post = blocking(context.pool(), move |conn| {
      Post::update_locked(conn, edit_id, locked)
    })
    .await??;

    // Mod tables
    let form = ModLockPostForm {
      mod_user_id: user.id,
      post_id: data.edit_id,
      locked: Some(locked),
    };
    blocking(context.pool(), move |conn| ModLockPost::create(conn, &form)).await??;

    // apub updates
    updated_post.send_update(&user, context).await?;

    // Refetch the post
    let edit_id = data.edit_id;
    let post_view = blocking(context.pool(), move |conn| {
      PostView::read(conn, edit_id, Some(user.id))
    })
    .await??;

    let res = PostResponse { post: post_view };

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

    let edit_id = data.edit_id;
    let orig_post = blocking(context.pool(), move |conn| Post::read(conn, edit_id)).await??;

    check_community_ban(user.id, orig_post.community_id, context.pool()).await?;

    // Verify that only the mods can sticky
    is_mod_or_admin(context.pool(), user.id, orig_post.community_id).await?;

    // Update the post
    let edit_id = data.edit_id;
    let stickied = data.stickied;
    let updated_post = blocking(context.pool(), move |conn| {
      Post::update_stickied(conn, edit_id, stickied)
    })
    .await??;

    // Mod tables
    let form = ModStickyPostForm {
      mod_user_id: user.id,
      post_id: data.edit_id,
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
    let edit_id = data.edit_id;
    let post_view = blocking(context.pool(), move |conn| {
      PostView::read(conn, edit_id, Some(user.id))
    })
    .await??;

    let res = PostResponse { post: post_view };

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
        return Err(APIError::err("couldnt_save_post").into());
      }
    } else {
      let unsave = move |conn: &'_ _| PostSaved::unsave(conn, &post_saved_form);
      if blocking(context.pool(), unsave).await?.is_err() {
        return Err(APIError::err("couldnt_save_post").into());
      }
    }

    let post_id = data.post_id;
    let user_id = user.id;
    let post_view = blocking(context.pool(), move |conn| {
      PostView::read(conn, post_id, Some(user_id))
    })
    .await??;

    Ok(PostResponse { post: post_view })
  }
}

#[async_trait::async_trait(?Send)]
impl Perform for PostJoin {
  type Response = PostJoinResponse;

  async fn perform(
    &self,
    context: &Data<LemmyContext>,
    websocket_id: Option<ConnectionId>,
  ) -> Result<PostJoinResponse, LemmyError> {
    let data: &PostJoin = &self;

    if let Some(ws_id) = websocket_id {
      context.chat_server().do_send(JoinPostRoom {
        post_id: data.post_id,
        id: ws_id,
      });
    }

    Ok(PostJoinResponse { joined: true })
  }
}
