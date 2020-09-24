use crate::{
  captcha_espeak_wav_base64,
  claims::Claims,
  get_user_from_jwt,
  get_user_from_jwt_opt,
  is_admin,
  Perform,
};
use actix_web::web::Data;
use anyhow::Context;
use bcrypt::verify;
use captcha::{gen, Difficulty};
use chrono::Duration;
use lemmy_apub::ApubObjectType;
use lemmy_db::{
  comment::*,
  comment_view::*,
  community::*,
  community_view::*,
  diesel_option_overwrite,
  moderator::*,
  naive_now,
  password_reset_request::*,
  post::*,
  post_view::*,
  private_message::*,
  private_message_view::*,
  site::*,
  site_view::*,
  user::*,
  user_mention::*,
  user_mention_view::*,
  user_view::*,
  Crud,
  Followable,
  Joinable,
  ListingType,
  SortType,
};
use lemmy_structs::{blocking, user::*};
use lemmy_utils::{
  apub::{generate_actor_keypair, make_apub_endpoint, EndpointType},
  email::send_email,
  location_info,
  settings::Settings,
  utils::{
    check_slurs,
    generate_random_string,
    is_valid_preferred_username,
    is_valid_username,
    naive_from_unix,
    remove_slurs,
  },
  APIError,
  ConnectionId,
  LemmyError,
};
use lemmy_websocket::{
  messages::{CaptchaItem, CheckCaptcha, JoinUserRoom, SendAllMessage, SendUserRoomMessage},
  LemmyContext,
  UserOperation,
};
use log::error;
use std::str::FromStr;

#[async_trait::async_trait(?Send)]
impl Perform for Login {
  type Response = LoginResponse;

  async fn perform(
    &self,
    context: &Data<LemmyContext>,
    _websocket_id: Option<ConnectionId>,
  ) -> Result<LoginResponse, LemmyError> {
    let data: &Login = &self;

    // Fetch that username / email
    let username_or_email = data.username_or_email.clone();
    let user = match blocking(context.pool(), move |conn| {
      User_::find_by_email_or_username(conn, &username_or_email)
    })
    .await?
    {
      Ok(user) => user,
      Err(_e) => return Err(APIError::err("couldnt_find_that_username_or_email").into()),
    };

    // Verify the password
    let valid: bool = verify(&data.password, &user.password_encrypted).unwrap_or(false);
    if !valid {
      return Err(APIError::err("password_incorrect").into());
    }

    // Return the jwt
    Ok(LoginResponse {
      jwt: Claims::jwt(user, Settings::get().hostname)?,
    })
  }
}

#[async_trait::async_trait(?Send)]
impl Perform for Register {
  type Response = LoginResponse;

  async fn perform(
    &self,
    context: &Data<LemmyContext>,
    _websocket_id: Option<ConnectionId>,
  ) -> Result<LoginResponse, LemmyError> {
    let data: &Register = &self;

    // Make sure site has open registration
    if let Ok(site) = blocking(context.pool(), move |conn| SiteView::read(conn)).await? {
      let site: SiteView = site;
      if !site.open_registration {
        return Err(APIError::err("registration_closed").into());
      }
    }

    // Make sure passwords match
    if data.password != data.password_verify {
      return Err(APIError::err("passwords_dont_match").into());
    }

    // If its not the admin, check the captcha
    if !data.admin && Settings::get().captcha.enabled {
      let check = context
        .chat_server()
        .send(CheckCaptcha {
          uuid: data
            .captcha_uuid
            .to_owned()
            .unwrap_or_else(|| "".to_string()),
          answer: data
            .captcha_answer
            .to_owned()
            .unwrap_or_else(|| "".to_string()),
        })
        .await?;
      if !check {
        return Err(APIError::err("captcha_incorrect").into());
      }
    }

    check_slurs(&data.username)?;

    // Make sure there are no admins
    let any_admins = blocking(context.pool(), move |conn| {
      UserView::admins(conn).map(|a| a.is_empty())
    })
    .await??;
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
      email: Some(data.email.to_owned()),
      matrix_user_id: None,
      avatar: None,
      banner: None,
      password_encrypted: data.password.to_owned(),
      preferred_username: None,
      published: None,
      updated: None,
      admin: data.admin,
      banned: false,
      show_nsfw: data.show_nsfw,
      theme: "darkly".into(),
      default_sort_type: SortType::Active as i16,
      default_listing_type: ListingType::Subscribed as i16,
      lang: "browser".into(),
      show_avatars: true,
      send_notifications_to_email: false,
      actor_id: Some(make_apub_endpoint(EndpointType::User, &data.username).to_string()),
      bio: None,
      local: true,
      private_key: Some(user_keypair.private_key),
      public_key: Some(user_keypair.public_key),
      last_refreshed_at: None,
    };

    // Create the user
    let inserted_user = match blocking(context.pool(), move |conn| {
      User_::register(conn, &user_form)
    })
    .await?
    {
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
    let main_community =
      match blocking(context.pool(), move |conn| Community::read(conn, 2)).await? {
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
            actor_id: Some(
              make_apub_endpoint(EndpointType::Community, default_community_name).to_string(),
            ),
            local: true,
            private_key: Some(main_community_keypair.private_key),
            public_key: Some(main_community_keypair.public_key),
            last_refreshed_at: None,
            published: None,
            icon: None,
            banner: None,
          };
          blocking(context.pool(), move |conn| {
            Community::create(conn, &community_form)
          })
          .await??
        }
      };

    // Sign them up for main community no matter what
    let community_follower_form = CommunityFollowerForm {
      community_id: main_community.id,
      user_id: inserted_user.id,
    };

    let follow = move |conn: &'_ _| CommunityFollower::follow(conn, &community_follower_form);
    if blocking(context.pool(), follow).await?.is_err() {
      return Err(APIError::err("community_follower_already_exists").into());
    };

    // If its an admin, add them as a mod and follower to main
    if data.admin {
      let community_moderator_form = CommunityModeratorForm {
        community_id: main_community.id,
        user_id: inserted_user.id,
      };

      let join = move |conn: &'_ _| CommunityModerator::join(conn, &community_moderator_form);
      if blocking(context.pool(), join).await?.is_err() {
        return Err(APIError::err("community_moderator_already_exists").into());
      }
    }

    // Return the jwt
    Ok(LoginResponse {
      jwt: Claims::jwt(inserted_user, Settings::get().hostname)?,
    })
  }
}

#[async_trait::async_trait(?Send)]
impl Perform for GetCaptcha {
  type Response = GetCaptchaResponse;

  async fn perform(
    &self,
    context: &Data<LemmyContext>,
    _websocket_id: Option<ConnectionId>,
  ) -> Result<Self::Response, LemmyError> {
    let captcha_settings = Settings::get().captcha;

    if !captcha_settings.enabled {
      return Ok(GetCaptchaResponse { ok: None });
    }

    let captcha = match captcha_settings.difficulty.as_str() {
      "easy" => gen(Difficulty::Easy),
      "medium" => gen(Difficulty::Medium),
      "hard" => gen(Difficulty::Hard),
      _ => gen(Difficulty::Medium),
    };

    let answer = captcha.chars_as_string();

    let png_byte_array = captcha.as_png().expect("failed to generate captcha");

    let png = base64::encode(png_byte_array);

    let uuid = uuid::Uuid::new_v4().to_string();

    let wav = captcha_espeak_wav_base64(&answer).ok();

    let captcha_item = CaptchaItem {
      answer,
      uuid: uuid.to_owned(),
      expires: naive_now() + Duration::minutes(10), // expires in 10 minutes
    };

    // Stores the captcha item on the queue
    context.chat_server().do_send(captcha_item);

    Ok(GetCaptchaResponse {
      ok: Some(CaptchaResponse { png, uuid, wav }),
    })
  }
}

#[async_trait::async_trait(?Send)]
impl Perform for SaveUserSettings {
  type Response = LoginResponse;

  async fn perform(
    &self,
    context: &Data<LemmyContext>,
    _websocket_id: Option<ConnectionId>,
  ) -> Result<LoginResponse, LemmyError> {
    let data: &SaveUserSettings = &self;
    let user = get_user_from_jwt(&data.auth, context.pool()).await?;

    let user_id = user.id;
    let read_user = blocking(context.pool(), move |conn| User_::read(conn, user_id)).await??;

    let bio = match &data.bio {
      Some(bio) => {
        if bio.chars().count() <= 300 {
          Some(bio.to_owned())
        } else {
          return Err(APIError::err("bio_length_overflow").into());
        }
      }
      None => read_user.bio,
    };

    let avatar = diesel_option_overwrite(&data.avatar);
    let banner = diesel_option_overwrite(&data.banner);
    let email = diesel_option_overwrite(&data.email);

    // The DB constraint should stop too many characters
    let preferred_username = match &data.preferred_username {
      Some(preferred_username) => {
        if !is_valid_preferred_username(preferred_username.trim()) {
          return Err(APIError::err("invalid_username").into());
        }
        Some(preferred_username.trim().to_string())
      }
      None => read_user.preferred_username,
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
                let user = blocking(context.pool(), move |conn| {
                  User_::update_password(conn, user_id, &new_password)
                })
                .await??;
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
      avatar,
      banner,
      password_encrypted,
      preferred_username,
      published: Some(read_user.published),
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
      actor_id: Some(read_user.actor_id),
      bio,
      local: read_user.local,
      private_key: read_user.private_key,
      public_key: read_user.public_key,
      last_refreshed_at: None,
    };

    let res = blocking(context.pool(), move |conn| {
      User_::update(conn, user_id, &user_form)
    })
    .await?;
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
      jwt: Claims::jwt(updated_user, Settings::get().hostname)?,
    })
  }
}

#[async_trait::async_trait(?Send)]
impl Perform for GetUserDetails {
  type Response = GetUserDetailsResponse;

  async fn perform(
    &self,
    context: &Data<LemmyContext>,
    _websocket_id: Option<ConnectionId>,
  ) -> Result<GetUserDetailsResponse, LemmyError> {
    let data: &GetUserDetails = &self;
    let user = get_user_from_jwt_opt(&data.auth, context.pool()).await?;

    let show_nsfw = match &user {
      Some(user) => user.show_nsfw,
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
        let user = blocking(context.pool(), move |conn| {
          User_::read_from_name(conn, &username)
        })
        .await?;
        match user {
          Ok(user) => user.id,
          Err(_e) => return Err(APIError::err("couldnt_find_that_username_or_email").into()),
        }
      }
    };

    let user_view = blocking(context.pool(), move |conn| {
      UserView::get_user_secure(conn, user_details_id)
    })
    .await??;

    let page = data.page;
    let limit = data.limit;
    let saved_only = data.saved_only;
    let community_id = data.community_id;
    let user_id = user.map(|u| u.id);
    let (posts, comments) = blocking(context.pool(), move |conn| {
      let mut posts_query = PostQueryBuilder::create(conn)
        .sort(&sort)
        .show_nsfw(show_nsfw)
        .saved_only(saved_only)
        .for_community_id(community_id)
        .my_user_id(user_id)
        .page(page)
        .limit(limit);

      let mut comments_query = CommentQueryBuilder::create(conn)
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

      Ok((posts, comments)) as Result<_, LemmyError>
    })
    .await??;

    let follows = blocking(context.pool(), move |conn| {
      CommunityFollowerView::for_user(conn, user_details_id)
    })
    .await??;
    let moderates = blocking(context.pool(), move |conn| {
      CommunityModeratorView::for_user(conn, user_details_id)
    })
    .await??;

    // Return the jwt
    Ok(GetUserDetailsResponse {
      user: user_view,
      follows,
      moderates,
      comments,
      posts,
    })
  }
}

#[async_trait::async_trait(?Send)]
impl Perform for AddAdmin {
  type Response = AddAdminResponse;

  async fn perform(
    &self,
    context: &Data<LemmyContext>,
    websocket_id: Option<ConnectionId>,
  ) -> Result<AddAdminResponse, LemmyError> {
    let data: &AddAdmin = &self;
    let user = get_user_from_jwt(&data.auth, context.pool()).await?;

    // Make sure user is an admin
    is_admin(context.pool(), user.id).await?;

    let added = data.added;
    let added_user_id = data.user_id;
    let add_admin = move |conn: &'_ _| User_::add_admin(conn, added_user_id, added);
    if blocking(context.pool(), add_admin).await?.is_err() {
      return Err(APIError::err("couldnt_update_user").into());
    }

    // Mod tables
    let form = ModAddForm {
      mod_user_id: user.id,
      other_user_id: data.user_id,
      removed: Some(!data.added),
    };

    blocking(context.pool(), move |conn| ModAdd::create(conn, &form)).await??;

    let site_creator_id = blocking(context.pool(), move |conn| {
      Site::read(conn, 1).map(|s| s.creator_id)
    })
    .await??;

    let mut admins = blocking(context.pool(), move |conn| UserView::admins(conn)).await??;
    let creator_index = admins
      .iter()
      .position(|r| r.id == site_creator_id)
      .context(location_info!())?;
    let creator_user = admins.remove(creator_index);
    admins.insert(0, creator_user);

    let res = AddAdminResponse { admins };

    context.chat_server().do_send(SendAllMessage {
      op: UserOperation::AddAdmin,
      response: res.clone(),
      websocket_id,
    });

    Ok(res)
  }
}

#[async_trait::async_trait(?Send)]
impl Perform for BanUser {
  type Response = BanUserResponse;

  async fn perform(
    &self,
    context: &Data<LemmyContext>,
    websocket_id: Option<ConnectionId>,
  ) -> Result<BanUserResponse, LemmyError> {
    let data: &BanUser = &self;
    let user = get_user_from_jwt(&data.auth, context.pool()).await?;

    // Make sure user is an admin
    is_admin(context.pool(), user.id).await?;

    let ban = data.ban;
    let banned_user_id = data.user_id;
    let ban_user = move |conn: &'_ _| User_::ban_user(conn, banned_user_id, ban);
    if blocking(context.pool(), ban_user).await?.is_err() {
      return Err(APIError::err("couldnt_update_user").into());
    }

    // Remove their data if that's desired
    if let Some(remove_data) = data.remove_data {
      // Posts
      blocking(context.pool(), move |conn: &'_ _| {
        Post::update_removed_for_creator(conn, banned_user_id, None, remove_data)
      })
      .await??;

      // Communities
      blocking(context.pool(), move |conn: &'_ _| {
        Community::update_removed_for_creator(conn, banned_user_id, remove_data)
      })
      .await??;

      // Comments
      blocking(context.pool(), move |conn: &'_ _| {
        Comment::update_removed_for_creator(conn, banned_user_id, remove_data)
      })
      .await??;
    }

    // Mod tables
    let expires = match data.expires {
      Some(time) => Some(naive_from_unix(time)),
      None => None,
    };

    let form = ModBanForm {
      mod_user_id: user.id,
      other_user_id: data.user_id,
      reason: data.reason.to_owned(),
      banned: Some(data.ban),
      expires,
    };

    blocking(context.pool(), move |conn| ModBan::create(conn, &form)).await??;

    let user_id = data.user_id;
    let user_view = blocking(context.pool(), move |conn| {
      UserView::get_user_secure(conn, user_id)
    })
    .await??;

    let res = BanUserResponse {
      user: user_view,
      banned: data.ban,
    };

    context.chat_server().do_send(SendAllMessage {
      op: UserOperation::BanUser,
      response: res.clone(),
      websocket_id,
    });

    Ok(res)
  }
}

#[async_trait::async_trait(?Send)]
impl Perform for GetReplies {
  type Response = GetRepliesResponse;

  async fn perform(
    &self,
    context: &Data<LemmyContext>,
    _websocket_id: Option<ConnectionId>,
  ) -> Result<GetRepliesResponse, LemmyError> {
    let data: &GetReplies = &self;
    let user = get_user_from_jwt(&data.auth, context.pool()).await?;

    let sort = SortType::from_str(&data.sort)?;

    let page = data.page;
    let limit = data.limit;
    let unread_only = data.unread_only;
    let user_id = user.id;
    let replies = blocking(context.pool(), move |conn| {
      ReplyQueryBuilder::create(conn, user_id)
        .sort(&sort)
        .unread_only(unread_only)
        .page(page)
        .limit(limit)
        .list()
    })
    .await??;

    Ok(GetRepliesResponse { replies })
  }
}

#[async_trait::async_trait(?Send)]
impl Perform for GetUserMentions {
  type Response = GetUserMentionsResponse;

  async fn perform(
    &self,
    context: &Data<LemmyContext>,
    _websocket_id: Option<ConnectionId>,
  ) -> Result<GetUserMentionsResponse, LemmyError> {
    let data: &GetUserMentions = &self;
    let user = get_user_from_jwt(&data.auth, context.pool()).await?;

    let sort = SortType::from_str(&data.sort)?;

    let page = data.page;
    let limit = data.limit;
    let unread_only = data.unread_only;
    let user_id = user.id;
    let mentions = blocking(context.pool(), move |conn| {
      UserMentionQueryBuilder::create(conn, user_id)
        .sort(&sort)
        .unread_only(unread_only)
        .page(page)
        .limit(limit)
        .list()
    })
    .await??;

    Ok(GetUserMentionsResponse { mentions })
  }
}

#[async_trait::async_trait(?Send)]
impl Perform for MarkUserMentionAsRead {
  type Response = UserMentionResponse;

  async fn perform(
    &self,
    context: &Data<LemmyContext>,
    _websocket_id: Option<ConnectionId>,
  ) -> Result<UserMentionResponse, LemmyError> {
    let data: &MarkUserMentionAsRead = &self;
    let user = get_user_from_jwt(&data.auth, context.pool()).await?;

    let user_mention_id = data.user_mention_id;
    let read_user_mention = blocking(context.pool(), move |conn| {
      UserMention::read(conn, user_mention_id)
    })
    .await??;

    if user.id != read_user_mention.recipient_id {
      return Err(APIError::err("couldnt_update_comment").into());
    }

    let user_mention_id = read_user_mention.id;
    let read = data.read;
    let update_mention = move |conn: &'_ _| UserMention::update_read(conn, user_mention_id, read);
    if blocking(context.pool(), update_mention).await?.is_err() {
      return Err(APIError::err("couldnt_update_comment").into());
    };

    let user_mention_id = read_user_mention.id;
    let user_id = user.id;
    let user_mention_view = blocking(context.pool(), move |conn| {
      UserMentionView::read(conn, user_mention_id, user_id)
    })
    .await??;

    Ok(UserMentionResponse {
      mention: user_mention_view,
    })
  }
}

#[async_trait::async_trait(?Send)]
impl Perform for MarkAllAsRead {
  type Response = GetRepliesResponse;

  async fn perform(
    &self,
    context: &Data<LemmyContext>,
    _websocket_id: Option<ConnectionId>,
  ) -> Result<GetRepliesResponse, LemmyError> {
    let data: &MarkAllAsRead = &self;
    let user = get_user_from_jwt(&data.auth, context.pool()).await?;

    let user_id = user.id;
    let replies = blocking(context.pool(), move |conn| {
      ReplyQueryBuilder::create(conn, user_id)
        .unread_only(true)
        .page(1)
        .limit(999)
        .list()
    })
    .await??;

    // TODO: this should probably be a bulk operation
    // Not easy to do as a bulk operation,
    // because recipient_id isn't in the comment table
    for reply in &replies {
      let reply_id = reply.id;
      let mark_as_read = move |conn: &'_ _| Comment::update_read(conn, reply_id, true);
      if blocking(context.pool(), mark_as_read).await?.is_err() {
        return Err(APIError::err("couldnt_update_comment").into());
      }
    }

    // Mark all user mentions as read
    let update_user_mentions = move |conn: &'_ _| UserMention::mark_all_as_read(conn, user_id);
    if blocking(context.pool(), update_user_mentions)
      .await?
      .is_err()
    {
      return Err(APIError::err("couldnt_update_comment").into());
    }

    // Mark all private_messages as read
    let update_pm = move |conn: &'_ _| PrivateMessage::mark_all_as_read(conn, user_id);
    if blocking(context.pool(), update_pm).await?.is_err() {
      return Err(APIError::err("couldnt_update_private_message").into());
    }

    Ok(GetRepliesResponse { replies: vec![] })
  }
}

#[async_trait::async_trait(?Send)]
impl Perform for DeleteAccount {
  type Response = LoginResponse;

  async fn perform(
    &self,
    context: &Data<LemmyContext>,
    _websocket_id: Option<ConnectionId>,
  ) -> Result<LoginResponse, LemmyError> {
    let data: &DeleteAccount = &self;
    let user = get_user_from_jwt(&data.auth, context.pool()).await?;

    // Verify the password
    let valid: bool = verify(&data.password, &user.password_encrypted).unwrap_or(false);
    if !valid {
      return Err(APIError::err("password_incorrect").into());
    }

    // Comments
    let user_id = user.id;
    let permadelete = move |conn: &'_ _| Comment::permadelete_for_creator(conn, user_id);
    if blocking(context.pool(), permadelete).await?.is_err() {
      return Err(APIError::err("couldnt_update_comment").into());
    }

    // Posts
    let permadelete = move |conn: &'_ _| Post::permadelete_for_creator(conn, user_id);
    if blocking(context.pool(), permadelete).await?.is_err() {
      return Err(APIError::err("couldnt_update_post").into());
    }

    Ok(LoginResponse {
      jwt: data.auth.to_owned(),
    })
  }
}

#[async_trait::async_trait(?Send)]
impl Perform for PasswordReset {
  type Response = PasswordResetResponse;

  async fn perform(
    &self,
    context: &Data<LemmyContext>,
    _websocket_id: Option<ConnectionId>,
  ) -> Result<PasswordResetResponse, LemmyError> {
    let data: &PasswordReset = &self;

    // Fetch that email
    let email = data.email.clone();
    let user = match blocking(context.pool(), move |conn| {
      User_::find_by_email(conn, &email)
    })
    .await?
    {
      Ok(user) => user,
      Err(_e) => return Err(APIError::err("couldnt_find_that_username_or_email").into()),
    };

    // Generate a random token
    let token = generate_random_string();

    // Insert the row
    let token2 = token.clone();
    let user_id = user.id;
    blocking(context.pool(), move |conn| {
      PasswordResetRequest::create_token(conn, user_id, &token2)
    })
    .await??;

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
impl Perform for PasswordChange {
  type Response = LoginResponse;

  async fn perform(
    &self,
    context: &Data<LemmyContext>,
    _websocket_id: Option<ConnectionId>,
  ) -> Result<LoginResponse, LemmyError> {
    let data: &PasswordChange = &self;

    // Fetch the user_id from the token
    let token = data.token.clone();
    let user_id = blocking(context.pool(), move |conn| {
      PasswordResetRequest::read_from_token(conn, &token).map(|p| p.user_id)
    })
    .await??;

    // Make sure passwords match
    if data.password != data.password_verify {
      return Err(APIError::err("passwords_dont_match").into());
    }

    // Update the user with the new password
    let password = data.password.clone();
    let updated_user = match blocking(context.pool(), move |conn| {
      User_::update_password(conn, user_id, &password)
    })
    .await?
    {
      Ok(user) => user,
      Err(_e) => return Err(APIError::err("couldnt_update_user").into()),
    };

    // Return the jwt
    Ok(LoginResponse {
      jwt: Claims::jwt(updated_user, Settings::get().hostname)?,
    })
  }
}

#[async_trait::async_trait(?Send)]
impl Perform for CreatePrivateMessage {
  type Response = PrivateMessageResponse;

  async fn perform(
    &self,
    context: &Data<LemmyContext>,
    websocket_id: Option<ConnectionId>,
  ) -> Result<PrivateMessageResponse, LemmyError> {
    let data: &CreatePrivateMessage = &self;
    let user = get_user_from_jwt(&data.auth, context.pool()).await?;

    let hostname = &format!("https://{}", Settings::get().hostname);

    let content_slurs_removed = remove_slurs(&data.content.to_owned());

    let private_message_form = PrivateMessageForm {
      content: content_slurs_removed.to_owned(),
      creator_id: user.id,
      recipient_id: data.recipient_id,
      deleted: None,
      read: None,
      updated: None,
      ap_id: None,
      local: true,
      published: None,
    };

    let inserted_private_message = match blocking(context.pool(), move |conn| {
      PrivateMessage::create(conn, &private_message_form)
    })
    .await?
    {
      Ok(private_message) => private_message,
      Err(_e) => {
        return Err(APIError::err("couldnt_create_private_message").into());
      }
    };

    let inserted_private_message_id = inserted_private_message.id;
    let updated_private_message = match blocking(context.pool(), move |conn| {
      let apub_id = make_apub_endpoint(
        EndpointType::PrivateMessage,
        &inserted_private_message_id.to_string(),
      )
      .to_string();
      PrivateMessage::update_ap_id(&conn, inserted_private_message_id, apub_id)
    })
    .await?
    {
      Ok(private_message) => private_message,
      Err(_e) => return Err(APIError::err("couldnt_create_private_message").into()),
    };

    updated_private_message.send_create(&user, context).await?;

    // Send notifications to the recipient
    let recipient_id = data.recipient_id;
    let recipient_user =
      blocking(context.pool(), move |conn| User_::read(conn, recipient_id)).await??;
    if recipient_user.send_notifications_to_email {
      if let Some(email) = recipient_user.email {
        let subject = &format!(
          "{} - Private Message from {}",
          Settings::get().hostname,
          user.name,
        );
        let html = &format!(
          "<h1>Private Message</h1><br><div>{} - {}</div><br><a href={}/inbox>inbox</a>",
          user.name, &content_slurs_removed, hostname
        );
        match send_email(subject, &email, &recipient_user.name, html) {
          Ok(_o) => _o,
          Err(e) => error!("{}", e),
        };
      }
    }

    let message = blocking(context.pool(), move |conn| {
      PrivateMessageView::read(conn, inserted_private_message.id)
    })
    .await??;

    let res = PrivateMessageResponse { message };

    context.chat_server().do_send(SendUserRoomMessage {
      op: UserOperation::CreatePrivateMessage,
      response: res.clone(),
      recipient_id,
      websocket_id,
    });

    Ok(res)
  }
}

#[async_trait::async_trait(?Send)]
impl Perform for EditPrivateMessage {
  type Response = PrivateMessageResponse;

  async fn perform(
    &self,
    context: &Data<LemmyContext>,
    websocket_id: Option<ConnectionId>,
  ) -> Result<PrivateMessageResponse, LemmyError> {
    let data: &EditPrivateMessage = &self;
    let user = get_user_from_jwt(&data.auth, context.pool()).await?;

    // Checking permissions
    let edit_id = data.edit_id;
    let orig_private_message = blocking(context.pool(), move |conn| {
      PrivateMessage::read(conn, edit_id)
    })
    .await??;
    if user.id != orig_private_message.creator_id {
      return Err(APIError::err("no_private_message_edit_allowed").into());
    }

    // Doing the update
    let content_slurs_removed = remove_slurs(&data.content);
    let edit_id = data.edit_id;
    let updated_private_message = match blocking(context.pool(), move |conn| {
      PrivateMessage::update_content(conn, edit_id, &content_slurs_removed)
    })
    .await?
    {
      Ok(private_message) => private_message,
      Err(_e) => return Err(APIError::err("couldnt_update_private_message").into()),
    };

    // Send the apub update
    updated_private_message.send_update(&user, context).await?;

    let edit_id = data.edit_id;
    let message = blocking(context.pool(), move |conn| {
      PrivateMessageView::read(conn, edit_id)
    })
    .await??;
    let recipient_id = message.recipient_id;

    let res = PrivateMessageResponse { message };

    context.chat_server().do_send(SendUserRoomMessage {
      op: UserOperation::EditPrivateMessage,
      response: res.clone(),
      recipient_id,
      websocket_id,
    });

    Ok(res)
  }
}

#[async_trait::async_trait(?Send)]
impl Perform for DeletePrivateMessage {
  type Response = PrivateMessageResponse;

  async fn perform(
    &self,
    context: &Data<LemmyContext>,
    websocket_id: Option<ConnectionId>,
  ) -> Result<PrivateMessageResponse, LemmyError> {
    let data: &DeletePrivateMessage = &self;
    let user = get_user_from_jwt(&data.auth, context.pool()).await?;

    // Checking permissions
    let edit_id = data.edit_id;
    let orig_private_message = blocking(context.pool(), move |conn| {
      PrivateMessage::read(conn, edit_id)
    })
    .await??;
    if user.id != orig_private_message.creator_id {
      return Err(APIError::err("no_private_message_edit_allowed").into());
    }

    // Doing the update
    let edit_id = data.edit_id;
    let deleted = data.deleted;
    let updated_private_message = match blocking(context.pool(), move |conn| {
      PrivateMessage::update_deleted(conn, edit_id, deleted)
    })
    .await?
    {
      Ok(private_message) => private_message,
      Err(_e) => return Err(APIError::err("couldnt_update_private_message").into()),
    };

    // Send the apub update
    if data.deleted {
      updated_private_message.send_delete(&user, context).await?;
    } else {
      updated_private_message
        .send_undo_delete(&user, context)
        .await?;
    }

    let edit_id = data.edit_id;
    let message = blocking(context.pool(), move |conn| {
      PrivateMessageView::read(conn, edit_id)
    })
    .await??;
    let recipient_id = message.recipient_id;

    let res = PrivateMessageResponse { message };

    context.chat_server().do_send(SendUserRoomMessage {
      op: UserOperation::DeletePrivateMessage,
      response: res.clone(),
      recipient_id,
      websocket_id,
    });

    Ok(res)
  }
}

#[async_trait::async_trait(?Send)]
impl Perform for MarkPrivateMessageAsRead {
  type Response = PrivateMessageResponse;

  async fn perform(
    &self,
    context: &Data<LemmyContext>,
    websocket_id: Option<ConnectionId>,
  ) -> Result<PrivateMessageResponse, LemmyError> {
    let data: &MarkPrivateMessageAsRead = &self;
    let user = get_user_from_jwt(&data.auth, context.pool()).await?;

    // Checking permissions
    let edit_id = data.edit_id;
    let orig_private_message = blocking(context.pool(), move |conn| {
      PrivateMessage::read(conn, edit_id)
    })
    .await??;
    if user.id != orig_private_message.recipient_id {
      return Err(APIError::err("couldnt_update_private_message").into());
    }

    // Doing the update
    let edit_id = data.edit_id;
    let read = data.read;
    match blocking(context.pool(), move |conn| {
      PrivateMessage::update_read(conn, edit_id, read)
    })
    .await?
    {
      Ok(private_message) => private_message,
      Err(_e) => return Err(APIError::err("couldnt_update_private_message").into()),
    };

    // No need to send an apub update

    let edit_id = data.edit_id;
    let message = blocking(context.pool(), move |conn| {
      PrivateMessageView::read(conn, edit_id)
    })
    .await??;
    let recipient_id = message.recipient_id;

    let res = PrivateMessageResponse { message };

    context.chat_server().do_send(SendUserRoomMessage {
      op: UserOperation::MarkPrivateMessageAsRead,
      response: res.clone(),
      recipient_id,
      websocket_id,
    });

    Ok(res)
  }
}

#[async_trait::async_trait(?Send)]
impl Perform for GetPrivateMessages {
  type Response = PrivateMessagesResponse;

  async fn perform(
    &self,
    context: &Data<LemmyContext>,
    _websocket_id: Option<ConnectionId>,
  ) -> Result<PrivateMessagesResponse, LemmyError> {
    let data: &GetPrivateMessages = &self;
    let user = get_user_from_jwt(&data.auth, context.pool()).await?;
    let user_id = user.id;

    let page = data.page;
    let limit = data.limit;
    let unread_only = data.unread_only;
    let messages = blocking(context.pool(), move |conn| {
      PrivateMessageQueryBuilder::create(&conn, user_id)
        .page(page)
        .limit(limit)
        .unread_only(unread_only)
        .list()
    })
    .await??;

    Ok(PrivateMessagesResponse { messages })
  }
}

#[async_trait::async_trait(?Send)]
impl Perform for UserJoin {
  type Response = UserJoinResponse;

  async fn perform(
    &self,
    context: &Data<LemmyContext>,
    websocket_id: Option<ConnectionId>,
  ) -> Result<UserJoinResponse, LemmyError> {
    let data: &UserJoin = &self;
    let user = get_user_from_jwt(&data.auth, context.pool()).await?;

    if let Some(ws_id) = websocket_id {
      context.chat_server().do_send(JoinUserRoom {
        user_id: user.id,
        id: ws_id,
      });
    }

    Ok(UserJoinResponse { joined: true })
  }
}
