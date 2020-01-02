use super::*;
use crate::settings::Settings;
use crate::{generate_random_string, send_email};
use bcrypt::verify;
use std::str::FromStr;

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
  theme: String,
  default_sort_type: i16,
  default_listing_type: i16,
  lang: String,
  avatar: Option<String>,
  email: Option<String>,
  new_password: Option<String>,
  new_password_verify: Option<String>,
  old_password: Option<String>,
  show_avatars: bool,
  send_notifications_to_email: bool,
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
  user: UserView,
  follows: Vec<CommunityFollowerView>,
  moderates: Vec<CommunityModeratorView>,
  comments: Vec<CommentView>,
  posts: Vec<PostView>,
  admins: Vec<UserView>,
}

#[derive(Serialize, Deserialize)]
pub struct GetRepliesResponse {
  op: String,
  replies: Vec<ReplyView>,
}

#[derive(Serialize, Deserialize)]
pub struct GetUserMentionsResponse {
  op: String,
  mentions: Vec<UserMentionView>,
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
  admins: Vec<UserView>,
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
  user: UserView,
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

#[derive(Serialize, Deserialize)]
pub struct GetUserMentions {
  sort: String,
  page: Option<i64>,
  limit: Option<i64>,
  unread_only: bool,
  auth: String,
}

#[derive(Serialize, Deserialize)]
pub struct EditUserMention {
  user_mention_id: i32,
  read: Option<bool>,
  auth: String,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct UserMentionResponse {
  op: String,
  mention: UserMentionView,
}

#[derive(Serialize, Deserialize)]
pub struct DeleteAccount {
  password: String,
  auth: String,
}

#[derive(Serialize, Deserialize)]
pub struct PasswordReset {
  email: String,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct PasswordResetResponse {
  op: String,
}

#[derive(Serialize, Deserialize)]
pub struct PasswordChange {
  token: String,
  password: String,
  password_verify: String,
}

impl Perform<LoginResponse> for Oper<Login> {
  fn perform(&self) -> Result<LoginResponse, Error> {
    let data: &Login = &self.data;
    let conn = establish_connection();

    // Fetch that username / email
    let user: User_ = match User_::find_by_email_or_username(&conn, &data.username_or_email) {
      Ok(user) => user,
      Err(_e) => return Err(APIError::err(&self.op, "couldnt_find_that_username_or_email").into()),
    };

    // Verify the password
    let valid: bool = verify(&data.password, &user.password_encrypted).unwrap_or(false);
    if !valid {
      return Err(APIError::err(&self.op, "password_incorrect").into());
    }

    // Return the jwt
    Ok(LoginResponse {
      op: self.op.to_string(),
      jwt: user.jwt(),
    })
  }
}

impl Perform<LoginResponse> for Oper<Register> {
  fn perform(&self) -> Result<LoginResponse, Error> {
    let data: &Register = &self.data;
    let conn = establish_connection();

    // Make sure site has open registration
    if let Ok(site) = SiteView::read(&conn) {
      if !site.open_registration {
        return Err(APIError::err(&self.op, "registration_closed").into());
      }
    }

    // Make sure passwords match
    if data.password != data.password_verify {
      return Err(APIError::err(&self.op, "passwords_dont_match").into());
    }

    if has_slurs(&data.username) {
      return Err(APIError::err(&self.op, "no_slurs").into());
    }

    // Make sure there are no admins
    if data.admin && !UserView::admins(&conn)?.is_empty() {
      return Err(APIError::err(&self.op, "admin_already_created").into());
    }

    // Register the new user
    let user_form = UserForm {
      name: data.username.to_owned(),
      fedi_name: Settings::get().hostname.to_owned(),
      email: data.email.to_owned(),
      avatar: None,
      password_encrypted: data.password.to_owned(),
      preferred_username: None,
      updated: None,
      admin: data.admin,
      banned: false,
      show_nsfw: data.show_nsfw,
      theme: "darkly".into(),
      default_sort_type: SortType::Hot as i16,
      default_listing_type: ListingType::Subscribed as i16,
      lang: "browser".into(),
      show_avatars: true,
      send_notifications_to_email: false,
    };

    // Create the user
    let inserted_user = match User_::register(&conn, &user_form) {
      Ok(user) => user,
      Err(_e) => return Err(APIError::err(&self.op, "user_already_exists").into()),
    };

    // Create the main community if it doesn't exist
    let main_community: Community = match Community::read(&conn, 2) {
      Ok(c) => c,
      Err(_e) => {
        let community_form = CommunityForm {
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
        Community::create(&conn, &community_form).unwrap()
      }
    };

    // Sign them up for main community no matter what
    let community_follower_form = CommunityFollowerForm {
      community_id: main_community.id,
      user_id: inserted_user.id,
    };

    let _inserted_community_follower =
      match CommunityFollower::follow(&conn, &community_follower_form) {
        Ok(user) => user,
        Err(_e) => return Err(APIError::err(&self.op, "community_follower_already_exists").into()),
      };

    // If its an admin, add them as a mod and follower to main
    if data.admin {
      let community_moderator_form = CommunityModeratorForm {
        community_id: main_community.id,
        user_id: inserted_user.id,
      };

      let _inserted_community_moderator =
        match CommunityModerator::join(&conn, &community_moderator_form) {
          Ok(user) => user,
          Err(_e) => {
            return Err(APIError::err(&self.op, "community_moderator_already_exists").into())
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

impl Perform<LoginResponse> for Oper<SaveUserSettings> {
  fn perform(&self) -> Result<LoginResponse, Error> {
    let data: &SaveUserSettings = &self.data;
    let conn = establish_connection();

    let claims = match Claims::decode(&data.auth) {
      Ok(claims) => claims.claims,
      Err(_e) => return Err(APIError::err(&self.op, "not_logged_in").into()),
    };

    let user_id = claims.id;

    let read_user = User_::read(&conn, user_id)?;

    let email = match &data.email {
      Some(email) => Some(email.to_owned()),
      None => read_user.email,
    };

    let password_encrypted = match &data.new_password {
      Some(new_password) => {
        match &data.new_password_verify {
          Some(new_password_verify) => {
            // Make sure passwords match
            if new_password != new_password_verify {
              return Err(APIError::err(&self.op, "passwords_dont_match").into());
            }

            // Check the old password
            match &data.old_password {
              Some(old_password) => {
                let valid: bool =
                  verify(old_password, &read_user.password_encrypted).unwrap_or(false);
                if !valid {
                  return Err(APIError::err(&self.op, "password_incorrect").into());
                }
                User_::update_password(&conn, user_id, &new_password)?.password_encrypted
              }
              None => return Err(APIError::err(&self.op, "password_incorrect").into()),
            }
          }
          None => return Err(APIError::err(&self.op, "passwords_dont_match").into()),
        }
      }
      None => read_user.password_encrypted,
    };

    let user_form = UserForm {
      name: read_user.name,
      fedi_name: read_user.fedi_name,
      email,
      avatar: data.avatar.to_owned(),
      password_encrypted,
      preferred_username: read_user.preferred_username,
      updated: Some(naive_now()),
      admin: read_user.admin,
      banned: read_user.banned,
      show_nsfw: data.show_nsfw,
      theme: data.theme.to_owned(),
      default_sort_type: data.default_sort_type,
      default_listing_type: data.default_listing_type,
      lang: data.lang.to_owned(),
      show_avatars: data.show_avatars,
      send_notifications_to_email: data.send_notifications_to_email,
    };

    let updated_user = match User_::update(&conn, user_id, &user_form) {
      Ok(user) => user,
      Err(_e) => return Err(APIError::err(&self.op, "couldnt_update_user").into()),
    };

    // Return the jwt
    Ok(LoginResponse {
      op: self.op.to_string(),
      jwt: updated_user.jwt(),
    })
  }
}

impl Perform<GetUserDetailsResponse> for Oper<GetUserDetails> {
  fn perform(&self) -> Result<GetUserDetailsResponse, Error> {
    let data: &GetUserDetails = &self.data;
    let conn = establish_connection();

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

    let sort = SortType::from_str(&data.sort)?;

    let user_details_id = match data.user_id {
      Some(id) => id,
      None => {
        match User_::read_from_name(
          &conn,
          data
            .username
            .to_owned()
            .unwrap_or_else(|| "admin".to_string()),
        ) {
          Ok(user) => user.id,
          Err(_e) => {
            return Err(APIError::err(&self.op, "couldnt_find_that_username_or_email").into())
          }
        }
      }
    };

    let user_view = UserView::read(&conn, user_details_id)?;

    let mut posts_query = PostQueryBuilder::create(&conn)
      .sort(&sort)
      .show_nsfw(show_nsfw)
      .saved_only(data.saved_only)
      .for_community_id(data.community_id)
      .my_user_id(user_id)
      .page(data.page)
      .limit(data.limit);

    let mut comments_query = CommentQueryBuilder::create(&conn)
      .sort(&sort)
      .saved_only(data.saved_only)
      .my_user_id(user_id)
      .page(data.page)
      .limit(data.limit);

    // If its saved only, you don't care what creator it was
    // Or, if its not saved, then you only want it for that specific creator
    if !data.saved_only {
      posts_query = posts_query.for_creator_id(user_details_id);
      comments_query = comments_query.for_creator_id(user_details_id);
    }

    let posts = posts_query.list()?;
    let comments = comments_query.list()?;

    let follows = CommunityFollowerView::for_user(&conn, user_details_id)?;
    let moderates = CommunityModeratorView::for_user(&conn, user_details_id)?;
    let site_creator_id = Site::read(&conn, 1)?.creator_id;
    let mut admins = UserView::admins(&conn)?;
    let creator_index = admins.iter().position(|r| r.id == site_creator_id).unwrap();
    let creator_user = admins.remove(creator_index);
    admins.insert(0, creator_user);

    // Return the jwt
    Ok(GetUserDetailsResponse {
      op: self.op.to_string(),
      user: user_view,
      follows,
      moderates,
      comments,
      posts,
      admins,
    })
  }
}

impl Perform<AddAdminResponse> for Oper<AddAdmin> {
  fn perform(&self) -> Result<AddAdminResponse, Error> {
    let data: &AddAdmin = &self.data;
    let conn = establish_connection();

    let claims = match Claims::decode(&data.auth) {
      Ok(claims) => claims.claims,
      Err(_e) => return Err(APIError::err(&self.op, "not_logged_in").into()),
    };

    let user_id = claims.id;

    // Make sure user is an admin
    if !UserView::read(&conn, user_id)?.admin {
      return Err(APIError::err(&self.op, "not_an_admin").into());
    }

    let read_user = User_::read(&conn, data.user_id)?;

    let user_form = UserForm {
      name: read_user.name,
      fedi_name: read_user.fedi_name,
      email: read_user.email,
      avatar: read_user.avatar,
      password_encrypted: read_user.password_encrypted,
      preferred_username: read_user.preferred_username,
      updated: Some(naive_now()),
      admin: data.added,
      banned: read_user.banned,
      show_nsfw: read_user.show_nsfw,
      theme: read_user.theme,
      default_sort_type: read_user.default_sort_type,
      default_listing_type: read_user.default_listing_type,
      lang: read_user.lang,
      show_avatars: read_user.show_avatars,
      send_notifications_to_email: read_user.send_notifications_to_email,
    };

    match User_::update(&conn, data.user_id, &user_form) {
      Ok(user) => user,
      Err(_e) => return Err(APIError::err(&self.op, "couldnt_update_user").into()),
    };

    // Mod tables
    let form = ModAddForm {
      mod_user_id: user_id,
      other_user_id: data.user_id,
      removed: Some(!data.added),
    };

    ModAdd::create(&conn, &form)?;

    let site_creator_id = Site::read(&conn, 1)?.creator_id;
    let mut admins = UserView::admins(&conn)?;
    let creator_index = admins.iter().position(|r| r.id == site_creator_id).unwrap();
    let creator_user = admins.remove(creator_index);
    admins.insert(0, creator_user);

    Ok(AddAdminResponse {
      op: self.op.to_string(),
      admins,
    })
  }
}

impl Perform<BanUserResponse> for Oper<BanUser> {
  fn perform(&self) -> Result<BanUserResponse, Error> {
    let data: &BanUser = &self.data;
    let conn = establish_connection();

    let claims = match Claims::decode(&data.auth) {
      Ok(claims) => claims.claims,
      Err(_e) => return Err(APIError::err(&self.op, "not_logged_in").into()),
    };

    let user_id = claims.id;

    // Make sure user is an admin
    if !UserView::read(&conn, user_id)?.admin {
      return Err(APIError::err(&self.op, "not_an_admin").into());
    }

    let read_user = User_::read(&conn, data.user_id)?;

    let user_form = UserForm {
      name: read_user.name,
      fedi_name: read_user.fedi_name,
      email: read_user.email,
      avatar: read_user.avatar,
      password_encrypted: read_user.password_encrypted,
      preferred_username: read_user.preferred_username,
      updated: Some(naive_now()),
      admin: read_user.admin,
      banned: data.ban,
      show_nsfw: read_user.show_nsfw,
      theme: read_user.theme,
      default_sort_type: read_user.default_sort_type,
      default_listing_type: read_user.default_listing_type,
      lang: read_user.lang,
      show_avatars: read_user.show_avatars,
      send_notifications_to_email: read_user.send_notifications_to_email,
    };

    match User_::update(&conn, data.user_id, &user_form) {
      Ok(user) => user,
      Err(_e) => return Err(APIError::err(&self.op, "couldnt_update_user").into()),
    };

    // Mod tables
    let expires = match data.expires {
      Some(time) => Some(naive_from_unix(time)),
      None => None,
    };

    let form = ModBanForm {
      mod_user_id: user_id,
      other_user_id: data.user_id,
      reason: data.reason.to_owned(),
      banned: Some(data.ban),
      expires,
    };

    ModBan::create(&conn, &form)?;

    let user_view = UserView::read(&conn, data.user_id)?;

    Ok(BanUserResponse {
      op: self.op.to_string(),
      user: user_view,
      banned: data.ban,
    })
  }
}

impl Perform<GetRepliesResponse> for Oper<GetReplies> {
  fn perform(&self) -> Result<GetRepliesResponse, Error> {
    let data: &GetReplies = &self.data;
    let conn = establish_connection();

    let claims = match Claims::decode(&data.auth) {
      Ok(claims) => claims.claims,
      Err(_e) => return Err(APIError::err(&self.op, "not_logged_in").into()),
    };

    let user_id = claims.id;

    let sort = SortType::from_str(&data.sort)?;

    let replies = ReplyQueryBuilder::create(&conn, user_id)
      .sort(&sort)
      .unread_only(data.unread_only)
      .page(data.page)
      .limit(data.limit)
      .list()?;

    Ok(GetRepliesResponse {
      op: self.op.to_string(),
      replies,
    })
  }
}

impl Perform<GetUserMentionsResponse> for Oper<GetUserMentions> {
  fn perform(&self) -> Result<GetUserMentionsResponse, Error> {
    let data: &GetUserMentions = &self.data;
    let conn = establish_connection();

    let claims = match Claims::decode(&data.auth) {
      Ok(claims) => claims.claims,
      Err(_e) => return Err(APIError::err(&self.op, "not_logged_in").into()),
    };

    let user_id = claims.id;

    let sort = SortType::from_str(&data.sort)?;

    let mentions = UserMentionQueryBuilder::create(&conn, user_id)
      .sort(&sort)
      .unread_only(data.unread_only)
      .page(data.page)
      .limit(data.limit)
      .list()?;

    Ok(GetUserMentionsResponse {
      op: self.op.to_string(),
      mentions,
    })
  }
}

impl Perform<UserMentionResponse> for Oper<EditUserMention> {
  fn perform(&self) -> Result<UserMentionResponse, Error> {
    let data: &EditUserMention = &self.data;
    let conn = establish_connection();

    let claims = match Claims::decode(&data.auth) {
      Ok(claims) => claims.claims,
      Err(_e) => return Err(APIError::err(&self.op, "not_logged_in").into()),
    };

    let user_id = claims.id;

    let user_mention = UserMention::read(&conn, data.user_mention_id)?;

    let user_mention_form = UserMentionForm {
      recipient_id: user_id,
      comment_id: user_mention.comment_id,
      read: data.read.to_owned(),
    };

    let _updated_user_mention =
      match UserMention::update(&conn, user_mention.id, &user_mention_form) {
        Ok(comment) => comment,
        Err(_e) => return Err(APIError::err(&self.op, "couldnt_update_comment").into()),
      };

    let user_mention_view = UserMentionView::read(&conn, user_mention.id, user_id)?;

    Ok(UserMentionResponse {
      op: self.op.to_string(),
      mention: user_mention_view,
    })
  }
}

impl Perform<GetRepliesResponse> for Oper<MarkAllAsRead> {
  fn perform(&self) -> Result<GetRepliesResponse, Error> {
    let data: &MarkAllAsRead = &self.data;
    let conn = establish_connection();

    let claims = match Claims::decode(&data.auth) {
      Ok(claims) => claims.claims,
      Err(_e) => return Err(APIError::err(&self.op, "not_logged_in").into()),
    };

    let user_id = claims.id;

    let replies = ReplyQueryBuilder::create(&conn, user_id)
      .unread_only(true)
      .page(1)
      .limit(999)
      .list()?;

    for reply in &replies {
      let comment_form = CommentForm {
        content: reply.to_owned().content,
        parent_id: reply.to_owned().parent_id,
        post_id: reply.to_owned().post_id,
        creator_id: reply.to_owned().creator_id,
        removed: None,
        deleted: None,
        read: Some(true),
        updated: reply.to_owned().updated,
      };

      let _updated_comment = match Comment::update(&conn, reply.id, &comment_form) {
        Ok(comment) => comment,
        Err(_e) => return Err(APIError::err(&self.op, "couldnt_update_comment").into()),
      };
    }

    // Mentions
    let mentions = UserMentionQueryBuilder::create(&conn, user_id)
      .unread_only(true)
      .page(1)
      .limit(999)
      .list()?;

    for mention in &mentions {
      let mention_form = UserMentionForm {
        recipient_id: mention.to_owned().recipient_id,
        comment_id: mention.to_owned().id,
        read: Some(true),
      };

      let _updated_mention =
        match UserMention::update(&conn, mention.user_mention_id, &mention_form) {
          Ok(mention) => mention,
          Err(_e) => return Err(APIError::err(&self.op, "couldnt_update_comment").into()),
        };
    }

    Ok(GetRepliesResponse {
      op: self.op.to_string(),
      replies: vec![],
    })
  }
}

impl Perform<LoginResponse> for Oper<DeleteAccount> {
  fn perform(&self) -> Result<LoginResponse, Error> {
    let data: &DeleteAccount = &self.data;
    let conn = establish_connection();

    let claims = match Claims::decode(&data.auth) {
      Ok(claims) => claims.claims,
      Err(_e) => return Err(APIError::err(&self.op, "not_logged_in").into()),
    };

    let user_id = claims.id;

    let user: User_ = User_::read(&conn, user_id)?;

    // Verify the password
    let valid: bool = verify(&data.password, &user.password_encrypted).unwrap_or(false);
    if !valid {
      return Err(APIError::err(&self.op, "password_incorrect").into());
    }

    // Comments
    let comments = CommentQueryBuilder::create(&conn)
      .for_creator_id(user_id)
      .limit(std::i64::MAX)
      .list()?;

    for comment in &comments {
      let comment_form = CommentForm {
        content: "*Permananently Deleted*".to_string(),
        parent_id: comment.to_owned().parent_id,
        post_id: comment.to_owned().post_id,
        creator_id: comment.to_owned().creator_id,
        removed: None,
        deleted: Some(true),
        read: None,
        updated: Some(naive_now()),
      };

      let _updated_comment = match Comment::update(&conn, comment.id, &comment_form) {
        Ok(comment) => comment,
        Err(_e) => return Err(APIError::err(&self.op, "couldnt_update_comment").into()),
      };
    }

    // Posts
    let posts = PostQueryBuilder::create(&conn)
      .sort(&SortType::New)
      .for_creator_id(user_id)
      .limit(std::i64::MAX)
      .list()?;

    for post in &posts {
      let post_form = PostForm {
        name: "*Permananently Deleted*".to_string(),
        url: Some("https://deleted.com".to_string()),
        body: Some("*Permananently Deleted*".to_string()),
        creator_id: post.to_owned().creator_id,
        community_id: post.to_owned().community_id,
        removed: None,
        deleted: Some(true),
        nsfw: post.to_owned().nsfw,
        locked: None,
        stickied: None,
        updated: Some(naive_now()),
      };

      let _updated_post = match Post::update(&conn, post.id, &post_form) {
        Ok(post) => post,
        Err(_e) => return Err(APIError::err(&self.op, "couldnt_update_post").into()),
      };
    }

    Ok(LoginResponse {
      op: self.op.to_string(),
      jwt: data.auth.to_owned(),
    })
  }
}

impl Perform<PasswordResetResponse> for Oper<PasswordReset> {
  fn perform(&self) -> Result<PasswordResetResponse, Error> {
    let data: &PasswordReset = &self.data;
    let conn = establish_connection();

    // Fetch that email
    let user: User_ = match User_::find_by_email(&conn, &data.email) {
      Ok(user) => user,
      Err(_e) => return Err(APIError::err(&self.op, "couldnt_find_that_username_or_email").into()),
    };

    // Generate a random token
    let token = generate_random_string();

    // Insert the row
    PasswordResetRequest::create_token(&conn, user.id, &token)?;

    // Email the pure token to the user.
    // TODO no i18n support here.
    let user_email = &user.email.expect("email");
    let subject = &format!("Password reset for {}", user.name);
    let hostname = &format!("https://{}", Settings::get().hostname); //TODO add https for now.
    let html = &format!("<h1>Password Reset Request for {}</h1><br><a href={}/password_change/{}>Click here to reset your password</a>", user.name, hostname, &token);
    match send_email(subject, user_email, &user.name, html) {
      Ok(_o) => _o,
      Err(_e) => return Err(APIError::err(&self.op, &_e).into()),
    };

    Ok(PasswordResetResponse {
      op: self.op.to_string(),
    })
  }
}

impl Perform<LoginResponse> for Oper<PasswordChange> {
  fn perform(&self) -> Result<LoginResponse, Error> {
    let data: &PasswordChange = &self.data;
    let conn = establish_connection();

    // Fetch the user_id from the token
    let user_id = PasswordResetRequest::read_from_token(&conn, &data.token)?.user_id;

    // Make sure passwords match
    if data.password != data.password_verify {
      return Err(APIError::err(&self.op, "passwords_dont_match").into());
    }

    // Update the user with the new password
    let updated_user = match User_::update_password(&conn, user_id, &data.password) {
      Ok(user) => user,
      Err(_e) => return Err(APIError::err(&self.op, "couldnt_update_user").into()),
    };

    // Return the jwt
    Ok(LoginResponse {
      op: self.op.to_string(),
      jwt: updated_user.jwt(),
    })
  }
}
