use super::*;

#[derive(Serialize, Deserialize)]
pub struct CreatePost {
  name: String,
  url: Option<String>,
  body: Option<String>,
  nsfw: bool,
  pub community_id: i32,
  auth: String,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct PostResponse {
  pub post: PostView,
}

#[derive(Serialize, Deserialize)]
pub struct GetPost {
  pub id: i32,
  auth: Option<String>,
}

#[derive(Serialize, Deserialize)]
pub struct GetPostResponse {
  post: PostView,
  comments: Vec<CommentView>,
  community: CommunityView,
  moderators: Vec<CommunityModeratorView>,
  admins: Vec<UserView>,
  pub online: usize,
}

#[derive(Serialize, Deserialize)]
pub struct GetPosts {
  type_: String,
  sort: String,
  page: Option<i64>,
  limit: Option<i64>,
  pub community_id: Option<i32>,
  auth: Option<String>,
}

#[derive(Serialize, Deserialize)]
pub struct GetPostsResponse {
  posts: Vec<PostView>,
}

#[derive(Serialize, Deserialize)]
pub struct CreatePostLike {
  post_id: i32,
  score: i16,
  auth: String,
}

#[derive(Serialize, Deserialize)]
pub struct EditPost {
  pub edit_id: i32,
  creator_id: i32,
  community_id: i32,
  name: String,
  url: Option<String>,
  body: Option<String>,
  removed: Option<bool>,
  deleted: Option<bool>,
  nsfw: bool,
  locked: Option<bool>,
  stickied: Option<bool>,
  reason: Option<String>,
  auth: String,
}

#[derive(Serialize, Deserialize)]
pub struct SavePost {
  post_id: i32,
  save: bool,
  auth: String,
}

impl Perform for Oper<CreatePost> {
  type Response = PostResponse;

  fn perform(
    &self,
    pool: Pool<ConnectionManager<PgConnection>>,
    websocket_info: Option<WebsocketInfo>,
  ) -> Result<PostResponse, Error> {
    let data: &CreatePost = &self.data;

    let claims = match Claims::decode(&data.auth) {
      Ok(claims) => claims.claims,
      Err(_e) => return Err(APIError::err("not_logged_in").into()),
    };

    if let Err(slurs) = slur_check(&data.name) {
      return Err(APIError::err(&slurs_vec_to_str(slurs)).into());
    }

    if let Some(body) = &data.body {
      if let Err(slurs) = slur_check(body) {
        return Err(APIError::err(&slurs_vec_to_str(slurs)).into());
      }
    }

    let user_id = claims.id;

    let conn = pool.get()?;

    // Check for a community ban
    if CommunityUserBanView::get(&conn, user_id, data.community_id).is_ok() {
      return Err(APIError::err("community_ban").into());
    }

    // Check for a site ban
    if UserView::read(&conn, user_id)?.banned {
      return Err(APIError::err("site_ban").into());
    }

    // Fetch Iframely and Pictshare cached image
    let (iframely_title, iframely_description, iframely_html, pictshare_thumbnail) =
      fetch_iframely_and_pictshare_data(data.url.to_owned());

    let post_form = PostForm {
      name: data.name.to_owned(),
      url: data.url.to_owned(),
      body: data.body.to_owned(),
      community_id: data.community_id,
      creator_id: user_id,
      removed: None,
      deleted: None,
      nsfw: data.nsfw,
      locked: None,
      stickied: None,
      updated: None,
      embed_title: iframely_title,
      embed_description: iframely_description,
      embed_html: iframely_html,
      thumbnail_url: pictshare_thumbnail,
    };

    let inserted_post = match Post::create(&conn, &post_form) {
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

    // They like their own post by default
    let like_form = PostLikeForm {
      post_id: inserted_post.id,
      user_id,
      score: 1,
    };

    // Only add the like if the score isnt 0
    let _inserted_like = match PostLike::like(&conn, &like_form) {
      Ok(like) => like,
      Err(_e) => return Err(APIError::err("couldnt_like_post").into()),
    };

    // Refetch the view
    let post_view = match PostView::read(&conn, inserted_post.id, Some(user_id)) {
      Ok(post) => post,
      Err(_e) => return Err(APIError::err("couldnt_find_post").into()),
    };

    let res = PostResponse { post: post_view };

    if let Some(ws) = websocket_info {
      ws.chatserver.do_send(SendPost {
        op: UserOperation::CreatePost,
        post: res.clone(),
        my_id: ws.id,
      });
    }

    Ok(res)
  }
}

impl Perform for Oper<GetPost> {
  type Response = GetPostResponse;

  fn perform(
    &self,
    pool: Pool<ConnectionManager<PgConnection>>,
    websocket_info: Option<WebsocketInfo>,
  ) -> Result<GetPostResponse, Error> {
    let data: &GetPost = &self.data;

    let user_id: Option<i32> = match &data.auth {
      Some(auth) => match Claims::decode(&auth) {
        Ok(claims) => {
          let user_id = claims.claims.id;
          Some(user_id)
        }
        Err(_e) => None,
      },
      None => None,
    };

    let conn = pool.get()?;

    let post_view = match PostView::read(&conn, data.id, user_id) {
      Ok(post) => post,
      Err(_e) => return Err(APIError::err("couldnt_find_post").into()),
    };

    let comments = CommentQueryBuilder::create(&conn)
      .for_post_id(data.id)
      .my_user_id(user_id)
      .limit(9999)
      .list()?;

    let community = CommunityView::read(&conn, post_view.community_id, user_id)?;

    let moderators = CommunityModeratorView::for_community(&conn, post_view.community_id)?;

    let site_creator_id = Site::read(&conn, 1)?.creator_id;
    let mut admins = UserView::admins(&conn)?;
    let creator_index = admins.iter().position(|r| r.id == site_creator_id).unwrap();
    let creator_user = admins.remove(creator_index);
    admins.insert(0, creator_user);

    let online = if let Some(ws) = websocket_info {
      if let Some(id) = ws.id {
        ws.chatserver.do_send(JoinPostRoom {
          post_id: data.id,
          id,
        });
      }

      // TODO
      1
    // let fut = async {
    //   ws.chatserver.send(GetPostUsersOnline {post_id: data.id}).await.unwrap()
    // };
    // Runtime::new().unwrap().block_on(fut)
    } else {
      0
    };

    // Return the jwt
    Ok(GetPostResponse {
      post: post_view,
      comments,
      community,
      moderators,
      admins,
      online,
    })
  }
}

impl Perform for Oper<GetPosts> {
  type Response = GetPostsResponse;

  fn perform(
    &self,
    pool: Pool<ConnectionManager<PgConnection>>,
    websocket_info: Option<WebsocketInfo>,
  ) -> Result<GetPostsResponse, Error> {
    let data: &GetPosts = &self.data;

    let user_claims: Option<Claims> = match &data.auth {
      Some(auth) => match Claims::decode(&auth) {
        Ok(claims) => Some(claims.claims),
        Err(_e) => None,
      },
      None => None,
    };

    let user_id = match &user_claims {
      Some(claims) => Some(claims.id),
      None => None,
    };

    let show_nsfw = match &user_claims {
      Some(claims) => claims.show_nsfw,
      None => false,
    };

    let type_ = ListingType::from_str(&data.type_)?;
    let sort = SortType::from_str(&data.sort)?;

    let conn = pool.get()?;

    let posts = match PostQueryBuilder::create(&conn)
      .listing_type(type_)
      .sort(&sort)
      .show_nsfw(show_nsfw)
      .for_community_id(data.community_id)
      .my_user_id(user_id)
      .page(data.page)
      .limit(data.limit)
      .list()
    {
      Ok(posts) => posts,
      Err(_e) => return Err(APIError::err("couldnt_get_posts").into()),
    };

    if let Some(ws) = websocket_info {
      // You don't need to join the specific community room, bc this is already handled by
      // GetCommunity
      if data.community_id.is_none() {
        if let Some(id) = ws.id {
          // 0 is the "all" community
          ws.chatserver.do_send(JoinCommunityRoom {
            community_id: 0,
            id,
          });
        }
      }
    }

    Ok(GetPostsResponse { posts })
  }
}

impl Perform for Oper<CreatePostLike> {
  type Response = PostResponse;

  fn perform(
    &self,
    pool: Pool<ConnectionManager<PgConnection>>,
    websocket_info: Option<WebsocketInfo>,
  ) -> Result<PostResponse, Error> {
    let data: &CreatePostLike = &self.data;

    let claims = match Claims::decode(&data.auth) {
      Ok(claims) => claims.claims,
      Err(_e) => return Err(APIError::err("not_logged_in").into()),
    };

    let user_id = claims.id;

    let conn = pool.get()?;

    // Don't do a downvote if site has downvotes disabled
    if data.score == -1 {
      let site = SiteView::read(&conn)?;
      if !site.enable_downvotes {
        return Err(APIError::err("downvotes_disabled").into());
      }
    }

    // Check for a community ban
    let post = Post::read(&conn, data.post_id)?;
    if CommunityUserBanView::get(&conn, user_id, post.community_id).is_ok() {
      return Err(APIError::err("community_ban").into());
    }

    // Check for a site ban
    if UserView::read(&conn, user_id)?.banned {
      return Err(APIError::err("site_ban").into());
    }

    let like_form = PostLikeForm {
      post_id: data.post_id,
      user_id,
      score: data.score,
    };

    // Remove any likes first
    PostLike::remove(&conn, &like_form)?;

    // Only add the like if the score isnt 0
    let do_add = like_form.score != 0 && (like_form.score == 1 || like_form.score == -1);
    if do_add {
      let _inserted_like = match PostLike::like(&conn, &like_form) {
        Ok(like) => like,
        Err(_e) => return Err(APIError::err("couldnt_like_post").into()),
      };
    }

    let post_view = match PostView::read(&conn, data.post_id, Some(user_id)) {
      Ok(post) => post,
      Err(_e) => return Err(APIError::err("couldnt_find_post").into()),
    };

    let res = PostResponse { post: post_view };

    if let Some(ws) = websocket_info {
      ws.chatserver.do_send(SendPost {
        op: UserOperation::CreatePostLike,
        post: res.clone(),
        my_id: ws.id,
      });
    }

    Ok(res)
  }
}

impl Perform for Oper<EditPost> {
  type Response = PostResponse;

  fn perform(
    &self,
    pool: Pool<ConnectionManager<PgConnection>>,
    websocket_info: Option<WebsocketInfo>,
  ) -> Result<PostResponse, Error> {
    let data: &EditPost = &self.data;

    if let Err(slurs) = slur_check(&data.name) {
      return Err(APIError::err(&slurs_vec_to_str(slurs)).into());
    }

    if let Some(body) = &data.body {
      if let Err(slurs) = slur_check(body) {
        return Err(APIError::err(&slurs_vec_to_str(slurs)).into());
      }
    }

    let claims = match Claims::decode(&data.auth) {
      Ok(claims) => claims.claims,
      Err(_e) => return Err(APIError::err("not_logged_in").into()),
    };

    let user_id = claims.id;

    let conn = pool.get()?;

    // Verify its the creator or a mod or admin
    let mut editors: Vec<i32> = vec![data.creator_id];
    editors.append(
      &mut CommunityModeratorView::for_community(&conn, data.community_id)?
        .into_iter()
        .map(|m| m.user_id)
        .collect(),
    );
    editors.append(&mut UserView::admins(&conn)?.into_iter().map(|a| a.id).collect());
    if !editors.contains(&user_id) {
      return Err(APIError::err("no_post_edit_allowed").into());
    }

    // Check for a community ban
    if CommunityUserBanView::get(&conn, user_id, data.community_id).is_ok() {
      return Err(APIError::err("community_ban").into());
    }

    // Check for a site ban
    if UserView::read(&conn, user_id)?.banned {
      return Err(APIError::err("site_ban").into());
    }

    // Fetch Iframely and Pictshare cached image
    let (iframely_title, iframely_description, iframely_html, pictshare_thumbnail) =
      fetch_iframely_and_pictshare_data(data.url.to_owned());

    let post_form = PostForm {
      name: data.name.to_owned(),
      url: data.url.to_owned(),
      body: data.body.to_owned(),
      creator_id: data.creator_id.to_owned(),
      community_id: data.community_id,
      removed: data.removed.to_owned(),
      deleted: data.deleted.to_owned(),
      nsfw: data.nsfw,
      locked: data.locked.to_owned(),
      stickied: data.stickied.to_owned(),
      updated: Some(naive_now()),
      embed_title: iframely_title,
      embed_description: iframely_description,
      embed_html: iframely_html,
      thumbnail_url: pictshare_thumbnail,
    };

    let _updated_post = match Post::update(&conn, data.edit_id, &post_form) {
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

    // Mod tables
    if let Some(removed) = data.removed.to_owned() {
      let form = ModRemovePostForm {
        mod_user_id: user_id,
        post_id: data.edit_id,
        removed: Some(removed),
        reason: data.reason.to_owned(),
      };
      ModRemovePost::create(&conn, &form)?;
    }

    if let Some(locked) = data.locked.to_owned() {
      let form = ModLockPostForm {
        mod_user_id: user_id,
        post_id: data.edit_id,
        locked: Some(locked),
      };
      ModLockPost::create(&conn, &form)?;
    }

    if let Some(stickied) = data.stickied.to_owned() {
      let form = ModStickyPostForm {
        mod_user_id: user_id,
        post_id: data.edit_id,
        stickied: Some(stickied),
      };
      ModStickyPost::create(&conn, &form)?;
    }

    let post_view = PostView::read(&conn, data.edit_id, Some(user_id))?;

    let res = PostResponse { post: post_view };

    if let Some(ws) = websocket_info {
      ws.chatserver.do_send(SendPost {
        op: UserOperation::EditPost,
        post: res.clone(),
        my_id: ws.id,
      });
    }

    Ok(res)
  }
}

impl Perform for Oper<SavePost> {
  type Response = PostResponse;

  fn perform(
    &self,
    pool: Pool<ConnectionManager<PgConnection>>,
    _websocket_info: Option<WebsocketInfo>,
  ) -> Result<PostResponse, Error> {
    let data: &SavePost = &self.data;

    let claims = match Claims::decode(&data.auth) {
      Ok(claims) => claims.claims,
      Err(_e) => return Err(APIError::err("not_logged_in").into()),
    };

    let user_id = claims.id;

    let post_saved_form = PostSavedForm {
      post_id: data.post_id,
      user_id,
    };

    let conn = pool.get()?;

    if data.save {
      match PostSaved::save(&conn, &post_saved_form) {
        Ok(post) => post,
        Err(_e) => return Err(APIError::err("couldnt_save_post").into()),
      };
    } else {
      match PostSaved::unsave(&conn, &post_saved_form) {
        Ok(post) => post,
        Err(_e) => return Err(APIError::err("couldnt_save_post").into()),
      };
    }

    let post_view = PostView::read(&conn, data.post_id, Some(user_id))?;

    Ok(PostResponse { post: post_view })
  }
}
