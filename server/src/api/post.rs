use crate::{
  api::{APIError, Oper, Perform},
  apub::{ApubLikeableType, ApubObjectType},
  blocking,
  db::{
    comment_view::*,
    community_view::*,
    moderator::*,
    post::*,
    post_view::*,
    site::*,
    site_view::*,
    user::*,
    user_view::*,
    Crud,
    Likeable,
    ListingType,
    Saveable,
    SortType,
  },
  fetch_iframely_and_pictrs_data,
  naive_now,
  slur_check,
  slurs_vec_to_str,
  websocket::{
    server::{JoinCommunityRoom, JoinPostRoom, SendPost},
    UserOperation,
    WebsocketInfo,
  },
  DbPool,
  LemmyError,
};
use serde::{Deserialize, Serialize};
use std::str::FromStr;

#[derive(Serialize, Deserialize, Debug)]
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

#[derive(Serialize, Deserialize, Debug)]
pub struct GetPosts {
  type_: String,
  sort: String,
  page: Option<i64>,
  limit: Option<i64>,
  pub community_id: Option<i32>,
  auth: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct GetPostsResponse {
  pub posts: Vec<PostView>,
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

#[async_trait::async_trait(?Send)]
impl Perform for Oper<CreatePost> {
  type Response = PostResponse;

  async fn perform(
    &self,
    pool: &DbPool,
    websocket_info: Option<WebsocketInfo>,
  ) -> Result<PostResponse, LemmyError> {
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

    // Check for a community ban
    let community_id = data.community_id;
    let is_banned =
      move |conn: &'_ _| CommunityUserBanView::get(conn, user_id, community_id).is_ok();
    if blocking(pool, is_banned).await? {
      return Err(APIError::err("community_ban").into());
    }

    // Check for a site ban
    let user = blocking(pool, move |conn| User_::read(conn, user_id)).await??;
    if user.banned {
      return Err(APIError::err("site_ban").into());
    }

    // Fetch Iframely and pictrs cached image
    let (iframely_title, iframely_description, iframely_html, pictrs_thumbnail) =
      fetch_iframely_and_pictrs_data(&self.client, data.url.to_owned()).await;

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
      thumbnail_url: pictrs_thumbnail,
      ap_id: "http://fake.com".into(),
      local: true,
      published: None,
    };

    let inserted_post = match blocking(pool, move |conn| Post::create(conn, &post_form)).await? {
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
    let updated_post =
      match blocking(pool, move |conn| Post::update_ap_id(conn, inserted_post_id)).await? {
        Ok(post) => post,
        Err(_e) => return Err(APIError::err("couldnt_create_post").into()),
      };

    updated_post.send_create(&user, &self.client, pool).await?;

    // They like their own post by default
    let like_form = PostLikeForm {
      post_id: inserted_post.id,
      user_id,
      score: 1,
    };

    let like = move |conn: &'_ _| PostLike::like(conn, &like_form);
    if blocking(pool, like).await?.is_err() {
      return Err(APIError::err("couldnt_like_post").into());
    }

    updated_post.send_like(&user, &self.client, pool).await?;

    // Refetch the view
    let inserted_post_id = inserted_post.id;
    let post_view = match blocking(pool, move |conn| {
      PostView::read(conn, inserted_post_id, Some(user_id))
    })
    .await?
    {
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

#[async_trait::async_trait(?Send)]
impl Perform for Oper<GetPost> {
  type Response = GetPostResponse;

  async fn perform(
    &self,
    pool: &DbPool,
    websocket_info: Option<WebsocketInfo>,
  ) -> Result<GetPostResponse, LemmyError> {
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

    let id = data.id;
    let post_view = match blocking(pool, move |conn| PostView::read(conn, id, user_id)).await? {
      Ok(post) => post,
      Err(_e) => return Err(APIError::err("couldnt_find_post").into()),
    };

    let id = data.id;
    let comments = blocking(pool, move |conn| {
      CommentQueryBuilder::create(conn)
        .for_post_id(id)
        .my_user_id(user_id)
        .limit(9999)
        .list()
    })
    .await??;

    let community_id = post_view.community_id;
    let community = blocking(pool, move |conn| {
      CommunityView::read(conn, community_id, user_id)
    })
    .await??;

    let community_id = post_view.community_id;
    let moderators = blocking(pool, move |conn| {
      CommunityModeratorView::for_community(conn, community_id)
    })
    .await??;

    let site_creator_id =
      blocking(pool, move |conn| Site::read(conn, 1).map(|s| s.creator_id)).await??;

    let mut admins = blocking(pool, move |conn| UserView::admins(conn)).await??;
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

#[async_trait::async_trait(?Send)]
impl Perform for Oper<GetPosts> {
  type Response = GetPostsResponse;

  async fn perform(
    &self,
    pool: &DbPool,
    websocket_info: Option<WebsocketInfo>,
  ) -> Result<GetPostsResponse, LemmyError> {
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

    let page = data.page;
    let limit = data.limit;
    let community_id = data.community_id;
    let posts = match blocking(pool, move |conn| {
      PostQueryBuilder::create(conn)
        .listing_type(type_)
        .sort(&sort)
        .show_nsfw(show_nsfw)
        .for_community_id(community_id)
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

#[async_trait::async_trait(?Send)]
impl Perform for Oper<CreatePostLike> {
  type Response = PostResponse;

  async fn perform(
    &self,
    pool: &DbPool,
    websocket_info: Option<WebsocketInfo>,
  ) -> Result<PostResponse, LemmyError> {
    let data: &CreatePostLike = &self.data;

    let claims = match Claims::decode(&data.auth) {
      Ok(claims) => claims.claims,
      Err(_e) => return Err(APIError::err("not_logged_in").into()),
    };

    let user_id = claims.id;

    // Don't do a downvote if site has downvotes disabled
    if data.score == -1 {
      let site = blocking(pool, move |conn| SiteView::read(conn)).await??;
      if !site.enable_downvotes {
        return Err(APIError::err("downvotes_disabled").into());
      }
    }

    // Check for a community ban
    let post_id = data.post_id;
    let post = blocking(pool, move |conn| Post::read(conn, post_id)).await??;

    let community_id = post.community_id;
    let is_banned =
      move |conn: &'_ _| CommunityUserBanView::get(conn, user_id, community_id).is_ok();
    if blocking(pool, is_banned).await? {
      return Err(APIError::err("community_ban").into());
    }

    // Check for a site ban
    let user = blocking(pool, move |conn| User_::read(conn, user_id)).await??;
    if user.banned {
      return Err(APIError::err("site_ban").into());
    }

    let like_form = PostLikeForm {
      post_id: data.post_id,
      user_id,
      score: data.score,
    };

    // Remove any likes first
    let like_form2 = like_form.clone();
    blocking(pool, move |conn| PostLike::remove(conn, &like_form2)).await??;

    // Only add the like if the score isnt 0
    let do_add = like_form.score != 0 && (like_form.score == 1 || like_form.score == -1);
    if do_add {
      let like_form2 = like_form.clone();
      let like = move |conn: &'_ _| PostLike::like(conn, &like_form2);
      if blocking(pool, like).await?.is_err() {
        return Err(APIError::err("couldnt_like_post").into());
      }

      if like_form.score == 1 {
        post.send_like(&user, &self.client, pool).await?;
      } else if like_form.score == -1 {
        post.send_dislike(&user, &self.client, pool).await?;
      }
    } else {
      post.send_undo_like(&user, &self.client, pool).await?;
    }

    let post_id = data.post_id;
    let post_view = match blocking(pool, move |conn| {
      PostView::read(conn, post_id, Some(user_id))
    })
    .await?
    {
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

#[async_trait::async_trait(?Send)]
impl Perform for Oper<EditPost> {
  type Response = PostResponse;

  async fn perform(
    &self,
    pool: &DbPool,
    websocket_info: Option<WebsocketInfo>,
  ) -> Result<PostResponse, LemmyError> {
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

    // Verify its the creator or a mod or admin
    let community_id = data.community_id;
    let mut editors: Vec<i32> = vec![data.creator_id];
    editors.append(
      &mut blocking(pool, move |conn| {
        CommunityModeratorView::for_community(conn, community_id)
          .map(|v| v.into_iter().map(|m| m.user_id).collect())
      })
      .await??,
    );
    editors.append(
      &mut blocking(pool, move |conn| {
        UserView::admins(conn).map(|v| v.into_iter().map(|a| a.id).collect())
      })
      .await??,
    );
    if !editors.contains(&user_id) {
      return Err(APIError::err("no_post_edit_allowed").into());
    }

    // Check for a community ban
    let community_id = data.community_id;
    let is_banned =
      move |conn: &'_ _| CommunityUserBanView::get(conn, user_id, community_id).is_ok();
    if blocking(pool, is_banned).await? {
      return Err(APIError::err("community_ban").into());
    }

    // Check for a site ban
    let user = blocking(pool, move |conn| User_::read(conn, user_id)).await??;
    if user.banned {
      return Err(APIError::err("site_ban").into());
    }

    // Fetch Iframely and Pictrs cached image
    let (iframely_title, iframely_description, iframely_html, pictrs_thumbnail) =
      fetch_iframely_and_pictrs_data(&self.client, data.url.to_owned()).await;

    let edit_id = data.edit_id;
    let read_post = blocking(pool, move |conn| Post::read(conn, edit_id)).await??;

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
      thumbnail_url: pictrs_thumbnail,
      ap_id: read_post.ap_id,
      local: read_post.local,
      published: None,
    };

    let edit_id = data.edit_id;
    let res = blocking(pool, move |conn| Post::update(conn, edit_id, &post_form)).await?;
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

    // Mod tables
    if let Some(removed) = data.removed.to_owned() {
      let form = ModRemovePostForm {
        mod_user_id: user_id,
        post_id: data.edit_id,
        removed: Some(removed),
        reason: data.reason.to_owned(),
      };
      blocking(pool, move |conn| ModRemovePost::create(conn, &form)).await??;
    }

    if let Some(locked) = data.locked.to_owned() {
      let form = ModLockPostForm {
        mod_user_id: user_id,
        post_id: data.edit_id,
        locked: Some(locked),
      };
      blocking(pool, move |conn| ModLockPost::create(conn, &form)).await??;
    }

    if let Some(stickied) = data.stickied.to_owned() {
      let form = ModStickyPostForm {
        mod_user_id: user_id,
        post_id: data.edit_id,
        stickied: Some(stickied),
      };
      blocking(pool, move |conn| ModStickyPost::create(conn, &form)).await??;
    }

    if let Some(deleted) = data.deleted.to_owned() {
      if deleted {
        updated_post.send_delete(&user, &self.client, pool).await?;
      } else {
        updated_post
          .send_undo_delete(&user, &self.client, pool)
          .await?;
      }
    } else if let Some(removed) = data.removed.to_owned() {
      if removed {
        updated_post.send_remove(&user, &self.client, pool).await?;
      } else {
        updated_post
          .send_undo_remove(&user, &self.client, pool)
          .await?;
      }
    } else {
      updated_post.send_update(&user, &self.client, pool).await?;
    }

    let edit_id = data.edit_id;
    let post_view = blocking(pool, move |conn| {
      PostView::read(conn, edit_id, Some(user_id))
    })
    .await??;

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

#[async_trait::async_trait(?Send)]
impl Perform for Oper<SavePost> {
  type Response = PostResponse;

  async fn perform(
    &self,
    pool: &DbPool,
    _websocket_info: Option<WebsocketInfo>,
  ) -> Result<PostResponse, LemmyError> {
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

    if data.save {
      let save = move |conn: &'_ _| PostSaved::save(conn, &post_saved_form);
      if blocking(pool, save).await?.is_err() {
        return Err(APIError::err("couldnt_save_post").into());
      }
    } else {
      let unsave = move |conn: &'_ _| PostSaved::unsave(conn, &post_saved_form);
      if blocking(pool, unsave).await?.is_err() {
        return Err(APIError::err("couldnt_save_post").into());
      }
    }

    let post_id = data.post_id;
    let post_view = blocking(pool, move |conn| {
      PostView::read(conn, post_id, Some(user_id))
    })
    .await??;

    Ok(PostResponse { post: post_view })
  }
}
