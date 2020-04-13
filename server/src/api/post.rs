use super::*;
use crate::apub::activities::{post_create, post_update};
use diesel::PgConnection;
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

impl Perform<PostResponse> for Oper<CreatePost> {
  fn perform(&self, conn: &PgConnection) -> Result<PostResponse, Error> {
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
    if CommunityUserBanView::get(&conn, user_id, data.community_id).is_ok() {
      return Err(APIError::err("community_ban").into());
    }

    // Check for a site ban
    let user = User_::read(&conn, user_id)?;
    if user.banned {
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
      ap_id: "changeme".into(),
      local: true,
      published: None,
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

    let updated_post = match Post::update_ap_id(&conn, inserted_post.id) {
      Ok(post) => post,
      Err(_e) => return Err(APIError::err("couldnt_create_post").into()),
    };

    post_create(&updated_post, &user, conn)?;

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

    Ok(PostResponse { post: post_view })
  }
}

impl Perform<GetPostResponse> for Oper<GetPost> {
  fn perform(&self, conn: &PgConnection) -> Result<GetPostResponse, Error> {
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

    // Return the jwt
    Ok(GetPostResponse {
      post: post_view,
      comments,
      community,
      moderators,
      admins,
      online: 0,
    })
  }
}

impl Perform<GetPostsResponse> for Oper<GetPosts> {
  fn perform(&self, conn: &PgConnection) -> Result<GetPostsResponse, Error> {
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

    Ok(GetPostsResponse { posts })
  }
}

impl Perform<PostResponse> for Oper<CreatePostLike> {
  fn perform(&self, conn: &PgConnection) -> Result<PostResponse, Error> {
    let data: &CreatePostLike = &self.data;

    let claims = match Claims::decode(&data.auth) {
      Ok(claims) => claims.claims,
      Err(_e) => return Err(APIError::err("not_logged_in").into()),
    };

    let user_id = claims.id;

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

    // just output the score
    Ok(PostResponse { post: post_view })
  }
}

impl Perform<PostResponse> for Oper<EditPost> {
  fn perform(&self, conn: &PgConnection) -> Result<PostResponse, Error> {
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
    let user = User_::read(&conn, user_id)?;
    if user.banned {
      return Err(APIError::err("site_ban").into());
    }

    // Fetch Iframely and Pictshare cached image
    let (iframely_title, iframely_description, iframely_html, pictshare_thumbnail) =
      fetch_iframely_and_pictshare_data(data.url.to_owned());

    let read_post = Post::read(&conn, data.edit_id)?;

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
      ap_id: read_post.ap_id,
      local: read_post.local,
      published: None,
    };

    let updated_post = match Post::update(&conn, data.edit_id, &post_form) {
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

    post_update(&updated_post, &user, conn)?;

    let post_view = PostView::read(&conn, data.edit_id, Some(user_id))?;

    Ok(PostResponse { post: post_view })
  }
}

impl Perform<PostResponse> for Oper<SavePost> {
  fn perform(&self, conn: &PgConnection) -> Result<PostResponse, Error> {
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
