use crate::{
  captcha_espeak_wav_base64,
  collect_moderated_communities,
  get_local_user_view_from_jwt,
  get_local_user_view_from_jwt_opt,
  is_admin,
  password_length_check,
  Perform,
};
use actix_web::web::Data;
use anyhow::Context;
use bcrypt::verify;
use captcha::{gen, Difficulty};
use chrono::Duration;
use lemmy_api_structs::{blocking, person::*, send_email_to_user};
use lemmy_apub::{
  generate_apub_endpoint,
  generate_followers_url,
  generate_inbox_url,
  generate_shared_inbox_url,
  ApubObjectType,
  EndpointType,
};
use lemmy_db_queries::{
  diesel_option_overwrite,
  diesel_option_overwrite_to_url,
  source::{
    comment::Comment_,
    community::Community_,
    local_user::LocalUser_,
    password_reset_request::PasswordResetRequest_,
    person::Person_,
    person_mention::PersonMention_,
    post::Post_,
    private_message::PrivateMessage_,
    site::Site_,
  },
  Crud,
  Followable,
  Joinable,
  ListingType,
  SortType,
};
use lemmy_db_schema::{
  naive_now,
  source::{
    comment::Comment,
    community::*,
    local_user::{LocalUser, LocalUserForm},
    moderator::*,
    password_reset_request::*,
    person::*,
    person_mention::*,
    post::Post,
    private_message::*,
    site::*,
  },
  CommunityId,
};
use lemmy_db_views::{
  comment_report_view::CommentReportView,
  comment_view::CommentQueryBuilder,
  local_user_view::LocalUserView,
  post_report_view::PostReportView,
  post_view::PostQueryBuilder,
  private_message_view::{PrivateMessageQueryBuilder, PrivateMessageView},
};
use lemmy_db_views_actor::{
  community_follower_view::CommunityFollowerView,
  community_moderator_view::CommunityModeratorView,
  person_mention_view::{PersonMentionQueryBuilder, PersonMentionView},
  person_view::PersonViewSafe,
};
use lemmy_utils::{
  apub::generate_actor_keypair,
  claims::Claims,
  email::send_email,
  location_info,
  settings::structs::Settings,
  utils::{
    check_slurs,
    generate_random_string,
    is_valid_preferred_username,
    is_valid_username,
    naive_from_unix,
    remove_slurs,
  },
  ApiError,
  ConnectionId,
  LemmyError,
};
use lemmy_websocket::{
  messages::{CaptchaItem, CheckCaptcha, SendAllMessage, SendUserRoomMessage},
  LemmyContext,
  UserOperation,
};
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
    let local_user_view = match blocking(context.pool(), move |conn| {
      LocalUserView::find_by_email_or_name(conn, &username_or_email)
    })
    .await?
    {
      Ok(uv) => uv,
      Err(_e) => return Err(ApiError::err("couldnt_find_that_username_or_email").into()),
    };

    // Verify the password
    let valid: bool = verify(
      &data.password,
      &local_user_view.local_user.password_encrypted,
    )
    .unwrap_or(false);
    if !valid {
      return Err(ApiError::err("password_incorrect").into());
    }

    // Return the jwt
    Ok(LoginResponse {
      jwt: Claims::jwt(local_user_view.local_user.id.0)?,
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
    if let Ok(site) = blocking(context.pool(), move |conn| Site::read_simple(conn)).await? {
      if !site.open_registration {
        return Err(ApiError::err("registration_closed").into());
      }
    }

    password_length_check(&data.password)?;

    // Make sure passwords match
    if data.password != data.password_verify {
      return Err(ApiError::err("passwords_dont_match").into());
    }

    // Check if there are admins. False if admins exist
    let no_admins = blocking(context.pool(), move |conn| {
      PersonViewSafe::admins(conn).map(|a| a.is_empty())
    })
    .await??;

    // If its not the admin, check the captcha
    if !no_admins && Settings::get().captcha().enabled {
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
        return Err(ApiError::err("captcha_incorrect").into());
      }
    }

    check_slurs(&data.username)?;

    let actor_keypair = generate_actor_keypair()?;
    if !is_valid_username(&data.username) {
      return Err(ApiError::err("invalid_username").into());
    }
    let actor_id = generate_apub_endpoint(EndpointType::Person, &data.username)?;

    // We have to create both a person, and local_user

    // Register the new person
    let person_form = PersonForm {
      name: data.username.to_owned(),
      actor_id: Some(actor_id.clone()),
      private_key: Some(Some(actor_keypair.private_key)),
      public_key: Some(Some(actor_keypair.public_key)),
      inbox_url: Some(generate_inbox_url(&actor_id)?),
      shared_inbox_url: Some(Some(generate_shared_inbox_url(&actor_id)?)),
      ..PersonForm::default()
    };

    // insert the person
    let inserted_person = match blocking(context.pool(), move |conn| {
      Person::create(conn, &person_form)
    })
    .await?
    {
      Ok(u) => u,
      Err(_) => {
        return Err(ApiError::err("user_already_exists").into());
      }
    };

    // Create the local user
    let local_user_form = LocalUserForm {
      person_id: inserted_person.id,
      email: Some(data.email.to_owned()),
      password_encrypted: data.password.to_owned(),
      admin: Some(no_admins),
      show_nsfw: Some(data.show_nsfw),
      theme: Some("browser".into()),
      default_sort_type: Some(SortType::Active as i16),
      default_listing_type: Some(ListingType::Subscribed as i16),
      lang: Some("browser".into()),
      show_avatars: Some(true),
      send_notifications_to_email: Some(false),
    };

    let inserted_local_user = match blocking(context.pool(), move |conn| {
      LocalUser::register(conn, &local_user_form)
    })
    .await?
    {
      Ok(lu) => lu,
      Err(e) => {
        let err_type = if e.to_string()
          == "duplicate key value violates unique constraint \"local_user_email_key\""
        {
          "email_already_exists"
        } else {
          "user_already_exists"
        };

        // If the local user creation errored, then delete that person
        blocking(context.pool(), move |conn| {
          Person::delete(&conn, inserted_person.id)
        })
        .await??;

        return Err(ApiError::err(err_type).into());
      }
    };

    let main_community_keypair = generate_actor_keypair()?;

    // Create the main community if it doesn't exist
    let main_community = match blocking(context.pool(), move |conn| {
      Community::read(conn, CommunityId(2))
    })
    .await?
    {
      Ok(c) => c,
      Err(_e) => {
        let default_community_name = "main";
        let actor_id = generate_apub_endpoint(EndpointType::Community, default_community_name)?;
        let community_form = CommunityForm {
          name: default_community_name.to_string(),
          title: "The Default Community".to_string(),
          description: Some("The Default Community".to_string()),
          creator_id: inserted_person.id,
          actor_id: Some(actor_id.to_owned()),
          private_key: Some(main_community_keypair.private_key),
          public_key: Some(main_community_keypair.public_key),
          followers_url: Some(generate_followers_url(&actor_id)?),
          inbox_url: Some(generate_inbox_url(&actor_id)?),
          shared_inbox_url: Some(Some(generate_shared_inbox_url(&actor_id)?)),
          ..CommunityForm::default()
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
      person_id: inserted_person.id,
      pending: false,
    };

    let follow = move |conn: &'_ _| CommunityFollower::follow(conn, &community_follower_form);
    if blocking(context.pool(), follow).await?.is_err() {
      return Err(ApiError::err("community_follower_already_exists").into());
    };

    // If its an admin, add them as a mod and follower to main
    if no_admins {
      let community_moderator_form = CommunityModeratorForm {
        community_id: main_community.id,
        person_id: inserted_person.id,
      };

      let join = move |conn: &'_ _| CommunityModerator::join(conn, &community_moderator_form);
      if blocking(context.pool(), join).await?.is_err() {
        return Err(ApiError::err("community_moderator_already_exists").into());
      }
    }

    // Return the jwt
    Ok(LoginResponse {
      jwt: Claims::jwt(inserted_local_user.id.0)?,
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
    let captcha_settings = Settings::get().captcha();

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
    let local_user_view = get_local_user_view_from_jwt(&data.auth, context.pool()).await?;

    let avatar = diesel_option_overwrite_to_url(&data.avatar)?;
    let banner = diesel_option_overwrite_to_url(&data.banner)?;
    let email = diesel_option_overwrite(&data.email);
    let bio = diesel_option_overwrite(&data.bio);
    let preferred_username = diesel_option_overwrite(&data.preferred_username);
    let matrix_user_id = diesel_option_overwrite(&data.matrix_user_id);

    if let Some(Some(bio)) = &bio {
      if bio.chars().count() > 300 {
        return Err(ApiError::err("bio_length_overflow").into());
      }
    }

    if let Some(Some(preferred_username)) = &preferred_username {
      if !is_valid_preferred_username(preferred_username.trim()) {
        return Err(ApiError::err("invalid_username").into());
      }
    }

    let local_user_id = local_user_view.local_user.id;
    let person_id = local_user_view.person.id;
    let password_encrypted = match &data.new_password {
      Some(new_password) => {
        match &data.new_password_verify {
          Some(new_password_verify) => {
            password_length_check(&new_password)?;

            // Make sure passwords match
            if new_password != new_password_verify {
              return Err(ApiError::err("passwords_dont_match").into());
            }

            // Check the old password
            match &data.old_password {
              Some(old_password) => {
                let valid: bool =
                  verify(old_password, &local_user_view.local_user.password_encrypted)
                    .unwrap_or(false);
                if !valid {
                  return Err(ApiError::err("password_incorrect").into());
                }
                let new_password = new_password.to_owned();
                let user = blocking(context.pool(), move |conn| {
                  LocalUser::update_password(conn, local_user_id, &new_password)
                })
                .await??;
                user.password_encrypted
              }
              None => return Err(ApiError::err("password_incorrect").into()),
            }
          }
          None => return Err(ApiError::err("passwords_dont_match").into()),
        }
      }
      None => local_user_view.local_user.password_encrypted,
    };

    let default_listing_type = data.default_listing_type;
    let default_sort_type = data.default_sort_type;

    let person_form = PersonForm {
      name: local_user_view.person.name,
      avatar,
      banner,
      inbox_url: None,
      preferred_username,
      published: None,
      updated: Some(naive_now()),
      banned: None,
      deleted: None,
      actor_id: None,
      bio,
      local: None,
      private_key: None,
      public_key: None,
      last_refreshed_at: None,
      shared_inbox_url: None,
      matrix_user_id,
    };

    let person_res = blocking(context.pool(), move |conn| {
      Person::update(conn, person_id, &person_form)
    })
    .await?;
    let _updated_person: Person = match person_res {
      Ok(p) => p,
      Err(_) => {
        return Err(ApiError::err("user_already_exists").into());
      }
    };

    let local_user_form = LocalUserForm {
      person_id,
      email,
      password_encrypted,
      admin: None,
      show_nsfw: data.show_nsfw,
      theme: data.theme.to_owned(),
      default_sort_type,
      default_listing_type,
      lang: data.lang.to_owned(),
      show_avatars: data.show_avatars,
      send_notifications_to_email: data.send_notifications_to_email,
    };

    let local_user_res = blocking(context.pool(), move |conn| {
      LocalUser::update(conn, local_user_id, &local_user_form)
    })
    .await?;
    let updated_local_user = match local_user_res {
      Ok(u) => u,
      Err(e) => {
        let err_type = if e.to_string()
          == "duplicate key value violates unique constraint \"local_user_email_key\""
        {
          "email_already_exists"
        } else {
          "user_already_exists"
        };

        return Err(ApiError::err(err_type).into());
      }
    };

    // Return the jwt
    Ok(LoginResponse {
      jwt: Claims::jwt(updated_local_user.id.0)?,
    })
  }
}

#[async_trait::async_trait(?Send)]
impl Perform for GetPersonDetails {
  type Response = GetPersonDetailsResponse;

  async fn perform(
    &self,
    context: &Data<LemmyContext>,
    _websocket_id: Option<ConnectionId>,
  ) -> Result<GetPersonDetailsResponse, LemmyError> {
    let data: &GetPersonDetails = &self;
    let local_user_view = get_local_user_view_from_jwt_opt(&data.auth, context.pool()).await?;

    let show_nsfw = match &local_user_view {
      Some(uv) => uv.local_user.show_nsfw,
      None => false,
    };

    let sort = SortType::from_str(&data.sort)?;

    let username = data
      .username
      .to_owned()
      .unwrap_or_else(|| "admin".to_string());
    let person_details_id = match data.person_id {
      Some(id) => id,
      None => {
        let person = blocking(context.pool(), move |conn| {
          Person::find_by_name(conn, &username)
        })
        .await?;
        match person {
          Ok(p) => p.id,
          Err(_e) => return Err(ApiError::err("couldnt_find_that_username_or_email").into()),
        }
      }
    };

    let person_id = local_user_view.map(|uv| uv.person.id);

    // You don't need to return settings for the user, since this comes back with GetSite
    // `my_user`
    let person_view = blocking(context.pool(), move |conn| {
      PersonViewSafe::read(conn, person_details_id)
    })
    .await??;

    let page = data.page;
    let limit = data.limit;
    let saved_only = data.saved_only;
    let community_id = data.community_id;

    let (posts, comments) = blocking(context.pool(), move |conn| {
      let mut posts_query = PostQueryBuilder::create(conn)
        .sort(&sort)
        .show_nsfw(show_nsfw)
        .saved_only(saved_only)
        .community_id(community_id)
        .my_person_id(person_id)
        .page(page)
        .limit(limit);

      let mut comments_query = CommentQueryBuilder::create(conn)
        .my_person_id(person_id)
        .sort(&sort)
        .saved_only(saved_only)
        .page(page)
        .limit(limit);

      // If its saved only, you don't care what creator it was
      // Or, if its not saved, then you only want it for that specific creator
      if !saved_only {
        posts_query = posts_query.creator_id(person_details_id);
        comments_query = comments_query.creator_id(person_details_id);
      }

      let posts = posts_query.list()?;
      let comments = comments_query.list()?;

      Ok((posts, comments)) as Result<_, LemmyError>
    })
    .await??;

    let mut follows = vec![];
    if let Some(pid) = person_id {
      if pid == person_details_id {
        follows = blocking(context.pool(), move |conn| {
          CommunityFollowerView::for_person(conn, person_details_id)
        })
        .await??;
      }
    };
    let moderates = blocking(context.pool(), move |conn| {
      CommunityModeratorView::for_person(conn, person_details_id)
    })
    .await??;

    // Return the jwt
    Ok(GetPersonDetailsResponse {
      person_view,
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
    let local_user_view = get_local_user_view_from_jwt(&data.auth, context.pool()).await?;

    // Make sure user is an admin
    is_admin(&local_user_view)?;

    let added = data.added;
    let added_person_id = data.person_id;
    let added_admin = match blocking(context.pool(), move |conn| {
      LocalUser::add_admin(conn, added_person_id, added)
    })
    .await?
    {
      Ok(a) => a,
      Err(_) => {
        return Err(ApiError::err("couldnt_update_user").into());
      }
    };

    // Mod tables
    let form = ModAddForm {
      mod_person_id: local_user_view.person.id,
      other_person_id: added_admin.person_id,
      removed: Some(!data.added),
    };

    blocking(context.pool(), move |conn| ModAdd::create(conn, &form)).await??;

    let site_creator_id = blocking(context.pool(), move |conn| {
      Site::read(conn, 1).map(|s| s.creator_id)
    })
    .await??;

    let mut admins = blocking(context.pool(), move |conn| PersonViewSafe::admins(conn)).await??;
    let creator_index = admins
      .iter()
      .position(|r| r.person.id == site_creator_id)
      .context(location_info!())?;
    let creator_person = admins.remove(creator_index);
    admins.insert(0, creator_person);

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
impl Perform for BanPerson {
  type Response = BanPersonResponse;

  async fn perform(
    &self,
    context: &Data<LemmyContext>,
    websocket_id: Option<ConnectionId>,
  ) -> Result<BanPersonResponse, LemmyError> {
    let data: &BanPerson = &self;
    let local_user_view = get_local_user_view_from_jwt(&data.auth, context.pool()).await?;

    // Make sure user is an admin
    is_admin(&local_user_view)?;

    let ban = data.ban;
    let banned_person_id = data.person_id;
    let ban_person = move |conn: &'_ _| Person::ban_person(conn, banned_person_id, ban);
    if blocking(context.pool(), ban_person).await?.is_err() {
      return Err(ApiError::err("couldnt_update_user").into());
    }

    // Remove their data if that's desired
    if data.remove_data {
      // Posts
      blocking(context.pool(), move |conn: &'_ _| {
        Post::update_removed_for_creator(conn, banned_person_id, None, true)
      })
      .await??;

      // Communities
      blocking(context.pool(), move |conn: &'_ _| {
        Community::update_removed_for_creator(conn, banned_person_id, true)
      })
      .await??;

      // Comments
      blocking(context.pool(), move |conn: &'_ _| {
        Comment::update_removed_for_creator(conn, banned_person_id, true)
      })
      .await??;
    }

    // Mod tables
    let expires = match data.expires {
      Some(time) => Some(naive_from_unix(time)),
      None => None,
    };

    let form = ModBanForm {
      mod_person_id: local_user_view.person.id,
      other_person_id: data.person_id,
      reason: data.reason.to_owned(),
      banned: Some(data.ban),
      expires,
    };

    blocking(context.pool(), move |conn| ModBan::create(conn, &form)).await??;

    let person_id = data.person_id;
    let person_view = blocking(context.pool(), move |conn| {
      PersonViewSafe::read(conn, person_id)
    })
    .await??;

    let res = BanPersonResponse {
      person_view,
      banned: data.ban,
    };

    context.chat_server().do_send(SendAllMessage {
      op: UserOperation::BanPerson,
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
    let local_user_view = get_local_user_view_from_jwt(&data.auth, context.pool()).await?;

    let sort = SortType::from_str(&data.sort)?;

    let page = data.page;
    let limit = data.limit;
    let unread_only = data.unread_only;
    let person_id = local_user_view.person.id;
    let replies = blocking(context.pool(), move |conn| {
      CommentQueryBuilder::create(conn)
        .sort(&sort)
        .unread_only(unread_only)
        .recipient_id(person_id)
        .my_person_id(person_id)
        .page(page)
        .limit(limit)
        .list()
    })
    .await??;

    Ok(GetRepliesResponse { replies })
  }
}

#[async_trait::async_trait(?Send)]
impl Perform for GetPersonMentions {
  type Response = GetPersonMentionsResponse;

  async fn perform(
    &self,
    context: &Data<LemmyContext>,
    _websocket_id: Option<ConnectionId>,
  ) -> Result<GetPersonMentionsResponse, LemmyError> {
    let data: &GetPersonMentions = &self;
    let local_user_view = get_local_user_view_from_jwt(&data.auth, context.pool()).await?;

    let sort = SortType::from_str(&data.sort)?;

    let page = data.page;
    let limit = data.limit;
    let unread_only = data.unread_only;
    let person_id = local_user_view.person.id;
    let mentions = blocking(context.pool(), move |conn| {
      PersonMentionQueryBuilder::create(conn)
        .recipient_id(person_id)
        .my_person_id(person_id)
        .sort(&sort)
        .unread_only(unread_only)
        .page(page)
        .limit(limit)
        .list()
    })
    .await??;

    Ok(GetPersonMentionsResponse { mentions })
  }
}

#[async_trait::async_trait(?Send)]
impl Perform for MarkPersonMentionAsRead {
  type Response = PersonMentionResponse;

  async fn perform(
    &self,
    context: &Data<LemmyContext>,
    _websocket_id: Option<ConnectionId>,
  ) -> Result<PersonMentionResponse, LemmyError> {
    let data: &MarkPersonMentionAsRead = &self;
    let local_user_view = get_local_user_view_from_jwt(&data.auth, context.pool()).await?;

    let person_mention_id = data.person_mention_id;
    let read_person_mention = blocking(context.pool(), move |conn| {
      PersonMention::read(conn, person_mention_id)
    })
    .await??;

    if local_user_view.person.id != read_person_mention.recipient_id {
      return Err(ApiError::err("couldnt_update_comment").into());
    }

    let person_mention_id = read_person_mention.id;
    let read = data.read;
    let update_mention =
      move |conn: &'_ _| PersonMention::update_read(conn, person_mention_id, read);
    if blocking(context.pool(), update_mention).await?.is_err() {
      return Err(ApiError::err("couldnt_update_comment").into());
    };

    let person_mention_id = read_person_mention.id;
    let person_id = local_user_view.person.id;
    let person_mention_view = blocking(context.pool(), move |conn| {
      PersonMentionView::read(conn, person_mention_id, Some(person_id))
    })
    .await??;

    Ok(PersonMentionResponse {
      person_mention_view,
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
    let local_user_view = get_local_user_view_from_jwt(&data.auth, context.pool()).await?;

    let person_id = local_user_view.person.id;
    let replies = blocking(context.pool(), move |conn| {
      CommentQueryBuilder::create(conn)
        .my_person_id(person_id)
        .recipient_id(person_id)
        .unread_only(true)
        .page(1)
        .limit(999)
        .list()
    })
    .await??;

    // TODO: this should probably be a bulk operation
    // Not easy to do as a bulk operation,
    // because recipient_id isn't in the comment table
    for comment_view in &replies {
      let reply_id = comment_view.comment.id;
      let mark_as_read = move |conn: &'_ _| Comment::update_read(conn, reply_id, true);
      if blocking(context.pool(), mark_as_read).await?.is_err() {
        return Err(ApiError::err("couldnt_update_comment").into());
      }
    }

    // Mark all user mentions as read
    let update_person_mentions =
      move |conn: &'_ _| PersonMention::mark_all_as_read(conn, person_id);
    if blocking(context.pool(), update_person_mentions)
      .await?
      .is_err()
    {
      return Err(ApiError::err("couldnt_update_comment").into());
    }

    // Mark all private_messages as read
    let update_pm = move |conn: &'_ _| PrivateMessage::mark_all_as_read(conn, person_id);
    if blocking(context.pool(), update_pm).await?.is_err() {
      return Err(ApiError::err("couldnt_update_private_message").into());
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
    let local_user_view = get_local_user_view_from_jwt(&data.auth, context.pool()).await?;

    // Verify the password
    let valid: bool = verify(
      &data.password,
      &local_user_view.local_user.password_encrypted,
    )
    .unwrap_or(false);
    if !valid {
      return Err(ApiError::err("password_incorrect").into());
    }

    // Comments
    let person_id = local_user_view.person.id;
    let permadelete = move |conn: &'_ _| Comment::permadelete_for_creator(conn, person_id);
    if blocking(context.pool(), permadelete).await?.is_err() {
      return Err(ApiError::err("couldnt_update_comment").into());
    }

    // Posts
    let permadelete = move |conn: &'_ _| Post::permadelete_for_creator(conn, person_id);
    if blocking(context.pool(), permadelete).await?.is_err() {
      return Err(ApiError::err("couldnt_update_post").into());
    }

    blocking(context.pool(), move |conn| {
      Person::delete_account(conn, person_id)
    })
    .await??;

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
    let local_user_view = match blocking(context.pool(), move |conn| {
      LocalUserView::find_by_email(conn, &email)
    })
    .await?
    {
      Ok(lu) => lu,
      Err(_e) => return Err(ApiError::err("couldnt_find_that_username_or_email").into()),
    };

    // Generate a random token
    let token = generate_random_string();

    // Insert the row
    let token2 = token.clone();
    let local_user_id = local_user_view.local_user.id;
    blocking(context.pool(), move |conn| {
      PasswordResetRequest::create_token(conn, local_user_id, &token2)
    })
    .await??;

    // Email the pure token to the user.
    // TODO no i18n support here.
    let email = &local_user_view.local_user.email.expect("email");
    let subject = &format!("Password reset for {}", local_user_view.person.name);
    let hostname = &Settings::get().get_protocol_and_hostname();
    let html = &format!("<h1>Password Reset Request for {}</h1><br><a href={}/password_change/{}>Click here to reset your password</a>", local_user_view.person.name, hostname, &token);
    match send_email(subject, email, &local_user_view.person.name, html) {
      Ok(_o) => _o,
      Err(_e) => return Err(ApiError::err(&_e).into()),
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
    let local_user_id = blocking(context.pool(), move |conn| {
      PasswordResetRequest::read_from_token(conn, &token).map(|p| p.local_user_id)
    })
    .await??;

    password_length_check(&data.password)?;

    // Make sure passwords match
    if data.password != data.password_verify {
      return Err(ApiError::err("passwords_dont_match").into());
    }

    // Update the user with the new password
    let password = data.password.clone();
    let updated_local_user = match blocking(context.pool(), move |conn| {
      LocalUser::update_password(conn, local_user_id, &password)
    })
    .await?
    {
      Ok(u) => u,
      Err(_e) => return Err(ApiError::err("couldnt_update_user").into()),
    };

    // Return the jwt
    Ok(LoginResponse {
      jwt: Claims::jwt(updated_local_user.id.0)?,
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
    let local_user_view = get_local_user_view_from_jwt(&data.auth, context.pool()).await?;

    let content_slurs_removed = remove_slurs(&data.content.to_owned());

    let private_message_form = PrivateMessageForm {
      content: content_slurs_removed.to_owned(),
      creator_id: local_user_view.person.id,
      recipient_id: data.recipient_id,
      ..PrivateMessageForm::default()
    };

    let inserted_private_message = match blocking(context.pool(), move |conn| {
      PrivateMessage::create(conn, &private_message_form)
    })
    .await?
    {
      Ok(private_message) => private_message,
      Err(_e) => {
        return Err(ApiError::err("couldnt_create_private_message").into());
      }
    };

    let inserted_private_message_id = inserted_private_message.id;
    let updated_private_message = match blocking(
      context.pool(),
      move |conn| -> Result<PrivateMessage, LemmyError> {
        let apub_id = generate_apub_endpoint(
          EndpointType::PrivateMessage,
          &inserted_private_message_id.to_string(),
        )?;
        Ok(PrivateMessage::update_ap_id(
          &conn,
          inserted_private_message_id,
          apub_id,
        )?)
      },
    )
    .await?
    {
      Ok(private_message) => private_message,
      Err(_e) => return Err(ApiError::err("couldnt_create_private_message").into()),
    };

    updated_private_message
      .send_create(&local_user_view.person, context)
      .await?;

    let private_message_view = blocking(context.pool(), move |conn| {
      PrivateMessageView::read(conn, inserted_private_message.id)
    })
    .await??;

    let res = PrivateMessageResponse {
      private_message_view,
    };

    // Send notifications to the local recipient, if one exists
    let recipient_id = data.recipient_id;
    if let Ok(local_recipient) = blocking(context.pool(), move |conn| {
      LocalUserView::read_person(conn, recipient_id)
    })
    .await?
    {
      send_email_to_user(
        &local_recipient,
        "Private Message from",
        "Private Message",
        &content_slurs_removed,
      );

      let local_recipient_id = local_recipient.local_user.id;
      context.chat_server().do_send(SendUserRoomMessage {
        op: UserOperation::CreatePrivateMessage,
        response: res.clone(),
        local_recipient_id,
        websocket_id,
      });
    }

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
    let local_user_view = get_local_user_view_from_jwt(&data.auth, context.pool()).await?;

    // Checking permissions
    let private_message_id = data.private_message_id;
    let orig_private_message = blocking(context.pool(), move |conn| {
      PrivateMessage::read(conn, private_message_id)
    })
    .await??;
    if local_user_view.person.id != orig_private_message.creator_id {
      return Err(ApiError::err("no_private_message_edit_allowed").into());
    }

    // Doing the update
    let content_slurs_removed = remove_slurs(&data.content);
    let private_message_id = data.private_message_id;
    let updated_private_message = match blocking(context.pool(), move |conn| {
      PrivateMessage::update_content(conn, private_message_id, &content_slurs_removed)
    })
    .await?
    {
      Ok(private_message) => private_message,
      Err(_e) => return Err(ApiError::err("couldnt_update_private_message").into()),
    };

    // Send the apub update
    updated_private_message
      .send_update(&local_user_view.person, context)
      .await?;

    let private_message_id = data.private_message_id;
    let private_message_view = blocking(context.pool(), move |conn| {
      PrivateMessageView::read(conn, private_message_id)
    })
    .await??;

    let res = PrivateMessageResponse {
      private_message_view,
    };

    // Send notifications to the local recipient, if one exists
    let recipient_id = orig_private_message.recipient_id;
    if let Ok(local_recipient) = blocking(context.pool(), move |conn| {
      LocalUserView::read_person(conn, recipient_id)
    })
    .await?
    {
      let local_recipient_id = local_recipient.local_user.id;
      context.chat_server().do_send(SendUserRoomMessage {
        op: UserOperation::EditPrivateMessage,
        response: res.clone(),
        local_recipient_id,
        websocket_id,
      });
    }

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
    let local_user_view = get_local_user_view_from_jwt(&data.auth, context.pool()).await?;

    // Checking permissions
    let private_message_id = data.private_message_id;
    let orig_private_message = blocking(context.pool(), move |conn| {
      PrivateMessage::read(conn, private_message_id)
    })
    .await??;
    if local_user_view.person.id != orig_private_message.creator_id {
      return Err(ApiError::err("no_private_message_edit_allowed").into());
    }

    // Doing the update
    let private_message_id = data.private_message_id;
    let deleted = data.deleted;
    let updated_private_message = match blocking(context.pool(), move |conn| {
      PrivateMessage::update_deleted(conn, private_message_id, deleted)
    })
    .await?
    {
      Ok(private_message) => private_message,
      Err(_e) => return Err(ApiError::err("couldnt_update_private_message").into()),
    };

    // Send the apub update
    if data.deleted {
      updated_private_message
        .send_delete(&local_user_view.person, context)
        .await?;
    } else {
      updated_private_message
        .send_undo_delete(&local_user_view.person, context)
        .await?;
    }

    let private_message_id = data.private_message_id;
    let private_message_view = blocking(context.pool(), move |conn| {
      PrivateMessageView::read(conn, private_message_id)
    })
    .await??;

    let res = PrivateMessageResponse {
      private_message_view,
    };

    // Send notifications to the local recipient, if one exists
    let recipient_id = orig_private_message.recipient_id;
    if let Ok(local_recipient) = blocking(context.pool(), move |conn| {
      LocalUserView::read_person(conn, recipient_id)
    })
    .await?
    {
      let local_recipient_id = local_recipient.local_user.id;
      context.chat_server().do_send(SendUserRoomMessage {
        op: UserOperation::DeletePrivateMessage,
        response: res.clone(),
        local_recipient_id,
        websocket_id,
      });
    }

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
    let local_user_view = get_local_user_view_from_jwt(&data.auth, context.pool()).await?;

    // Checking permissions
    let private_message_id = data.private_message_id;
    let orig_private_message = blocking(context.pool(), move |conn| {
      PrivateMessage::read(conn, private_message_id)
    })
    .await??;
    if local_user_view.person.id != orig_private_message.recipient_id {
      return Err(ApiError::err("couldnt_update_private_message").into());
    }

    // Doing the update
    let private_message_id = data.private_message_id;
    let read = data.read;
    match blocking(context.pool(), move |conn| {
      PrivateMessage::update_read(conn, private_message_id, read)
    })
    .await?
    {
      Ok(private_message) => private_message,
      Err(_e) => return Err(ApiError::err("couldnt_update_private_message").into()),
    };

    // No need to send an apub update
    let private_message_id = data.private_message_id;
    let private_message_view = blocking(context.pool(), move |conn| {
      PrivateMessageView::read(conn, private_message_id)
    })
    .await??;

    let res = PrivateMessageResponse {
      private_message_view,
    };

    // Send notifications to the local recipient, if one exists
    let recipient_id = orig_private_message.recipient_id;
    if let Ok(local_recipient) = blocking(context.pool(), move |conn| {
      LocalUserView::read_person(conn, recipient_id)
    })
    .await?
    {
      let local_recipient_id = local_recipient.local_user.id;
      context.chat_server().do_send(SendUserRoomMessage {
        op: UserOperation::MarkPrivateMessageAsRead,
        response: res.clone(),
        local_recipient_id,
        websocket_id,
      });
    }

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
    let local_user_view = get_local_user_view_from_jwt(&data.auth, context.pool()).await?;
    let person_id = local_user_view.person.id;

    let page = data.page;
    let limit = data.limit;
    let unread_only = data.unread_only;
    let messages = blocking(context.pool(), move |conn| {
      PrivateMessageQueryBuilder::create(&conn, person_id)
        .page(page)
        .limit(limit)
        .unread_only(unread_only)
        .list()
    })
    .await??;

    Ok(PrivateMessagesResponse {
      private_messages: messages,
    })
  }
}

#[async_trait::async_trait(?Send)]
impl Perform for GetReportCount {
  type Response = GetReportCountResponse;

  async fn perform(
    &self,
    context: &Data<LemmyContext>,
    websocket_id: Option<ConnectionId>,
  ) -> Result<GetReportCountResponse, LemmyError> {
    let data: &GetReportCount = &self;
    let local_user_view = get_local_user_view_from_jwt(&data.auth, context.pool()).await?;

    let person_id = local_user_view.person.id;
    let community_id = data.community;
    let community_ids =
      collect_moderated_communities(person_id, community_id, context.pool()).await?;

    let res = {
      if community_ids.is_empty() {
        GetReportCountResponse {
          community: None,
          comment_reports: 0,
          post_reports: 0,
        }
      } else {
        let ids = community_ids.clone();
        let comment_reports = blocking(context.pool(), move |conn| {
          CommentReportView::get_report_count(conn, &ids)
        })
        .await??;

        let ids = community_ids.clone();
        let post_reports = blocking(context.pool(), move |conn| {
          PostReportView::get_report_count(conn, &ids)
        })
        .await??;

        GetReportCountResponse {
          community: data.community,
          comment_reports,
          post_reports,
        }
      }
    };

    context.chat_server().do_send(SendUserRoomMessage {
      op: UserOperation::GetReportCount,
      response: res.clone(),
      local_recipient_id: local_user_view.local_user.id,
      websocket_id,
    });

    Ok(res)
  }
}
