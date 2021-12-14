pub mod comment;
pub mod community;
pub mod person;
pub mod post;
pub mod site;
pub mod websocket;

use crate::site::FederatedInstances;
use lemmy_db_schema::{
  newtypes::{CommunityId, LocalUserId, PersonId, PostId},
  source::{
    community::Community,
    email_verification::{EmailVerification, EmailVerificationForm},
    password_reset_request::PasswordResetRequest,
    person_block::PersonBlock,
    post::{Post, PostRead, PostReadForm},
    registration_application::RegistrationApplication,
    secret::Secret,
    site::Site,
  },
  traits::{Crud, Readable},
  DbPool,
};
use lemmy_db_views::local_user_view::{LocalUserSettingsView, LocalUserView};
use lemmy_db_views_actor::{
  community_person_ban_view::CommunityPersonBanView,
  community_view::CommunityView,
};
use lemmy_utils::{
  claims::Claims,
  email::send_email,
  settings::structs::{FederationConfig, Settings},
  utils::generate_random_string,
  LemmyError,
  Sensitive,
};
use url::Url;

pub async fn blocking<F, T>(pool: &DbPool, f: F) -> Result<T, LemmyError>
where
  F: FnOnce(&diesel::PgConnection) -> T + Send + 'static,
  T: Send + 'static,
{
  let pool = pool.clone();
  let blocking_span = tracing::info_span!("blocking operation");
  let res = actix_web::web::block(move || {
    let entered = blocking_span.enter();
    let conn = pool.get()?;
    let res = (f)(&conn);
    drop(entered);
    Ok(res) as Result<T, LemmyError>
  })
  .await?;

  res
}

pub async fn is_mod_or_admin(
  pool: &DbPool,
  person_id: PersonId,
  community_id: CommunityId,
) -> Result<(), LemmyError> {
  let is_mod_or_admin = blocking(pool, move |conn| {
    CommunityView::is_mod_or_admin(conn, person_id, community_id)
  })
  .await?;
  if !is_mod_or_admin {
    return Err(LemmyError::from_message("not_a_mod_or_admin"));
  }
  Ok(())
}

pub fn is_admin(local_user_view: &LocalUserView) -> Result<(), LemmyError> {
  if !local_user_view.person.admin {
    return Err(LemmyError::from_message("not_an_admin"));
  }
  Ok(())
}

pub async fn get_post(post_id: PostId, pool: &DbPool) -> Result<Post, LemmyError> {
  blocking(pool, move |conn| Post::read(conn, post_id))
    .await?
    .map_err(LemmyError::from)
    .map_err(|e| e.with_message("couldnt_find_post"))
}

pub async fn mark_post_as_read(
  person_id: PersonId,
  post_id: PostId,
  pool: &DbPool,
) -> Result<PostRead, LemmyError> {
  let post_read_form = PostReadForm { post_id, person_id };

  blocking(pool, move |conn| {
    PostRead::mark_as_read(conn, &post_read_form)
  })
  .await?
  .map_err(LemmyError::from)
  .map_err(|e| e.with_message("couldnt_mark_post_as_read"))
}

pub async fn mark_post_as_unread(
  person_id: PersonId,
  post_id: PostId,
  pool: &DbPool,
) -> Result<usize, LemmyError> {
  let post_read_form = PostReadForm { post_id, person_id };

  blocking(pool, move |conn| {
    PostRead::mark_as_unread(conn, &post_read_form)
  })
  .await?
  .map_err(LemmyError::from)
  .map_err(|e| e.with_message("couldnt_mark_post_as_read"))
}

pub async fn get_local_user_view_from_jwt(
  jwt: &str,
  pool: &DbPool,
  secret: &Secret,
) -> Result<LocalUserView, LemmyError> {
  let claims = Claims::decode(jwt, &secret.jwt_secret)
    .map_err(LemmyError::from)
    .map_err(|e| e.with_message("not_logged_in"))?
    .claims;
  let local_user_id = LocalUserId(claims.sub);
  let local_user_view =
    blocking(pool, move |conn| LocalUserView::read(conn, local_user_id)).await??;
  // Check for a site ban
  if local_user_view.person.is_banned() {
    return Err(LemmyError::from_message("site_ban"));
  }

  // Check for user deletion
  if local_user_view.person.deleted {
    return Err(LemmyError::from_message("deleted"));
  }

  check_validator_time(&local_user_view.local_user.validator_time, &claims)?;

  Ok(local_user_view)
}

/// Checks if user's token was issued before user's password reset.
pub fn check_validator_time(
  validator_time: &chrono::NaiveDateTime,
  claims: &Claims,
) -> Result<(), LemmyError> {
  let user_validation_time = validator_time.timestamp();
  if user_validation_time > claims.iat {
    Err(LemmyError::from_message("not_logged_in"))
  } else {
    Ok(())
  }
}

pub async fn get_local_user_view_from_jwt_opt(
  jwt: Option<&Sensitive<String>>,
  pool: &DbPool,
  secret: &Secret,
) -> Result<Option<LocalUserView>, LemmyError> {
  match jwt {
    Some(jwt) => Ok(Some(get_local_user_view_from_jwt(jwt, pool, secret).await?)),
    None => Ok(None),
  }
}

pub async fn get_local_user_settings_view_from_jwt(
  jwt: &Sensitive<String>,
  pool: &DbPool,
  secret: &Secret,
) -> Result<LocalUserSettingsView, LemmyError> {
  let claims = Claims::decode(jwt.as_ref(), &secret.jwt_secret)
    .map_err(LemmyError::from)
    .map_err(|e| e.with_message("not_logged_in"))?
    .claims;
  let local_user_id = LocalUserId(claims.sub);
  let local_user_view = blocking(pool, move |conn| {
    LocalUserSettingsView::read(conn, local_user_id)
  })
  .await??;
  // Check for a site ban
  if local_user_view.person.is_banned() {
    return Err(LemmyError::from_message("site_ban"));
  }

  check_validator_time(&local_user_view.local_user.validator_time, &claims)?;

  Ok(local_user_view)
}

pub async fn get_local_user_settings_view_from_jwt_opt(
  jwt: Option<&Sensitive<String>>,
  pool: &DbPool,
  secret: &Secret,
) -> Result<Option<LocalUserSettingsView>, LemmyError> {
  match jwt {
    Some(jwt) => Ok(Some(
      get_local_user_settings_view_from_jwt(jwt, pool, secret).await?,
    )),
    None => Ok(None),
  }
}

pub async fn check_community_ban(
  person_id: PersonId,
  community_id: CommunityId,
  pool: &DbPool,
) -> Result<(), LemmyError> {
  let is_banned =
    move |conn: &'_ _| CommunityPersonBanView::get(conn, person_id, community_id).is_ok();
  if blocking(pool, is_banned).await? {
    Err(LemmyError::from_message("community_ban"))
  } else {
    Ok(())
  }
}

pub async fn check_community_deleted_or_removed(
  community_id: CommunityId,
  pool: &DbPool,
) -> Result<(), LemmyError> {
  let community = blocking(pool, move |conn| Community::read(conn, community_id))
    .await?
    .map_err(LemmyError::from)
    .map_err(|e| e.with_message("couldnt_find_community"))?;
  if community.deleted || community.removed {
    Err(LemmyError::from_message("deleted"))
  } else {
    Ok(())
  }
}

pub fn check_post_deleted_or_removed(post: &Post) -> Result<(), LemmyError> {
  if post.deleted || post.removed {
    Err(LemmyError::from_message("deleted"))
  } else {
    Ok(())
  }
}

pub async fn check_person_block(
  my_id: PersonId,
  potential_blocker_id: PersonId,
  pool: &DbPool,
) -> Result<(), LemmyError> {
  let is_blocked = move |conn: &'_ _| PersonBlock::read(conn, potential_blocker_id, my_id).is_ok();
  if blocking(pool, is_blocked).await? {
    Err(LemmyError::from_message("person_block"))
  } else {
    Ok(())
  }
}

pub async fn check_downvotes_enabled(score: i16, pool: &DbPool) -> Result<(), LemmyError> {
  if score == -1 {
    let site = blocking(pool, Site::read_simple).await??;
    if !site.enable_downvotes {
      return Err(LemmyError::from_message("downvotes_disabled"));
    }
  }
  Ok(())
}

pub async fn check_private_instance(
  local_user_view: &Option<LocalUserView>,
  pool: &DbPool,
) -> Result<(), LemmyError> {
  if local_user_view.is_none() {
    let site = blocking(pool, Site::read_simple).await??;
    if site.private_instance {
      return Err(LemmyError::from_message("instance_is_private"));
    }
  }
  Ok(())
}

pub async fn build_federated_instances(
  pool: &DbPool,
  federation_config: &FederationConfig,
  hostname: &str,
) -> Result<Option<FederatedInstances>, LemmyError> {
  let federation = federation_config.to_owned();
  if federation.enabled {
    let distinct_communities = blocking(pool, move |conn| {
      Community::distinct_federated_communities(conn)
    })
    .await??;

    let allowed = federation.allowed_instances;
    let blocked = federation.blocked_instances;

    let mut linked = distinct_communities
      .iter()
      .map(|actor_id| Ok(Url::parse(actor_id)?.host_str().unwrap_or("").to_string()))
      .collect::<Result<Vec<String>, LemmyError>>()?;

    if let Some(allowed) = allowed.as_ref() {
      linked.extend_from_slice(allowed);
    }

    if let Some(blocked) = blocked.as_ref() {
      linked.retain(|a| !blocked.contains(a) && !a.eq(hostname));
    }

    // Sort and remove dupes
    linked.sort_unstable();
    linked.dedup();

    Ok(Some(FederatedInstances {
      linked,
      allowed,
      blocked,
    }))
  } else {
    Ok(None)
  }
}

/// Checks the password length
pub fn password_length_check(pass: &str) -> Result<(), LemmyError> {
  if !(10..=60).contains(&pass.len()) {
    Err(LemmyError::from_message("invalid_password"))
  } else {
    Ok(())
  }
}

/// Checks the site description length
pub fn site_description_length_check(description: &str) -> Result<(), LemmyError> {
  if description.len() > 150 {
    Err(LemmyError::from_message("site_description_length_overflow"))
  } else {
    Ok(())
  }
}

/// Checks for a honeypot. If this field is filled, fail the rest of the function
pub fn honeypot_check(honeypot: &Option<String>) -> Result<(), LemmyError> {
  if honeypot.is_some() {
    Err(LemmyError::from_message("honeypot_fail"))
  } else {
    Ok(())
  }
}

pub fn send_email_to_user(
  local_user_view: &LocalUserView,
  subject_text: &str,
  body_text: &str,
  comment_content: &str,
  settings: &Settings,
) {
  if local_user_view.person.banned || !local_user_view.local_user.send_notifications_to_email {
    return;
  }

  if let Some(user_email) = &local_user_view.local_user.email {
    let subject = &format!(
      "{} - {} {}",
      subject_text, settings.hostname, local_user_view.person.name,
    );
    let html = &format!(
      "<h1>{}</h1><br><div>{} - {}</div><br><a href={}/inbox>inbox</a>",
      body_text,
      local_user_view.person.name,
      comment_content,
      settings.get_protocol_and_hostname()
    );
    match send_email(
      subject,
      user_email,
      &local_user_view.person.name,
      html,
      settings,
    ) {
      Ok(_o) => _o,
      Err(e) => tracing::error!("{}", e),
    };
  }
}

pub async fn send_password_reset_email(
  local_user_view: &LocalUserView,
  pool: &DbPool,
  settings: &Settings,
) -> Result<(), LemmyError> {
  // Generate a random token
  let token = generate_random_string();

  // Insert the row
  let token2 = token.clone();
  let local_user_id = local_user_view.local_user.id;
  blocking(pool, move |conn| {
    PasswordResetRequest::create_token(conn, local_user_id, &token2)
  })
  .await??;

  let email = &local_user_view.local_user.email.to_owned().expect("email");
  let subject = &format!("Password reset for {}", local_user_view.person.name);
  let protocol_and_hostname = settings.get_protocol_and_hostname();
  let html = &format!("<h1>Password Reset Request for {}</h1><br><a href={}/password_change/{}>Click here to reset your password</a>", local_user_view.person.name, protocol_and_hostname, &token);
  send_email(subject, email, &local_user_view.person.name, html, settings)
}

/// Send a verification email
pub async fn send_verification_email(
  local_user_id: LocalUserId,
  new_email: &str,
  username: &str,
  pool: &DbPool,
  settings: &Settings,
) -> Result<(), LemmyError> {
  let form = EmailVerificationForm {
    local_user_id,
    email: new_email.to_string(),
    verification_token: generate_random_string(),
  };
  let verify_link = format!(
    "{}/verify_email/{}",
    settings.get_protocol_and_hostname(),
    &form.verification_token
  );
  blocking(pool, move |conn| EmailVerification::create(conn, &form)).await??;

  let subject = format!("Verify your email address for {}", settings.hostname);
  let body = format!(
    concat!(
      "Please click the link below to verify your email address ",
      "for the account @{}@{}. Ignore this email if the account isn't yours.<br><br>",
      "<a href=\"{}\">Verify your email</a>"
    ),
    username, settings.hostname, verify_link
  );
  send_email(&subject, new_email, username, &body, settings)?;

  Ok(())
}

pub fn send_email_verification_success(
  local_user_view: &LocalUserView,
  settings: &Settings,
) -> Result<(), LemmyError> {
  let email = &local_user_view.local_user.email.to_owned().expect("email");
  let subject = &format!("Email verified for {}", local_user_view.person.actor_id);
  let html = "Your email has been verified.";
  send_email(subject, email, &local_user_view.person.name, html, settings)
}

pub fn send_application_approved_email(
  local_user_view: &LocalUserView,
  settings: &Settings,
) -> Result<(), LemmyError> {
  let email = &local_user_view.local_user.email.to_owned().expect("email");
  let subject = &format!(
    "Registration approved for {}",
    local_user_view.person.actor_id
  );
  let html = &format!(
    "Your registration application has been approved. Welcome to {}!",
    settings.hostname
  );
  send_email(subject, email, &local_user_view.person.name, html, settings)
}

pub async fn check_registration_application(
  site: &Site,
  local_user_view: &LocalUserView,
  pool: &DbPool,
) -> Result<(), LemmyError> {
  if site.require_application
    && !local_user_view.local_user.accepted_application
    && !local_user_view.person.admin
  {
    // Fetch the registration, see if its denied
    let local_user_id = local_user_view.local_user.id;
    let registration = blocking(pool, move |conn| {
      RegistrationApplication::find_by_local_user_id(conn, local_user_id)
    })
    .await??;
    if registration.deny_reason.is_some() {
      return Err(LemmyError::from_message("registration_denied"));
    } else {
      return Err(LemmyError::from_message("registration_application_pending"));
    }
  }
  Ok(())
}

/// TODO this check should be removed after https://github.com/LemmyNet/lemmy/issues/868 is done.
pub async fn check_private_instance_and_federation_enabled(
  pool: &DbPool,
  settings: &Settings,
) -> Result<(), LemmyError> {
  let site_opt = blocking(pool, Site::read_simple).await?;

  if let Ok(site) = site_opt {
    if site.private_instance && settings.federation.enabled {
      return Err(LemmyError::from_message(
        "Cannot have both private instance and federation enabled.",
      ));
    }
  }
  Ok(())
}
