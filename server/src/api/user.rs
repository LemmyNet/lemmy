use super::*;
use crate::apub::{gen_keypair_str, make_apub_endpoint, EndpointType};
use crate::settings::Settings;
use crate::{generate_random_string, send_email};
use bcrypt::verify;
use diesel::PgConnection;
use log::error;
use std::str::FromStr;

#[derive(Serialize, Deserialize, Debug)]
pub struct Login {
  username_or_email: String,
  password: String,
}

#[derive(Serialize, Deserialize)]
pub struct Register {
  pub username: String,
  pub email: Option<String>,
  pub password: String,
  pub password_verify: String,
  pub admin: bool,
  pub show_nsfw: bool,
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
  matrix_user_id: Option<String>,
  new_password: Option<String>,
  new_password_verify: Option<String>,
  old_password: Option<String>,
  show_avatars: bool,
  send_notifications_to_email: bool,
  auth: String,
}

#[derive(Serialize, Deserialize)]
pub struct LoginResponse {
  pub jwt: String,
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
  user: UserView,
  follows: Vec<CommunityFollowerView>,
  moderates: Vec<CommunityModeratorView>,
  comments: Vec<CommentView>,
  posts: Vec<PostView>,
  admins: Vec<UserView>,
}

#[derive(Serialize, Deserialize)]
pub struct GetRepliesResponse {
  replies: Vec<ReplyView>,
}

#[derive(Serialize, Deserialize)]
pub struct GetUserMentionsResponse {
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
pub struct PasswordResetResponse {}

#[derive(Serialize, Deserialize)]
pub struct PasswordChange {
  token: String,
  password: String,
  password_verify: String,
}

#[derive(Serialize, Deserialize)]
pub struct CreatePrivateMessage {
  content: String,
  pub recipient_id: i32,
  auth: String,
}

#[derive(Serialize, Deserialize)]
pub struct EditPrivateMessage {
  edit_id: i32,
  content: Option<String>,
  deleted: Option<bool>,
  read: Option<bool>,
  auth: String,
}

#[derive(Serialize, Deserialize)]
pub struct GetPrivateMessages {
  unread_only: bool,
  page: Option<i64>,
  limit: Option<i64>,
  auth: String,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct PrivateMessagesResponse {
  messages: Vec<PrivateMessageView>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct PrivateMessageResponse {
  message: PrivateMessageView,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct UserJoin {
  auth: String,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct UserJoinResponse {
  pub user_id: i32,
}

impl Perform<LoginResponse> for Oper<Login> {
  fn perform(&self, conn: &PgConnection) -> Result<LoginResponse, Error> {
    let data: &Login = &self.data;

    // Fetch that username / email
    let user: User_ = match User_::find_by_email_or_username(&conn, &data.username_or_email) {
      Ok(user) => user,
      Err(_e) => return Err(APIError::err("couldnt_find_that_username_or_email").into()),
    };

    // Verify the password
    let valid: bool = verify(&data.password, &user.password_encrypted).unwrap_or(false);
    if !valid {
      return Err(APIError::err("password_incorrect").into());
    }

    // Return the jwt
    Ok(LoginResponse { jwt: user.jwt() })
  }
}

impl Perform<LoginResponse> for Oper<Register> {
  fn perform(&self, conn: &PgConnection) -> Result<LoginResponse, Error> {
    let data: &Register = &self.data;

    // Make sure site has open registration
    if let Ok(site) = SiteView::read(&conn) {
      if !site.open_registration {
        return Err(APIError::err("registration_closed").into());
      }
    }

    // Make sure passwords match
    if data.password != data.password_verify {
      return Err(APIError::err("passwords_dont_match").into());
    }

    if let Err(slurs) = slur_check(&data.username) {
      return Err(APIError::err(&slurs_vec_to_str(slurs)).into());
    }

    // Make sure there are no admins
    if data.admin && !UserView::admins(&conn)?.is_empty() {
      return Err(APIError::err("admin_already_created").into());
    }

    let (user_public_key, user_private_key) = gen_keypair_str();

    // Register the new user
    let user_form = UserForm {
      name: data.username.to_owned(),
      email: data.email.to_owned(),
      matrix_user_id: None,
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
      actor_id: make_apub_endpoint(EndpointType::User, &data.username).to_string(),
      bio: None,
      local: true,
      private_key: Some(user_private_key),
      public_key: Some(user_public_key),
      last_refreshed_at: None,
    };

    // Create the user
    let inserted_user = match User_::register(&conn, &user_form) {
      Ok(user) => user,
      Err(e) => {
        let err_type = if e.to_string()
          == "duplicate key value violates unique constraint \"user__email_key\""
        {
          "email_already_exists"
        } else {
          "user_already_exists"
        };

        return Err(APIError::err(err_type).into());
      }
    };

    let (community_public_key, community_private_key) = gen_keypair_str();

    // Create the main community if it doesn't exist
    let main_community: Community = match Community::read(&conn, 2) {
      Ok(c) => c,
      Err(_e) => {
        let default_community_name = "main";
        let community_form = CommunityForm {
          name: default_community_name.to_string(),
          title: "The Default Community".to_string(),
          description: Some("The Default Community".to_string()),
          category_id: 1,
          nsfw: false,
          creator_id: inserted_user.id,
          removed: None,
          deleted: None,
          updated: None,
          actor_id: make_apub_endpoint(EndpointType::Community, default_community_name).to_string(),
          local: true,
          private_key: Some(community_private_key),
          public_key: Some(community_public_key),
          last_refreshed_at: None,
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
        Err(_e) => return Err(APIError::err("community_follower_already_exists").into()),
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
          Err(_e) => return Err(APIError::err("community_moderator_already_exists").into()),
        };
    }

    // Return the jwt
    Ok(LoginResponse {
      jwt: inserted_user.jwt(),
    })
  }
}

impl Perform<LoginResponse> for Oper<SaveUserSettings> {
  fn perform(&self, conn: &PgConnection) -> Result<LoginResponse, Error> {
    let data: &SaveUserSettings = &self.data;

    let claims = match Claims::decode(&data.auth) {
      Ok(claims) => claims.claims,
      Err(_e) => return Err(APIError::err("not_logged_in").into()),
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
              return Err(APIError::err("passwords_dont_match").into());
            }

            // Check the old password
            match &data.old_password {
              Some(old_password) => {
                let valid: bool =
                  verify(old_password, &read_user.password_encrypted).unwrap_or(false);
                if !valid {
                  return Err(APIError::err("password_incorrect").into());
                }
                User_::update_password(&conn, user_id, &new_password)?.password_encrypted
              }
              None => return Err(APIError::err("password_incorrect").into()),
            }
          }
          None => return Err(APIError::err("passwords_dont_match").into()),
        }
      }
      None => read_user.password_encrypted,
    };

    let user_form = UserForm {
      name: read_user.name,
      email,
      matrix_user_id: data.matrix_user_id.to_owned(),
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
      actor_id: read_user.actor_id,
      bio: read_user.bio,
      local: read_user.local,
      private_key: read_user.private_key,
      public_key: read_user.public_key,
      last_refreshed_at: None,
    };

    let updated_user = match User_::update(&conn, user_id, &user_form) {
      Ok(user) => user,
      Err(e) => {
        let err_type = if e.to_string()
          == "duplicate key value violates unique constraint \"user__email_key\""
        {
          "email_already_exists"
        } else {
          "user_already_exists"
        };

        return Err(APIError::err(err_type).into());
      }
    };

    // Return the jwt
    Ok(LoginResponse {
      jwt: updated_user.jwt(),
    })
  }
}

impl Perform<GetUserDetailsResponse> for Oper<GetUserDetails> {
  fn perform(&self, conn: &PgConnection) -> Result<GetUserDetailsResponse, Error> {
    let data: &GetUserDetails = &self.data;

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
          Err(_e) => return Err(APIError::err("couldnt_find_that_username_or_email").into()),
        }
      }
    };

    let mut user_view = UserView::read(&conn, user_details_id)?;

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

    // If its not the same user, remove the email
    if let Some(user_id) = user_id {
      if user_details_id != user_id {
        user_view.email = None;
      }
    } else {
      user_view.email = None;
    }

    // Return the jwt
    Ok(GetUserDetailsResponse {
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
  fn perform(&self, conn: &PgConnection) -> Result<AddAdminResponse, Error> {
    let data: &AddAdmin = &self.data;

    let claims = match Claims::decode(&data.auth) {
      Ok(claims) => claims.claims,
      Err(_e) => return Err(APIError::err("not_logged_in").into()),
    };

    let user_id = claims.id;

    // Make sure user is an admin
    if !UserView::read(&conn, user_id)?.admin {
      return Err(APIError::err("not_an_admin").into());
    }

    match User_::add_admin(&conn, user_id, data.added) {
      Ok(user) => user,
      Err(_e) => return Err(APIError::err("couldnt_update_user").into()),
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

    Ok(AddAdminResponse { admins })
  }
}

impl Perform<BanUserResponse> for Oper<BanUser> {
  fn perform(&self, conn: &PgConnection) -> Result<BanUserResponse, Error> {
    let data: &BanUser = &self.data;

    let claims = match Claims::decode(&data.auth) {
      Ok(claims) => claims.claims,
      Err(_e) => return Err(APIError::err("not_logged_in").into()),
    };

    let user_id = claims.id;

    // Make sure user is an admin
    if !UserView::read(&conn, user_id)?.admin {
      return Err(APIError::err("not_an_admin").into());
    }

    match User_::ban_user(&conn, user_id, data.ban) {
      Ok(user) => user,
      Err(_e) => return Err(APIError::err("couldnt_update_user").into()),
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
      user: user_view,
      banned: data.ban,
    })
  }
}

impl Perform<GetRepliesResponse> for Oper<GetReplies> {
  fn perform(&self, conn: &PgConnection) -> Result<GetRepliesResponse, Error> {
    let data: &GetReplies = &self.data;

    let claims = match Claims::decode(&data.auth) {
      Ok(claims) => claims.claims,
      Err(_e) => return Err(APIError::err("not_logged_in").into()),
    };

    let user_id = claims.id;

    let sort = SortType::from_str(&data.sort)?;

    let replies = ReplyQueryBuilder::create(&conn, user_id)
      .sort(&sort)
      .unread_only(data.unread_only)
      .page(data.page)
      .limit(data.limit)
      .list()?;

    Ok(GetRepliesResponse { replies })
  }
}

impl Perform<GetUserMentionsResponse> for Oper<GetUserMentions> {
  fn perform(&self, conn: &PgConnection) -> Result<GetUserMentionsResponse, Error> {
    let data: &GetUserMentions = &self.data;

    let claims = match Claims::decode(&data.auth) {
      Ok(claims) => claims.claims,
      Err(_e) => return Err(APIError::err("not_logged_in").into()),
    };

    let user_id = claims.id;

    let sort = SortType::from_str(&data.sort)?;

    let mentions = UserMentionQueryBuilder::create(&conn, user_id)
      .sort(&sort)
      .unread_only(data.unread_only)
      .page(data.page)
      .limit(data.limit)
      .list()?;

    Ok(GetUserMentionsResponse { mentions })
  }
}

impl Perform<UserMentionResponse> for Oper<EditUserMention> {
  fn perform(&self, conn: &PgConnection) -> Result<UserMentionResponse, Error> {
    let data: &EditUserMention = &self.data;

    let claims = match Claims::decode(&data.auth) {
      Ok(claims) => claims.claims,
      Err(_e) => return Err(APIError::err("not_logged_in").into()),
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
        Err(_e) => return Err(APIError::err("couldnt_update_comment").into()),
      };

    let user_mention_view = UserMentionView::read(&conn, user_mention.id, user_id)?;

    Ok(UserMentionResponse {
      mention: user_mention_view,
    })
  }
}

impl Perform<GetRepliesResponse> for Oper<MarkAllAsRead> {
  fn perform(&self, conn: &PgConnection) -> Result<GetRepliesResponse, Error> {
    let data: &MarkAllAsRead = &self.data;

    let claims = match Claims::decode(&data.auth) {
      Ok(claims) => claims.claims,
      Err(_e) => return Err(APIError::err("not_logged_in").into()),
    };

    let user_id = claims.id;

    let replies = ReplyQueryBuilder::create(&conn, user_id)
      .unread_only(true)
      .page(1)
      .limit(999)
      .list()?;

    for reply in &replies {
      match Comment::mark_as_read(&conn, reply.id) {
        Ok(comment) => comment,
        Err(_e) => return Err(APIError::err("couldnt_update_comment").into()),
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
          Err(_e) => return Err(APIError::err("couldnt_update_comment").into()),
        };
    }

    // messages
    let messages = PrivateMessageQueryBuilder::create(&conn, user_id)
      .page(1)
      .limit(999)
      .unread_only(true)
      .list()?;

    for message in &messages {
      let private_message_form = PrivateMessageForm {
        content: None,
        creator_id: message.to_owned().creator_id,
        recipient_id: message.to_owned().recipient_id,
        deleted: None,
        read: Some(true),
        updated: None,
      };

      let _updated_message = match PrivateMessage::update(&conn, message.id, &private_message_form)
      {
        Ok(message) => message,
        Err(_e) => return Err(APIError::err("couldnt_update_private_message").into()),
      };
    }

    Ok(GetRepliesResponse { replies: vec![] })
  }
}

impl Perform<LoginResponse> for Oper<DeleteAccount> {
  fn perform(&self, conn: &PgConnection) -> Result<LoginResponse, Error> {
    let data: &DeleteAccount = &self.data;

    let claims = match Claims::decode(&data.auth) {
      Ok(claims) => claims.claims,
      Err(_e) => return Err(APIError::err("not_logged_in").into()),
    };

    let user_id = claims.id;

    let user: User_ = User_::read(&conn, user_id)?;

    // Verify the password
    let valid: bool = verify(&data.password, &user.password_encrypted).unwrap_or(false);
    if !valid {
      return Err(APIError::err("password_incorrect").into());
    }

    // Comments
    let comments = CommentQueryBuilder::create(&conn)
      .for_creator_id(user_id)
      .limit(std::i64::MAX)
      .list()?;

    for comment in &comments {
      let _updated_comment = match Comment::permadelete(&conn, comment.id) {
        Ok(comment) => comment,
        Err(_e) => return Err(APIError::err("couldnt_update_comment").into()),
      };
    }

    // Posts
    let posts = PostQueryBuilder::create(&conn)
      .sort(&SortType::New)
      .for_creator_id(user_id)
      .limit(std::i64::MAX)
      .list()?;

    for post in &posts {
      let _updated_post = match Post::permadelete(&conn, post.id) {
        Ok(post) => post,
        Err(_e) => return Err(APIError::err("couldnt_update_post").into()),
      };
    }

    Ok(LoginResponse {
      jwt: data.auth.to_owned(),
    })
  }
}

impl Perform<PasswordResetResponse> for Oper<PasswordReset> {
  fn perform(&self, conn: &PgConnection) -> Result<PasswordResetResponse, Error> {
    let data: &PasswordReset = &self.data;

    // Fetch that email
    let user: User_ = match User_::find_by_email(&conn, &data.email) {
      Ok(user) => user,
      Err(_e) => return Err(APIError::err("couldnt_find_that_username_or_email").into()),
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
      Err(_e) => return Err(APIError::err(&_e).into()),
    };

    Ok(PasswordResetResponse {})
  }
}

impl Perform<LoginResponse> for Oper<PasswordChange> {
  fn perform(&self, conn: &PgConnection) -> Result<LoginResponse, Error> {
    let data: &PasswordChange = &self.data;

    // Fetch the user_id from the token
    let user_id = PasswordResetRequest::read_from_token(&conn, &data.token)?.user_id;

    // Make sure passwords match
    if data.password != data.password_verify {
      return Err(APIError::err("passwords_dont_match").into());
    }

    // Update the user with the new password
    let updated_user = match User_::update_password(&conn, user_id, &data.password) {
      Ok(user) => user,
      Err(_e) => return Err(APIError::err("couldnt_update_user").into()),
    };

    // Return the jwt
    Ok(LoginResponse {
      jwt: updated_user.jwt(),
    })
  }
}

impl Perform<PrivateMessageResponse> for Oper<CreatePrivateMessage> {
  fn perform(&self, conn: &PgConnection) -> Result<PrivateMessageResponse, Error> {
    let data: &CreatePrivateMessage = &self.data;

    let claims = match Claims::decode(&data.auth) {
      Ok(claims) => claims.claims,
      Err(_e) => return Err(APIError::err("not_logged_in").into()),
    };

    let user_id = claims.id;

    let hostname = &format!("https://{}", Settings::get().hostname);

    // Check for a site ban
    if UserView::read(&conn, user_id)?.banned {
      return Err(APIError::err("site_ban").into());
    }

    let content_slurs_removed = remove_slurs(&data.content.to_owned());

    let private_message_form = PrivateMessageForm {
      content: Some(content_slurs_removed.to_owned()),
      creator_id: user_id,
      recipient_id: data.recipient_id,
      deleted: None,
      read: None,
      updated: None,
    };

    let inserted_private_message = match PrivateMessage::create(&conn, &private_message_form) {
      Ok(private_message) => private_message,
      Err(_e) => {
        return Err(APIError::err("couldnt_create_private_message").into());
      }
    };

    // Send notifications to the recipient
    let recipient_user = User_::read(&conn, data.recipient_id)?;
    if recipient_user.send_notifications_to_email {
      if let Some(email) = recipient_user.email {
        let subject = &format!(
          "{} - Private Message from {}",
          Settings::get().hostname,
          claims.username
        );
        let html = &format!(
          "<h1>Private Message</h1><br><div>{} - {}</div><br><a href={}/inbox>inbox</a>",
          claims.username, &content_slurs_removed, hostname
        );
        match send_email(subject, &email, &recipient_user.name, html) {
          Ok(_o) => _o,
          Err(e) => error!("{}", e),
        };
      }
    }

    let message = PrivateMessageView::read(&conn, inserted_private_message.id)?;

    Ok(PrivateMessageResponse { message })
  }
}

impl Perform<PrivateMessageResponse> for Oper<EditPrivateMessage> {
  fn perform(&self, conn: &PgConnection) -> Result<PrivateMessageResponse, Error> {
    let data: &EditPrivateMessage = &self.data;

    let claims = match Claims::decode(&data.auth) {
      Ok(claims) => claims.claims,
      Err(_e) => return Err(APIError::err("not_logged_in").into()),
    };

    let user_id = claims.id;

    let orig_private_message = PrivateMessage::read(&conn, data.edit_id)?;

    // Check for a site ban
    if UserView::read(&conn, user_id)?.banned {
      return Err(APIError::err("site_ban").into());
    }

    // Check to make sure they are the creator (or the recipient marking as read
    if !(data.read.is_some() && orig_private_message.recipient_id.eq(&user_id)
      || orig_private_message.creator_id.eq(&user_id))
    {
      return Err(APIError::err("no_private_message_edit_allowed").into());
    }

    let content_slurs_removed = match &data.content {
      Some(content) => Some(remove_slurs(content)),
      None => None,
    };

    let private_message_form = PrivateMessageForm {
      content: content_slurs_removed,
      creator_id: orig_private_message.creator_id,
      recipient_id: orig_private_message.recipient_id,
      deleted: data.deleted.to_owned(),
      read: data.read.to_owned(),
      updated: if data.read.is_some() {
        orig_private_message.updated
      } else {
        Some(naive_now())
      },
    };

    let _updated_private_message =
      match PrivateMessage::update(&conn, data.edit_id, &private_message_form) {
        Ok(private_message) => private_message,
        Err(_e) => return Err(APIError::err("couldnt_update_private_message").into()),
      };

    let message = PrivateMessageView::read(&conn, data.edit_id)?;

    Ok(PrivateMessageResponse { message })
  }
}

impl Perform<PrivateMessagesResponse> for Oper<GetPrivateMessages> {
  fn perform(&self, conn: &PgConnection) -> Result<PrivateMessagesResponse, Error> {
    let data: &GetPrivateMessages = &self.data;

    let claims = match Claims::decode(&data.auth) {
      Ok(claims) => claims.claims,
      Err(_e) => return Err(APIError::err("not_logged_in").into()),
    };

    let user_id = claims.id;

    let messages = PrivateMessageQueryBuilder::create(&conn, user_id)
      .page(data.page)
      .limit(data.limit)
      .unread_only(data.unread_only)
      .list()?;

    Ok(PrivateMessagesResponse { messages })
  }
}

impl Perform<UserJoinResponse> for Oper<UserJoin> {
  fn perform(&self, _conn: &PgConnection) -> Result<UserJoinResponse, Error> {
    let data: &UserJoin = &self.data;

    let claims = match Claims::decode(&data.auth) {
      Ok(claims) => claims.claims,
      Err(_e) => return Err(APIError::err("not_logged_in").into()),
    };

    let user_id = claims.id;
    Ok(UserJoinResponse { user_id })
  }
}
