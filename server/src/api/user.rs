use std::str::FromStr;

use bcrypt::verify;
use failure::Error;
use serde::{Deserialize, Serialize};

use crate::api::{Perform, self};
use crate::db::comment;
use crate::db::comment_view;
use crate::db::community;
use crate::db::community_view;
use crate::db::moderator;
use crate::db::post_view;
use crate::db::user;
use crate::db::user_view;
use crate::db::{
    Crud,
    Followable,
    Joinable,
    SortType,
    establish_connection,
};

#[derive(Serialize, Deserialize, Debug)]
pub struct Login {
  username_or_email: String,
  password: String,
}

#[derive(Serialize, Deserialize)]
pub struct Register {
  username: String,
  email: Option<String>,
  password: String,
  password_verify: String,
  admin: bool,
  show_nsfw: bool,
}

#[derive(Serialize, Deserialize)]
pub struct SaveUserSettings {
  show_nsfw: bool,
  auth: String,
}

#[derive(Serialize, Deserialize)]
pub struct LoginResponse {
  op: String,
  jwt: String,
}

#[derive(Serialize, Deserialize)]
pub struct GetUserDetails {
  user_id: Option<i32>,
  username: Option<String>,
  sort: String,
  page: Option<i64>,
  limit: Option<i64>,
  community_id: Option<i32>,
  saved_only: bool,
  auth: Option<String>,
}

#[derive(Serialize, Deserialize)]
pub struct GetUserDetailsResponse {
  op: String,
  user: user_view::UserView,
  follows: Vec<community_view::CommunityFollowerView>,
  moderates: Vec<community_view::CommunityModeratorView>,
  comments: Vec<comment_view::CommentView>,
  posts: Vec<post_view::PostView>,
}

#[derive(Serialize, Deserialize)]
pub struct GetRepliesResponse {
  op: String,
  replies: Vec<comment_view::ReplyView>,
}

#[derive(Serialize, Deserialize)]
pub struct MarkAllAsRead {
  auth: String,
}

#[derive(Serialize, Deserialize)]
pub struct AddAdmin {
  user_id: i32,
  added: bool,
  auth: String,
}

#[derive(Serialize, Deserialize)]
pub struct AddAdminResponse {
  op: String,
  admins: Vec<user_view::UserView>,
}

#[derive(Serialize, Deserialize)]
pub struct BanUser {
  user_id: i32,
  ban: bool,
  reason: Option<String>,
  expires: Option<i64>,
  auth: String,
}

#[derive(Serialize, Deserialize)]
pub struct BanUserResponse {
  op: String,
  user: user_view::UserView,
  banned: bool,
}

#[derive(Serialize, Deserialize)]
pub struct GetReplies {
  sort: String,
  page: Option<i64>,
  limit: Option<i64>,
  unread_only: bool,
  auth: String,
}

impl Perform<LoginResponse> for api::Oper<Login> {
  fn perform(&self) -> Result<LoginResponse, Error> {
    let data: &Login = &self.data;
    let conn = establish_connection();

    // Fetch that username / email
    let user: user::User_ = match user::User_::find_by_email_or_username(&conn, &data.username_or_email) {
      Ok(user) => user,
      Err(_e) => {
        return Err(api::APIError::err(
          &self.op,
          "couldnt_find_that_username_or_email",
        ))?
      }
    };

    // Verify the password
    let valid: bool = verify(&data.password, &user.password_encrypted).unwrap_or(false);
    if !valid {
      return Err(api::APIError::err(&self.op, "password_incorrect"))?;
    }

    // Return the jwt
    Ok(LoginResponse {
      op: self.op.to_string(),
      jwt: user.jwt(),
    })
  }
}

impl Perform<LoginResponse> for api::Oper<Register> {
  fn perform(&self) -> Result<LoginResponse, Error> {
    let data: &Register = &self.data;
    let conn = establish_connection();

    // Make sure passwords match
    if &data.password != &data.password_verify {
      return Err(api::APIError::err(&self.op, "passwords_dont_match"))?;
    }

    if crate::has_slurs(&data.username) {
      return Err(api::APIError::err(&self.op, "no_slurs"))?;
    }

    // Make sure there are no admins
    if data.admin && user_view::UserView::admins(&conn)?.len() > 0 {
      return Err(api::APIError::err(&self.op, "admin_already_created"))?;
    }

    // Register the new user
    let user_form = user::UserForm {
      name: data.username.to_owned(),
      fedi_name: crate::Settings::get().hostname.into(),
      email: data.email.to_owned(),
      password_encrypted: data.password.to_owned(),
      preferred_username: None,
      updated: None,
      admin: data.admin,
      banned: false,
      show_nsfw: data.show_nsfw,
    };

    // Create the user
    let inserted_user = match user::User_::register(&conn, &user_form) {
      Ok(user) => user,
      Err(_e) => return Err(api::APIError::err(&self.op, "user_already_exists"))?,
    };

    // Create the main community if it doesn't exist
    let main_community: community::Community = match community::Community::read(&conn, 2) {
      Ok(c) => c,
      Err(_e) => {
        let community_form = community::CommunityForm {
          name: "main".to_string(),
          title: "The Default Community".to_string(),
          description: Some("The Default Community".to_string()),
          category_id: 1,
          nsfw: false,
          creator_id: inserted_user.id,
          removed: None,
          deleted: None,
          updated: None,
        };
        community::Community::create(&conn, &community_form).unwrap()
      }
    };

    // Sign them up for main community no matter what
    let community_follower_form = community::CommunityFollowerForm {
      community_id: main_community.id,
      user_id: inserted_user.id,
    };

    let _inserted_community_follower =
      match community::CommunityFollower::follow(&conn, &community_follower_form) {
        Ok(user) => user,
        Err(_e) => return Err(api::APIError::err(&self.op, "community_follower_already_exists"))?,
      };

    // If its an admin, add them as a mod and follower to main
    if data.admin {
      let community_moderator_form = community::CommunityModeratorForm {
        community_id: main_community.id,
        user_id: inserted_user.id,
      };

      let _inserted_community_moderator =
        match community::CommunityModerator::join(&conn, &community_moderator_form) {
          Ok(user) => user,
          Err(_e) => {
            return Err(api::APIError::err(
              &self.op,
              "community_moderator_already_exists",
            ))?
          }
        };
    }

    // Return the jwt
    Ok(LoginResponse {
      op: self.op.to_string(),
      jwt: inserted_user.jwt(),
    })
  }
}

impl Perform<LoginResponse> for api::Oper<SaveUserSettings> {
  fn perform(&self) -> Result<LoginResponse, Error> {
    let data: &SaveUserSettings = &self.data;
    let conn = establish_connection();

    let claims = match user::Claims::decode(&data.auth) {
      Ok(claims) => claims.claims,
      Err(_e) => return Err(api::APIError::err(&self.op, "not_logged_in"))?,
    };

    let user_id = claims.id;

    let read_user = user::User_::read(&conn, user_id)?;

    let user_form = user::UserForm {
      name: read_user.name,
      fedi_name: read_user.fedi_name,
      email: read_user.email,
      password_encrypted: read_user.password_encrypted,
      preferred_username: read_user.preferred_username,
      updated: Some(crate::naive_now()),
      admin: read_user.admin,
      banned: read_user.banned,
      show_nsfw: data.show_nsfw,
    };

    let updated_user = match user::User_::update(&conn, user_id, &user_form) {
      Ok(user) => user,
      Err(_e) => return Err(api::APIError::err(&self.op, "couldnt_update_user"))?,
    };

    // Return the jwt
    Ok(LoginResponse {
      op: self.op.to_string(),
      jwt: updated_user.jwt(),
    })
  }
}

impl Perform<GetUserDetailsResponse> for api::Oper<GetUserDetails> {
  fn perform(&self) -> Result<GetUserDetailsResponse, Error> {
    let data: &GetUserDetails = &self.data;
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

    //TODO add save
    let sort = SortType::from_str(&data.sort)?;

    let user_details_id = match data.user_id {
      Some(id) => id,
      None => {
        user::User_::read_from_name(
          &conn,
          data.username.to_owned().unwrap_or("admin".to_string()),
        )?
        .id
      }
    };

    let user_view = user_view::UserView::read(&conn, user_details_id)?;

    // If its saved only, you don't care what creator it was
    let posts = if data.saved_only {
      post_view::PostView::list(
        &conn,
        post_view::PostListingType::All,
        &sort,
        data.community_id,
        None,
        None,
        None,
        Some(user_details_id),
        show_nsfw,
        data.saved_only,
        false,
        data.page,
        data.limit,
      )?
    } else {
      post_view::PostView::list(
        &conn,
        post_view::PostListingType::All,
        &sort,
        data.community_id,
        Some(user_details_id),
        None,
        None,
        user_id,
        show_nsfw,
        data.saved_only,
        false,
        data.page,
        data.limit,
      )?
    };
    let comments = if data.saved_only {
      comment_view::CommentView::list(
        &conn,
        &sort,
        None,
        None,
        None,
        Some(user_details_id),
        data.saved_only,
        data.page,
        data.limit,
      )?
    } else {
      comment_view::CommentView::list(
        &conn,
        &sort,
        None,
        Some(user_details_id),
        None,
        user_id,
        data.saved_only,
        data.page,
        data.limit,
      )?
    };

    let follows = community_view::CommunityFollowerView::for_user(&conn, user_details_id)?;
    let moderates = community_view::CommunityModeratorView::for_user(&conn, user_details_id)?;

    // Return the jwt
    Ok(GetUserDetailsResponse {
      op: self.op.to_string(),
      user: user_view,
      follows: follows,
      moderates: moderates,
      comments: comments,
      posts: posts,
    })
  }
}

impl Perform<AddAdminResponse> for api::Oper<AddAdmin> {
  fn perform(&self) -> Result<AddAdminResponse, Error> {
    let data: &AddAdmin = &self.data;
    let conn = establish_connection();

    let claims = match user::Claims::decode(&data.auth) {
      Ok(claims) => claims.claims,
      Err(_e) => return Err(api::APIError::err(&self.op, "not_logged_in"))?,
    };

    let user_id = claims.id;

    // Make sure user is an admin
    if user_view::UserView::read(&conn, user_id)?.admin == false {
      return Err(api::APIError::err(&self.op, "not_an_admin"))?;
    }

    let read_user = user::User_::read(&conn, data.user_id)?;

    let user_form = user::UserForm {
      name: read_user.name,
      fedi_name: read_user.fedi_name,
      email: read_user.email,
      password_encrypted: read_user.password_encrypted,
      preferred_username: read_user.preferred_username,
      updated: Some(crate::naive_now()),
      admin: data.added,
      banned: read_user.banned,
      show_nsfw: read_user.show_nsfw,
    };

    match user::User_::update(&conn, data.user_id, &user_form) {
      Ok(user) => user,
      Err(_e) => return Err(api::APIError::err(&self.op, "couldnt_update_user"))?,
    };

    // Mod tables
    let form = moderator::ModAddForm {
      mod_user_id: user_id,
      other_user_id: data.user_id,
      removed: Some(!data.added),
    };

    moderator::ModAdd::create(&conn, &form)?;

    let site_creator_id = community::Site::read(&conn, 1)?.creator_id;
    let mut admins = user_view::UserView::admins(&conn)?;
    let creator_index = admins.iter().position(|r| r.id == site_creator_id).unwrap();
    let creator_user = admins.remove(creator_index);
    admins.insert(0, creator_user);

    Ok(AddAdminResponse {
      op: self.op.to_string(),
      admins: admins,
    })
  }
}

impl Perform<BanUserResponse> for api::Oper<BanUser> {
  fn perform(&self) -> Result<BanUserResponse, Error> {
    let data: &BanUser = &self.data;
    let conn = establish_connection();

    let claims = match user::Claims::decode(&data.auth) {
      Ok(claims) => claims.claims,
      Err(_e) => return Err(api::APIError::err(&self.op, "not_logged_in"))?,
    };

    let user_id = claims.id;

    // Make sure user is an admin
    if user_view::UserView::read(&conn, user_id)?.admin == false {
      return Err(api::APIError::err(&self.op, "not_an_admin"))?;
    }

    let read_user = user::User_::read(&conn, data.user_id)?;

    let user_form = user::UserForm {
      name: read_user.name,
      fedi_name: read_user.fedi_name,
      email: read_user.email,
      password_encrypted: read_user.password_encrypted,
      preferred_username: read_user.preferred_username,
      updated: Some(crate::naive_now()),
      admin: read_user.admin,
      banned: data.ban,
      show_nsfw: read_user.show_nsfw,
    };

    match user::User_::update(&conn, data.user_id, &user_form) {
      Ok(user) => user,
      Err(_e) => return Err(api::APIError::err(&self.op, "couldnt_update_user"))?,
    };

    // Mod tables
    let expires = match data.expires {
      Some(time) => Some(crate::naive_from_unix(time)),
      None => None,
    };

    let form = moderator::ModBanForm {
      mod_user_id: user_id,
      other_user_id: data.user_id,
      reason: data.reason.to_owned(),
      banned: Some(data.ban),
      expires: expires,
    };

    moderator::ModBan::create(&conn, &form)?;

    let user_view = user_view::UserView::read(&conn, data.user_id)?;

    Ok(BanUserResponse {
      op: self.op.to_string(),
      user: user_view,
      banned: data.ban,
    })
  }
}

impl Perform<GetRepliesResponse> for api::Oper<GetReplies> {
  fn perform(&self) -> Result<GetRepliesResponse, Error> {
    let data: &GetReplies = &self.data;
    let conn = establish_connection();

    let claims = match user::Claims::decode(&data.auth) {
      Ok(claims) => claims.claims,
      Err(_e) => return Err(api::APIError::err(&self.op, "not_logged_in"))?,
    };

    let user_id = claims.id;

    let sort = SortType::from_str(&data.sort)?;

    let replies = comment_view::ReplyView::get_replies(
      &conn,
      user_id,
      &sort,
      data.unread_only,
      data.page,
      data.limit,
    )?;

    // Return the jwt
    Ok(GetRepliesResponse {
      op: self.op.to_string(),
      replies: replies,
    })
  }
}

impl Perform<GetRepliesResponse> for api::Oper<MarkAllAsRead> {
  fn perform(&self) -> Result<GetRepliesResponse, Error> {
    let data: &MarkAllAsRead = &self.data;
    let conn = establish_connection();

    let claims = match user::Claims::decode(&data.auth) {
      Ok(claims) => claims.claims,
      Err(_e) => return Err(api::APIError::err(&self.op, "not_logged_in"))?,
    };

    let user_id = claims.id;

    let replies = comment_view::ReplyView::get_replies(&conn, user_id, &SortType::New, true, Some(1), Some(999))?;

    for reply in &replies {
      let comment_form = comment::CommentForm {
        content: reply.to_owned().content,
        parent_id: reply.to_owned().parent_id,
        post_id: reply.to_owned().post_id,
        creator_id: reply.to_owned().creator_id,
        removed: None,
        deleted: None,
        read: Some(true),
        updated: reply.to_owned().updated,
      };

      let _updated_comment = match comment::Comment::update(&conn, reply.id, &comment_form) {
        Ok(comment) => comment,
        Err(_e) => return Err(api::APIError::err(&self.op, "couldnt_update_comment"))?,
      };
    }

    let replies = comment_view::ReplyView::get_replies(&conn, user_id, &SortType::New, true, Some(1), Some(999))?;

    Ok(GetRepliesResponse {
      op: self.op.to_string(),
      replies: replies,
    })
  }
}
