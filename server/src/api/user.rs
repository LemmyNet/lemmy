use crate::{
  api::{APIError, Oper, Perform},
  apub::{
    extensions::signatures::generate_actor_keypair, make_apub_endpoint, ApubObjectType,
    EndpointType,
  },
  db::{
    comment::*, comment_view::*, community::*, community_view::*, moderator::*,
    password_reset_request::*, post::*, post_view::*, private_message::*, private_message_view::*,
    site::*, site_view::*, user::*, user_mention::*, user_mention_view::*, user_view::*, Crud,
    Followable, Joinable, ListingType, SortType,
  },
  generate_random_string, is_valid_username, naive_from_unix, naive_now, remove_slurs, send_email,
  settings::Settings,
  slur_check, slurs_vec_to_str,
  websocket::{
    server::{JoinUserRoom, SendAllMessage, SendUserRoomMessage},
    UserOperation, WebsocketInfo,
  },
  DbPool, LemmyError,
};
use bcrypt::verify;
use log::error;
use serde::{Deserialize, Serialize};
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

#[derive(Serialize, Deserialize, Clone)]
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

#[derive(Serialize, Deserialize, Clone)]
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
  pub message: PrivateMessageView,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct UserJoin {
  auth: String,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct UserJoinResponse {
  pub user_id: i32,
}

#[async_trait::async_trait(?Send)]
impl Perform for Oper<Login> {
  type Response = LoginResponse;

  async fn perform(
    &self,
    pool: DbPool,
    _websocket_info: Option<WebsocketInfo>,
  ) -> Result<LoginResponse, LemmyError> {
    let data: &Login = &self.data;

    // Fetch that username / email
    let username_or_email = data.username_or_email.clone();
    let user: User_ = match unblock!(
      pool,
      conn,
      User_::find_by_email_or_username(&conn, &username_or_email)
    ) {
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

#[async_trait::async_trait(?Send)]
impl Perform for Oper<Register> {
  type Response = LoginResponse;

  async fn perform(
    &self,
    pool: DbPool,
    _websocket_info: Option<WebsocketInfo>,
  ) -> Result<LoginResponse, LemmyError> {
    let data: &Register = &self.data;

    // Make sure site has open registration
    if let Ok(site) = unblock!(pool, conn, SiteView::read(&conn)) {
      let site: SiteView = site;
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
    let any_admins: bool = unblock!(pool, conn, UserView::admins(&conn)?.is_empty());
    if data.admin && !any_admins {
      return Err(APIError::err("admin_already_created").into());
    }

    let user_keypair = generate_actor_keypair()?;
    if !is_valid_username(&data.username) {
      return Err(APIError::err("invalid_username").into());
    }

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
      private_key: Some(user_keypair.private_key),
      public_key: Some(user_keypair.public_key),
      last_refreshed_at: None,
    };

    // Create the user
    let res: Result<_, diesel::result::Error> =
      unblock!(pool, conn, User_::register(&conn, &user_form));
    let inserted_user: User_ = match res {
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

    let main_community_keypair = generate_actor_keypair()?;

    // Create the main community if it doesn't exist
    let main_community: Community = match unblock!(pool, conn, Community::read(&conn, 2)) {
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
          private_key: Some(main_community_keypair.private_key),
          public_key: Some(main_community_keypair.public_key),
          last_refreshed_at: None,
          published: None,
        };
        unblock!(pool, conn, Community::create(&conn, &community_form)?)
      }
    };

    // Sign them up for main community no matter what
    let community_follower_form = CommunityFollowerForm {
      community_id: main_community.id,
      user_id: inserted_user.id,
    };

    let _inserted_community_follower: CommunityFollower = match unblock!(
      pool,
      conn,
      CommunityFollower::follow(&conn, &community_follower_form)
    ) {
      Ok(user) => user,
      Err(_e) => return Err(APIError::err("community_follower_already_exists").into()),
    };

    // If its an admin, add them as a mod and follower to main
    if data.admin {
      let community_moderator_form = CommunityModeratorForm {
        community_id: main_community.id,
        user_id: inserted_user.id,
      };

      let _inserted_community_moderator: CommunityModerator = match unblock!(
        pool,
        conn,
        CommunityModerator::join(&conn, &community_moderator_form)
      ) {
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

#[async_trait::async_trait(?Send)]
impl Perform for Oper<SaveUserSettings> {
  type Response = LoginResponse;

  async fn perform(
    &self,
    pool: DbPool,
    _websocket_info: Option<WebsocketInfo>,
  ) -> Result<LoginResponse, LemmyError> {
    let data: &SaveUserSettings = &self.data;

    let claims = match Claims::decode(&data.auth) {
      Ok(claims) => claims.claims,
      Err(_e) => return Err(APIError::err("not_logged_in").into()),
    };

    let user_id = claims.id;

    let read_user: User_ = unblock!(pool, conn, User_::read(&conn, user_id)?);

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
                let new_password = new_password.to_owned();
                let user: User_ = unblock!(
                  pool,
                  conn,
                  User_::update_password(&conn, user_id, &new_password)?
                );
                user.password_encrypted
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

    let res: Result<_, diesel::result::Error> =
      unblock!(pool, conn, User_::update(&conn, user_id, &user_form));
    let updated_user: User_ = match res {
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

#[async_trait::async_trait(?Send)]
impl Perform for Oper<GetUserDetails> {
  type Response = GetUserDetailsResponse;

  async fn perform(
    &self,
    pool: DbPool,
    _websocket_info: Option<WebsocketInfo>,
  ) -> Result<GetUserDetailsResponse, LemmyError> {
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

    let username = data
      .username
      .to_owned()
      .unwrap_or_else(|| "admin".to_string());
    let user_details_id = match data.user_id {
      Some(id) => id,
      None => {
        let user: Result<User_, _> = unblock!(pool, conn, User_::read_from_name(&conn, &username,));
        match user {
          Ok(user) => user.id,
          Err(_e) => return Err(APIError::err("couldnt_find_that_username_or_email").into()),
        }
      }
    };

    let mut user_view: UserView = unblock!(pool, conn, UserView::read(&conn, user_details_id)?);

    let page = data.page;
    let limit = data.limit;
    let saved_only = data.saved_only;
    let community_id = data.community_id;
    let (posts, comments): (Vec<PostView>, Vec<CommentView>) = unblock!(pool, conn, {
      let mut posts_query = PostQueryBuilder::create(&conn)
        .sort(&sort)
        .show_nsfw(show_nsfw)
        .saved_only(saved_only)
        .for_community_id(community_id)
        .my_user_id(user_id)
        .page(page)
        .limit(limit);

      let mut comments_query = CommentQueryBuilder::create(&conn)
        .sort(&sort)
        .saved_only(saved_only)
        .my_user_id(user_id)
        .page(page)
        .limit(limit);

      // If its saved only, you don't care what creator it was
      // Or, if its not saved, then you only want it for that specific creator
      if !saved_only {
        posts_query = posts_query.for_creator_id(user_details_id);
        comments_query = comments_query.for_creator_id(user_details_id);
      }

      let posts = posts_query.list()?;
      let comments = comments_query.list()?;

      (posts, comments)
    });

    let follows: Vec<CommunityFollowerView> = unblock!(
      pool,
      conn,
      CommunityFollowerView::for_user(&conn, user_details_id)?
    );
    let moderates: Vec<CommunityModeratorView> = unblock!(
      pool,
      conn,
      CommunityModeratorView::for_user(&conn, user_details_id)?
    );

    let site: Site = unblock!(pool, conn, Site::read(&conn, 1)?);
    let site_creator_id = site.creator_id;

    let mut admins: Vec<UserView> = unblock!(pool, conn, UserView::admins(&conn)?);
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

#[async_trait::async_trait(?Send)]
impl Perform for Oper<AddAdmin> {
  type Response = AddAdminResponse;

  async fn perform(
    &self,
    pool: DbPool,
    websocket_info: Option<WebsocketInfo>,
  ) -> Result<AddAdminResponse, LemmyError> {
    let data: &AddAdmin = &self.data;

    let claims = match Claims::decode(&data.auth) {
      Ok(claims) => claims.claims,
      Err(_e) => return Err(APIError::err("not_logged_in").into()),
    };

    let user_id = claims.id;

    // Make sure user is an admin
    if unblock!(pool, conn, !UserView::read(&conn, user_id)?.admin) {
      return Err(APIError::err("not_an_admin").into());
    }

    let added = data.added;
    if unblock!(pool, conn, User_::add_admin(&conn, user_id, added).is_err()) {
      return Err(APIError::err("couldnt_update_user").into());
    }

    // Mod tables
    let form = ModAddForm {
      mod_user_id: user_id,
      other_user_id: data.user_id,
      removed: Some(!data.added),
    };

    let _: ModAdd = unblock!(pool, conn, ModAdd::create(&conn, &form)?);

    let site_creator_id: i32 = unblock!(pool, conn, Site::read(&conn, 1)?.creator_id);

    let mut admins: Vec<UserView> = unblock!(pool, conn, UserView::admins(&conn)?);
    let creator_index = admins.iter().position(|r| r.id == site_creator_id).unwrap();
    let creator_user = admins.remove(creator_index);
    admins.insert(0, creator_user);

    let res = AddAdminResponse { admins };

    if let Some(ws) = websocket_info {
      ws.chatserver.do_send(SendAllMessage {
        op: UserOperation::AddAdmin,
        response: res.clone(),
        my_id: ws.id,
      });
    }

    Ok(res)
  }
}

#[async_trait::async_trait(?Send)]
impl Perform for Oper<BanUser> {
  type Response = BanUserResponse;

  async fn perform(
    &self,
    pool: DbPool,
    websocket_info: Option<WebsocketInfo>,
  ) -> Result<BanUserResponse, LemmyError> {
    let data: &BanUser = &self.data;

    let claims = match Claims::decode(&data.auth) {
      Ok(claims) => claims.claims,
      Err(_e) => return Err(APIError::err("not_logged_in").into()),
    };

    let user_id = claims.id;

    // Make sure user is an admin
    if unblock!(pool, conn, !UserView::read(&conn, user_id)?.admin) {
      return Err(APIError::err("not_an_admin").into());
    }

    let ban = data.ban;
    if unblock!(pool, conn, User_::ban_user(&conn, user_id, ban).is_err()) {
      return Err(APIError::err("couldnt_update_user").into());
    }

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

    let _: ModBan = unblock!(pool, conn, ModBan::create(&conn, &form)?);

    let user_id = data.user_id;
    let user_view: UserView = unblock!(pool, conn, UserView::read(&conn, user_id)?);

    let res = BanUserResponse {
      user: user_view,
      banned: data.ban,
    };

    if let Some(ws) = websocket_info {
      ws.chatserver.do_send(SendAllMessage {
        op: UserOperation::BanUser,
        response: res.clone(),
        my_id: ws.id,
      });
    }

    Ok(res)
  }
}

#[async_trait::async_trait(?Send)]
impl Perform for Oper<GetReplies> {
  type Response = GetRepliesResponse;

  async fn perform(
    &self,
    pool: DbPool,
    _websocket_info: Option<WebsocketInfo>,
  ) -> Result<GetRepliesResponse, LemmyError> {
    let data: &GetReplies = &self.data;

    let claims = match Claims::decode(&data.auth) {
      Ok(claims) => claims.claims,
      Err(_e) => return Err(APIError::err("not_logged_in").into()),
    };

    let user_id = claims.id;

    let sort = SortType::from_str(&data.sort)?;

    let page = data.page;
    let limit = data.limit;
    let unread_only = data.unread_only;
    let replies: Vec<_> = unblock!(
      pool,
      conn,
      ReplyQueryBuilder::create(&conn, user_id)
        .sort(&sort)
        .unread_only(unread_only)
        .page(page)
        .limit(limit)
        .list()?
    );

    Ok(GetRepliesResponse { replies })
  }
}

#[async_trait::async_trait(?Send)]
impl Perform for Oper<GetUserMentions> {
  type Response = GetUserMentionsResponse;

  async fn perform(
    &self,
    pool: DbPool,
    _websocket_info: Option<WebsocketInfo>,
  ) -> Result<GetUserMentionsResponse, LemmyError> {
    let data: &GetUserMentions = &self.data;

    let claims = match Claims::decode(&data.auth) {
      Ok(claims) => claims.claims,
      Err(_e) => return Err(APIError::err("not_logged_in").into()),
    };

    let user_id = claims.id;

    let sort = SortType::from_str(&data.sort)?;

    let page = data.page;
    let limit = data.limit;
    let unread_only = data.unread_only;
    let mentions: Vec<_> = unblock!(
      pool,
      conn,
      UserMentionQueryBuilder::create(&conn, user_id)
        .sort(&sort)
        .unread_only(unread_only)
        .page(page)
        .limit(limit)
        .list()?
    );

    Ok(GetUserMentionsResponse { mentions })
  }
}

#[async_trait::async_trait(?Send)]
impl Perform for Oper<EditUserMention> {
  type Response = UserMentionResponse;

  async fn perform(
    &self,
    pool: DbPool,
    _websocket_info: Option<WebsocketInfo>,
  ) -> Result<UserMentionResponse, LemmyError> {
    let data: &EditUserMention = &self.data;

    let claims = match Claims::decode(&data.auth) {
      Ok(claims) => claims.claims,
      Err(_e) => return Err(APIError::err("not_logged_in").into()),
    };

    let user_id = claims.id;

    let user_mention_id = data.user_mention_id;
    let user_mention: UserMention =
      unblock!(pool, conn, UserMention::read(&conn, user_mention_id)?);

    let user_mention_form = UserMentionForm {
      recipient_id: user_id,
      comment_id: user_mention.comment_id,
      read: data.read.to_owned(),
    };

    let user_mention_id = user_mention.id;
    let _updated_user_mention: UserMention = match unblock!(
      pool,
      conn,
      UserMention::update(&conn, user_mention_id, &user_mention_form)
    ) {
      Ok(comment) => comment,
      Err(_e) => return Err(APIError::err("couldnt_update_comment").into()),
    };

    let user_mention_id = user_mention.id;
    let user_mention_view: UserMentionView = unblock!(
      pool,
      conn,
      UserMentionView::read(&conn, user_mention_id, user_id)?
    );

    Ok(UserMentionResponse {
      mention: user_mention_view,
    })
  }
}

#[async_trait::async_trait(?Send)]
impl Perform for Oper<MarkAllAsRead> {
  type Response = GetRepliesResponse;

  async fn perform(
    &self,
    pool: DbPool,
    _websocket_info: Option<WebsocketInfo>,
  ) -> Result<GetRepliesResponse, LemmyError> {
    let data: &MarkAllAsRead = &self.data;

    let claims = match Claims::decode(&data.auth) {
      Ok(claims) => claims.claims,
      Err(_e) => return Err(APIError::err("not_logged_in").into()),
    };

    let user_id = claims.id;

    let replies: Vec<ReplyView> = unblock!(
      pool,
      conn,
      ReplyQueryBuilder::create(&conn, user_id)
        .unread_only(true)
        .page(1)
        .limit(999)
        .list()?
    );

    // TODO: this should probably be a bulk operation
    for reply in &replies {
      let reply_id = reply.id;
      if unblock!(pool, conn, Comment::mark_as_read(&conn, reply_id).is_err()) {
        return Err(APIError::err("couldnt_update_comment").into());
      }
    }

    // Mentions
    let mentions: Vec<UserMentionView> = unblock!(
      pool,
      conn,
      UserMentionQueryBuilder::create(&conn, user_id)
        .unread_only(true)
        .page(1)
        .limit(999)
        .list()?
    );

    // TODO: this should probably be a bulk operation
    for mention in &mentions {
      let mention_form = UserMentionForm {
        recipient_id: mention.to_owned().recipient_id,
        comment_id: mention.to_owned().id,
        read: Some(true),
      };

      let user_mention_id = mention.user_mention_id;
      if unblock!(
        pool,
        conn,
        UserMention::update(&conn, user_mention_id, &mention_form).is_err()
      ) {
        return Err(APIError::err("couldnt_update_comment").into());
      }
    }

    // messages
    let messages: Vec<PrivateMessageView> = unblock!(
      pool,
      conn,
      PrivateMessageQueryBuilder::create(&conn, user_id)
        .page(1)
        .limit(999)
        .unread_only(true)
        .list()?
    );

    // TODO: this should probably be a bulk operation
    for message in &messages {
      let private_message_form = PrivateMessageForm {
        content: message.to_owned().content,
        creator_id: message.to_owned().creator_id,
        recipient_id: message.to_owned().recipient_id,
        deleted: None,
        read: Some(true),
        updated: None,
        ap_id: message.to_owned().ap_id,
        local: message.local,
        published: None,
      };

      let message_id = message.id;
      if unblock!(
        pool,
        conn,
        PrivateMessage::update(&conn, message_id, &private_message_form).is_err()
      ) {
        return Err(APIError::err("couldnt_update_private_message").into());
      }
    }

    Ok(GetRepliesResponse { replies: vec![] })
  }
}

#[async_trait::async_trait(?Send)]
impl Perform for Oper<DeleteAccount> {
  type Response = LoginResponse;

  async fn perform(
    &self,
    pool: DbPool,
    _websocket_info: Option<WebsocketInfo>,
  ) -> Result<LoginResponse, LemmyError> {
    let data: &DeleteAccount = &self.data;

    let claims = match Claims::decode(&data.auth) {
      Ok(claims) => claims.claims,
      Err(_e) => return Err(APIError::err("not_logged_in").into()),
    };

    let user_id = claims.id;

    let user: User_ = unblock!(pool, conn, User_::read(&conn, user_id)?);

    // Verify the password
    let valid: bool = verify(&data.password, &user.password_encrypted).unwrap_or(false);
    if !valid {
      return Err(APIError::err("password_incorrect").into());
    }

    // Comments
    let comments: Vec<CommentView> = unblock!(
      pool,
      conn,
      CommentQueryBuilder::create(&conn)
        .for_creator_id(user_id)
        .limit(std::i64::MAX)
        .list()?
    );

    // TODO: this should probably be a bulk operation
    for comment in &comments {
      let comment_id = comment.id;
      if unblock!(pool, conn, Comment::permadelete(&conn, comment_id).is_err()) {
        return Err(APIError::err("couldnt_update_comment").into());
      }
    }

    // Posts
    let posts: Vec<PostView> = unblock!(
      pool,
      conn,
      PostQueryBuilder::create(&conn)
        .sort(&SortType::New)
        .for_creator_id(user_id)
        .limit(std::i64::MAX)
        .list()?
    );

    // TODO: this should probably be a bulk operation
    for post in &posts {
      let post_id = post.id;
      if unblock!(pool, conn, Post::permadelete(&conn, post_id).is_err()) {
        return Err(APIError::err("couldnt_update_post").into());
      }
    }

    Ok(LoginResponse {
      jwt: data.auth.to_owned(),
    })
  }
}

#[async_trait::async_trait(?Send)]
impl Perform for Oper<PasswordReset> {
  type Response = PasswordResetResponse;

  async fn perform(
    &self,
    pool: DbPool,
    _websocket_info: Option<WebsocketInfo>,
  ) -> Result<PasswordResetResponse, LemmyError> {
    let data: &PasswordReset = &self.data;

    // Fetch that email
    let email = data.email.clone();
    let user: User_ = match unblock!(pool, conn, User_::find_by_email(&conn, &email)) {
      Ok(user) => user,
      Err(_e) => return Err(APIError::err("couldnt_find_that_username_or_email").into()),
    };

    // Generate a random token
    let token = generate_random_string();

    // Insert the row
    let token2 = token.clone();
    let user_id = user.id;
    let _: PasswordResetRequest = unblock!(
      pool,
      conn,
      PasswordResetRequest::create_token(&conn, user_id, &token2)?
    );

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

#[async_trait::async_trait(?Send)]
impl Perform for Oper<PasswordChange> {
  type Response = LoginResponse;

  async fn perform(
    &self,
    pool: DbPool,
    _websocket_info: Option<WebsocketInfo>,
  ) -> Result<LoginResponse, LemmyError> {
    let data: &PasswordChange = &self.data;

    // Fetch the user_id from the token
    let token = data.token.clone();
    let user_id: i32 = unblock!(
      pool,
      conn,
      PasswordResetRequest::read_from_token(&conn, &token)?.user_id
    );

    // Make sure passwords match
    if data.password != data.password_verify {
      return Err(APIError::err("passwords_dont_match").into());
    }

    // Update the user with the new password
    let password = data.password.clone();
    let updated_user: User_ = match unblock!(
      pool,
      conn,
      User_::update_password(&conn, user_id, &password)
    ) {
      Ok(user) => user,
      Err(_e) => return Err(APIError::err("couldnt_update_user").into()),
    };

    // Return the jwt
    Ok(LoginResponse {
      jwt: updated_user.jwt(),
    })
  }
}

#[async_trait::async_trait(?Send)]
impl Perform for Oper<CreatePrivateMessage> {
  type Response = PrivateMessageResponse;

  async fn perform(
    &self,
    pool: DbPool,
    websocket_info: Option<WebsocketInfo>,
  ) -> Result<PrivateMessageResponse, LemmyError> {
    let data: &CreatePrivateMessage = &self.data;

    let claims = match Claims::decode(&data.auth) {
      Ok(claims) => claims.claims,
      Err(_e) => return Err(APIError::err("not_logged_in").into()),
    };

    let user_id = claims.id;

    let hostname = &format!("https://{}", Settings::get().hostname);

    // Check for a site ban
    let user: User_ = unblock!(pool, conn, User_::read(&conn, user_id)?);
    if user.banned {
      return Err(APIError::err("site_ban").into());
    }

    let content_slurs_removed = remove_slurs(&data.content.to_owned());

    let private_message_form = PrivateMessageForm {
      content: content_slurs_removed.to_owned(),
      creator_id: user_id,
      recipient_id: data.recipient_id,
      deleted: None,
      read: None,
      updated: None,
      ap_id: "http://fake.com".into(),
      local: true,
      published: None,
    };

    let inserted_private_message: PrivateMessage = match unblock!(
      pool,
      conn,
      PrivateMessage::create(&conn, &private_message_form)
    ) {
      Ok(private_message) => private_message,
      Err(_e) => {
        return Err(APIError::err("couldnt_create_private_message").into());
      }
    };

    let inserted_private_message_id = inserted_private_message.id;
    let updated_private_message: PrivateMessage = match unblock!(
      pool,
      conn,
      PrivateMessage::update_ap_id(&conn, inserted_private_message_id)
    ) {
      Ok(private_message) => private_message,
      Err(_e) => return Err(APIError::err("couldnt_create_private_message").into()),
    };

    updated_private_message
      .send_create(&user, &self.client, pool.clone())
      .await?;

    // Send notifications to the recipient
    let recipient_id = data.recipient_id;
    let recipient_user: User_ = unblock!(pool, conn, User_::read(&conn, recipient_id)?);
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

    let message: PrivateMessageView = unblock!(
      pool,
      conn,
      PrivateMessageView::read(&conn, inserted_private_message.id)?
    );

    let res = PrivateMessageResponse { message };

    if let Some(ws) = websocket_info {
      ws.chatserver.do_send(SendUserRoomMessage {
        op: UserOperation::CreatePrivateMessage,
        response: res.clone(),
        recipient_id: recipient_user.id,
        my_id: ws.id,
      });
    }

    Ok(res)
  }
}

#[async_trait::async_trait(?Send)]
impl Perform for Oper<EditPrivateMessage> {
  type Response = PrivateMessageResponse;

  async fn perform(
    &self,
    pool: DbPool,
    websocket_info: Option<WebsocketInfo>,
  ) -> Result<PrivateMessageResponse, LemmyError> {
    let data: &EditPrivateMessage = &self.data;

    let claims = match Claims::decode(&data.auth) {
      Ok(claims) => claims.claims,
      Err(_e) => return Err(APIError::err("not_logged_in").into()),
    };

    let user_id = claims.id;

    let edit_id = data.edit_id;
    let orig_private_message: PrivateMessage =
      unblock!(pool, conn, PrivateMessage::read(&conn, edit_id)?);

    // Check for a site ban
    let user: User_ = unblock!(pool, conn, User_::read(&conn, user_id)?);
    if user.banned {
      return Err(APIError::err("site_ban").into());
    }

    // Check to make sure they are the creator (or the recipient marking as read
    if !(data.read.is_some() && orig_private_message.recipient_id.eq(&user_id)
      || orig_private_message.creator_id.eq(&user_id))
    {
      return Err(APIError::err("no_private_message_edit_allowed").into());
    }

    let content_slurs_removed = match &data.content {
      Some(content) => remove_slurs(content),
      None => orig_private_message.content,
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
      ap_id: orig_private_message.ap_id,
      local: orig_private_message.local,
      published: None,
    };

    let edit_id = data.edit_id;
    let updated_private_message: PrivateMessage = match unblock!(
      pool,
      conn,
      PrivateMessage::update(&conn, edit_id, &private_message_form)
    ) {
      Ok(private_message) => private_message,
      Err(_e) => return Err(APIError::err("couldnt_update_private_message").into()),
    };

    if let Some(deleted) = data.deleted.to_owned() {
      if deleted {
        updated_private_message
          .send_delete(&user, &self.client, pool.clone())
          .await?;
      } else {
        updated_private_message
          .send_undo_delete(&user, &self.client, pool.clone())
          .await?;
      }
    } else {
      updated_private_message
        .send_update(&user, &self.client, pool.clone())
        .await?;
    }

    let edit_id = data.edit_id;
    let message: PrivateMessageView =
      unblock!(pool, conn, PrivateMessageView::read(&conn, edit_id)?);

    let res = PrivateMessageResponse { message };

    if let Some(ws) = websocket_info {
      ws.chatserver.do_send(SendUserRoomMessage {
        op: UserOperation::EditPrivateMessage,
        response: res.clone(),
        recipient_id: orig_private_message.recipient_id,
        my_id: ws.id,
      });
    }

    Ok(res)
  }
}

#[async_trait::async_trait(?Send)]
impl Perform for Oper<GetPrivateMessages> {
  type Response = PrivateMessagesResponse;

  async fn perform(
    &self,
    pool: DbPool,
    _websocket_info: Option<WebsocketInfo>,
  ) -> Result<PrivateMessagesResponse, LemmyError> {
    let data: &GetPrivateMessages = &self.data;

    let claims = match Claims::decode(&data.auth) {
      Ok(claims) => claims.claims,
      Err(_e) => return Err(APIError::err("not_logged_in").into()),
    };

    let user_id = claims.id;

    let page = data.page;
    let limit = data.limit;
    let unread_only = data.unread_only;
    let messages: Vec<_> = unblock!(
      pool,
      conn,
      PrivateMessageQueryBuilder::create(&conn, user_id)
        .page(page)
        .limit(limit)
        .unread_only(unread_only)
        .list()?
    );

    Ok(PrivateMessagesResponse { messages })
  }
}

#[async_trait::async_trait(?Send)]
impl Perform for Oper<UserJoin> {
  type Response = UserJoinResponse;

  async fn perform(
    &self,
    _pool: DbPool,
    websocket_info: Option<WebsocketInfo>,
  ) -> Result<UserJoinResponse, LemmyError> {
    let data: &UserJoin = &self.data;

    let claims = match Claims::decode(&data.auth) {
      Ok(claims) => claims.claims,
      Err(_e) => return Err(APIError::err("not_logged_in").into()),
    };

    let user_id = claims.id;

    if let Some(ws) = websocket_info {
      if let Some(id) = ws.id {
        ws.chatserver.do_send(JoinUserRoom { user_id, id });
      }
    }

    Ok(UserJoinResponse { user_id })
  }
}
