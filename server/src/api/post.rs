use std::str::FromStr;

use failure::Error;
use serde::{Deserialize, Serialize};

use crate::api::{Perform, self};
use crate::db::comment_view;
use crate::db::community;
use crate::db::community_view;
use crate::db::moderator;
use crate::db::post;
use crate::db::post_view;
use crate::db::user;
use crate::db::user_view;
use crate::db::{
    Crud,
    Likeable,
    Saveable,
    SortType,
    establish_connection,
};

#[derive(Serialize, Deserialize)]
pub struct CreatePost {
  name: String,
  url: Option<String>,
  body: Option<String>,
  nsfw: bool,
  community_id: i32,
  auth: String,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct PostResponse {
  op: String,
  pub post: post_view::PostView,
}

#[derive(Serialize, Deserialize)]
pub struct GetPost {
  pub id: i32,
  auth: Option<String>,
}

#[derive(Serialize, Deserialize)]
pub struct GetPostResponse {
  op: String,
  post: post_view::PostView,
  comments: Vec<comment_view::CommentView>,
  community: community_view::CommunityView,
  moderators: Vec<community_view::CommunityModeratorView>,
  admins: Vec<user_view::UserView>,
}

#[derive(Serialize, Deserialize)]
pub struct GetPosts {
  type_: String,
  sort: String,
  page: Option<i64>,
  limit: Option<i64>,
  community_id: Option<i32>,
  auth: Option<String>,
}

#[derive(Serialize, Deserialize)]
pub struct GetPostsResponse {
  op: String,
  posts: Vec<post_view::PostView>,
}

#[derive(Serialize, Deserialize)]
pub struct CreatePostLike {
  post_id: i32,
  score: i16,
  auth: String,
}

#[derive(Serialize, Deserialize)]
pub struct CreatePostLikeResponse {
  op: String,
  post: post_view::PostView,
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

impl Perform<PostResponse> for api::Oper<CreatePost> {
  fn perform(&self) -> Result<PostResponse, Error> {
    let data: &CreatePost = &self.data;
    let conn = establish_connection();

    let claims = match user::Claims::decode(&data.auth) {
      Ok(claims) => claims.claims,
      Err(_e) => return Err(api::APIError::err(&self.op, "not_logged_in"))?,
    };

    if crate::has_slurs(&data.name) || (data.body.is_some() && crate::has_slurs(&data.body.to_owned().unwrap())) {
      return Err(api::APIError::err(&self.op, "no_slurs"))?;
    }

    let user_id = claims.id;

    // Check for a community ban
    if community_view::CommunityUserBanView::get(&conn, user_id, data.community_id).is_ok() {
      return Err(api::APIError::err(&self.op, "community_ban"))?;
    }

    // Check for a site ban
    if user_view::UserView::read(&conn, user_id)?.banned {
      return Err(api::APIError::err(&self.op, "site_ban"))?;
    }

    let post_form = post::PostForm {
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
    };

    let inserted_post = match post::Post::create(&conn, &post_form) {
      Ok(post) => post,
      Err(_e) => return Err(api::APIError::err(&self.op, "couldnt_create_post"))?,
    };

    // They like their own post by default
    let like_form = post::PostLikeForm {
      post_id: inserted_post.id,
      user_id: user_id,
      score: 1,
    };

    // Only add the like if the score isnt 0
    let _inserted_like = match post::PostLike::like(&conn, &like_form) {
      Ok(like) => like,
      Err(_e) => return Err(api::APIError::err(&self.op, "couldnt_like_post"))?,
    };

    // Refetch the view
    let post_view = match post_view::PostView::read(&conn, inserted_post.id, Some(user_id)) {
      Ok(post) => post,
      Err(_e) => return Err(api::APIError::err(&self.op, "couldnt_find_post"))?,
    };

    Ok(PostResponse {
      op: self.op.to_string(),
      post: post_view,
    })
  }
}

impl Perform<GetPostResponse> for api::Oper<GetPost> {
  fn perform(&self) -> Result<GetPostResponse, Error> {
    let data: &GetPost = &self.data;
    let conn = establish_connection();

    let user_id: Option<i32> = match &data.auth {
      Some(auth) => match user::Claims::decode(&auth) {
        Ok(claims) => {
          let user_id = claims.claims.id;
          Some(user_id)
        }
        Err(_e) => None,
      },
      None => None,
    };

    let post_view = match post_view::PostView::read(&conn, data.id, user_id) {
      Ok(post) => post,
      Err(_e) => return Err(api::APIError::err(&self.op, "couldnt_find_post"))?,
    };

    let comments = comment_view::CommentView::list(
      &conn,
      &SortType::New,
      Some(data.id),
      None,
      None,
      user_id,
      false,
      None,
      Some(9999),
    )?;

    let community = community_view::CommunityView::read(&conn, post_view.community_id, user_id)?;

    let moderators = community_view::CommunityModeratorView::for_community(&conn, post_view.community_id)?;

    let site_creator_id = community::Site::read(&conn, 1)?.creator_id;
    let mut admins = user_view::UserView::admins(&conn)?;
    let creator_index = admins.iter().position(|r| r.id == site_creator_id).unwrap();
    let creator_user = admins.remove(creator_index);
    admins.insert(0, creator_user);

    // Return the jwt
    Ok(GetPostResponse {
      op: self.op.to_string(),
      post: post_view,
      comments: comments,
      community: community,
      moderators: moderators,
      admins: admins,
    })
  }
}

impl Perform<GetPostsResponse> for api::Oper<GetPosts> {
  fn perform(&self) -> Result<GetPostsResponse, Error> {
    let data: &GetPosts = &self.data;
    let conn = establish_connection();

    let user_claims: Option<user::Claims> = match &data.auth {
      Some(auth) => match user::Claims::decode(&auth) {
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

    let type_ = post_view::PostListingType::from_str(&data.type_)?;
    let sort = SortType::from_str(&data.sort)?;

    let posts = match post_view::PostView::list(
      &conn,
      type_,
      &sort,
      data.community_id,
      None,
      None,
      None,
      user_id,
      show_nsfw,
      false,
      false,
      data.page,
      data.limit,
    ) {
      Ok(posts) => posts,
      Err(_e) => return Err(api::APIError::err(&self.op, "couldnt_get_posts"))?,
    };

    Ok(GetPostsResponse {
      op: self.op.to_string(),
      posts: posts,
    })
  }
}

impl Perform<CreatePostLikeResponse> for api::Oper<CreatePostLike> {
  fn perform(&self) -> Result<CreatePostLikeResponse, Error> {
    let data: &CreatePostLike = &self.data;
    let conn = establish_connection();

    let claims = match user::Claims::decode(&data.auth) {
      Ok(claims) => claims.claims,
      Err(_e) => return Err(api::APIError::err(&self.op, "not_logged_in"))?,
    };

    let user_id = claims.id;

    // Check for a community ban
    let post = post::Post::read(&conn, data.post_id)?;
    if community_view::CommunityUserBanView::get(&conn, user_id, post.community_id).is_ok() {
      return Err(api::APIError::err(&self.op, "community_ban"))?;
    }

    // Check for a site ban
    if user_view::UserView::read(&conn, user_id)?.banned {
      return Err(api::APIError::err(&self.op, "site_ban"))?;
    }

    let like_form = post::PostLikeForm {
      post_id: data.post_id,
      user_id: user_id,
      score: data.score,
    };

    // Remove any likes first
    post::PostLike::remove(&conn, &like_form)?;

    // Only add the like if the score isnt 0
    let do_add = &like_form.score != &0 && (&like_form.score == &1 || &like_form.score == &-1);
    if do_add {
      let _inserted_like = match post::PostLike::like(&conn, &like_form) {
        Ok(like) => like,
        Err(_e) => return Err(api::APIError::err(&self.op, "couldnt_like_post"))?,
      };
    }

    let post_view = match post_view::PostView::read(&conn, data.post_id, Some(user_id)) {
      Ok(post) => post,
      Err(_e) => return Err(api::APIError::err(&self.op, "couldnt_find_post"))?,
    };

    // just output the score
    Ok(CreatePostLikeResponse {
      op: self.op.to_string(),
      post: post_view,
    })
  }
}

impl Perform<PostResponse> for api::Oper<EditPost> {
  fn perform(&self) -> Result<PostResponse, Error> {
    let data: &EditPost = &self.data;
    if crate::has_slurs(&data.name) || (data.body.is_some() && crate::has_slurs(&data.body.to_owned().unwrap())) {
      return Err(api::APIError::err(&self.op, "no_slurs"))?;
    }

    let conn = establish_connection();

    let claims = match user::Claims::decode(&data.auth) {
      Ok(claims) => claims.claims,
      Err(_e) => return Err(api::APIError::err(&self.op, "not_logged_in"))?,
    };

    let user_id = claims.id;

    // Verify its the creator or a mod or admin
    let mut editors: Vec<i32> = vec![data.creator_id];
    editors.append(
      &mut community_view::CommunityModeratorView::for_community(&conn, data.community_id)?
        .into_iter()
        .map(|m| m.user_id)
        .collect(),
    );
    editors.append(&mut user_view::UserView::admins(&conn)?.into_iter().map(|a| a.id).collect());
    if !editors.contains(&user_id) {
      return Err(api::APIError::err(&self.op, "no_post_edit_allowed"))?;
    }

    // Check for a community ban
    if community_view::CommunityUserBanView::get(&conn, user_id, data.community_id).is_ok() {
      return Err(api::APIError::err(&self.op, "community_ban"))?;
    }

    // Check for a site ban
    if user_view::UserView::read(&conn, user_id)?.banned {
      return Err(api::APIError::err(&self.op, "site_ban"))?;
    }

    let post_form = post::PostForm {
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
      updated: Some(crate::naive_now()),
    };

    let _updated_post = match post::Post::update(&conn, data.edit_id, &post_form) {
      Ok(post) => post,
      Err(_e) => return Err(api::APIError::err(&self.op, "couldnt_update_post"))?,
    };

    // Mod tables
    if let Some(removed) = data.removed.to_owned() {
      let form = moderator::ModRemovePostForm {
        mod_user_id: user_id,
        post_id: data.edit_id,
        removed: Some(removed),
        reason: data.reason.to_owned(),
      };
      moderator::ModRemovePost::create(&conn, &form)?;
    }

    if let Some(locked) = data.locked.to_owned() {
      let form = moderator::ModLockPostForm {
        mod_user_id: user_id,
        post_id: data.edit_id,
        locked: Some(locked),
      };
      moderator::ModLockPost::create(&conn, &form)?;
    }

    if let Some(stickied) = data.stickied.to_owned() {
      let form = moderator::ModStickyPostForm {
        mod_user_id: user_id,
        post_id: data.edit_id,
        stickied: Some(stickied),
      };
      moderator::ModStickyPost::create(&conn, &form)?;
    }

    let post_view = post_view::PostView::read(&conn, data.edit_id, Some(user_id))?;

    Ok(PostResponse {
      op: self.op.to_string(),
      post: post_view,
    })
  }
}

impl Perform<PostResponse> for api::Oper<SavePost> {
  fn perform(&self) -> Result<PostResponse, Error> {
    let data: &SavePost = &self.data;
    let conn = establish_connection();

    let claims = match user::Claims::decode(&data.auth) {
      Ok(claims) => claims.claims,
      Err(_e) => return Err(api::APIError::err(&self.op, "not_logged_in"))?,
    };

    let user_id = claims.id;

    let post_saved_form = post::PostSavedForm {
      post_id: data.post_id,
      user_id: user_id,
    };

    if data.save {
      match post::PostSaved::save(&conn, &post_saved_form) {
        Ok(post) => post,
        Err(_e) => return Err(api::APIError::err(&self.op, "couldnt_save_post"))?,
      };
    } else {
      match post::PostSaved::unsave(&conn, &post_saved_form) {
        Ok(post) => post,
        Err(_e) => return Err(api::APIError::err(&self.op, "couldnt_save_post"))?,
      };
    }

    let post_view = post_view::PostView::read(&conn, data.post_id, Some(user_id))?;

    Ok(PostResponse {
      op: self.op.to_string(),
      post: post_view,
    })
  }
}
