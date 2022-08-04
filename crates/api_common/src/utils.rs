use crate::{request::purge_image_from_pictrs, sensitive::Sensitive, site::FederatedInstances};
use chrono::NaiveDateTime;
use lemmy_db_schema::{
  impls::person::is_banned,
  newtypes::{CommunityId, LocalUserId, PersonId, PostId},
  source::{
    comment::Comment,
    community::Community,
    email_verification::{EmailVerification, EmailVerificationForm},
    password_reset_request::PasswordResetRequest,
    person::Person,
    person_block::PersonBlock,
    post::{Post, PostRead, PostReadForm},
    registration_application::RegistrationApplication,
    secret::Secret,
    site::Site,
  },
  traits::{Crud, Readable},
  utils::DbPool,
  ListingType,
};
use lemmy_db_views::{
  comment_view::CommentQuery,
  structs::{LocalUserSettingsView, LocalUserView},
};
use lemmy_db_views_actor::structs::{
  CommunityModeratorView,
  CommunityPersonBanView,
  CommunityView,
};
use lemmy_utils::{
  claims::Claims,
  email::{send_email, translations::Lang},
  error::LemmyError,
  settings::structs::Settings,
  utils::generate_random_string,
};
use reqwest_middleware::ClientWithMiddleware;
use rosetta_i18n::{Language, LanguageId};
use std::str::FromStr;
use tracing::warn;

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

#[tracing::instrument(skip_all)]
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

#[tracing::instrument(skip_all)]
pub async fn get_post(post_id: PostId, pool: &DbPool) -> Result<Post, LemmyError> {
  blocking(pool, move |conn| Post::read(conn, post_id))
    .await?
    .map_err(|e| LemmyError::from_error_message(e, "couldnt_find_post"))
}

#[tracing::instrument(skip_all)]
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
  .map_err(|e| LemmyError::from_error_message(e, "couldnt_mark_post_as_read"))
}

#[tracing::instrument(skip_all)]
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
  .map_err(|e| LemmyError::from_error_message(e, "couldnt_mark_post_as_read"))
}

#[tracing::instrument(skip_all)]
pub async fn get_local_user_view_from_jwt(
  jwt: &str,
  pool: &DbPool,
  secret: &Secret,
) -> Result<LocalUserView, LemmyError> {
  let claims = Claims::decode(jwt, &secret.jwt_secret)
    .map_err(|e| e.with_message("not_logged_in"))?
    .claims;
  let local_user_id = LocalUserId(claims.sub);
  let local_user_view =
    blocking(pool, move |conn| LocalUserView::read(conn, local_user_id)).await??;
  check_user_valid(
    local_user_view.person.banned,
    local_user_view.person.ban_expires,
    local_user_view.person.deleted,
  )?;

  check_validator_time(&local_user_view.local_user.validator_time, &claims)?;

  Ok(local_user_view)
}

/// Checks if user's token was issued before user's password reset.
pub fn check_validator_time(
  validator_time: &NaiveDateTime,
  claims: &Claims,
) -> Result<(), LemmyError> {
  let user_validation_time = validator_time.timestamp();
  if user_validation_time > claims.iat {
    Err(LemmyError::from_message("not_logged_in"))
  } else {
    Ok(())
  }
}

#[tracing::instrument(skip_all)]
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

#[tracing::instrument(skip_all)]
pub async fn get_local_user_settings_view_from_jwt_opt(
  jwt: Option<&Sensitive<String>>,
  pool: &DbPool,
  secret: &Secret,
) -> Result<Option<LocalUserSettingsView>, LemmyError> {
  match jwt {
    Some(jwt) => {
      let claims = Claims::decode(jwt.as_ref(), &secret.jwt_secret)
        .map_err(|e| e.with_message("not_logged_in"))?
        .claims;
      let local_user_id = LocalUserId(claims.sub);
      let local_user_view = blocking(pool, move |conn| {
        LocalUserSettingsView::read(conn, local_user_id)
      })
      .await??;
      check_user_valid(
        local_user_view.person.banned,
        local_user_view.person.ban_expires,
        local_user_view.person.deleted,
      )?;

      check_validator_time(&local_user_view.local_user.validator_time, &claims)?;

      Ok(Some(local_user_view))
    }
    None => Ok(None),
  }
}
pub fn check_user_valid(
  banned: bool,
  ban_expires: Option<NaiveDateTime>,
  deleted: bool,
) -> Result<(), LemmyError> {
  // Check for a site ban
  if is_banned(banned, ban_expires) {
    return Err(LemmyError::from_message("site_ban"));
  }

  // check for account deletion
  if deleted {
    return Err(LemmyError::from_message("deleted"));
  }

  Ok(())
}

#[tracing::instrument(skip_all)]
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

#[tracing::instrument(skip_all)]
pub async fn check_community_deleted_or_removed(
  community_id: CommunityId,
  pool: &DbPool,
) -> Result<(), LemmyError> {
  let community = blocking(pool, move |conn| Community::read(conn, community_id))
    .await?
    .map_err(|e| LemmyError::from_error_message(e, "couldnt_find_community"))?;
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

#[tracing::instrument(skip_all)]
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

#[tracing::instrument(skip_all)]
pub async fn check_downvotes_enabled(score: i16, pool: &DbPool) -> Result<(), LemmyError> {
  if score == -1 {
    let site = blocking(pool, Site::read_local_site).await??;
    if !site.enable_downvotes {
      return Err(LemmyError::from_message("downvotes_disabled"));
    }
  }
  Ok(())
}

#[tracing::instrument(skip_all)]
pub async fn check_private_instance(
  local_user_view: &Option<LocalUserView>,
  pool: &DbPool,
) -> Result<(), LemmyError> {
  if local_user_view.is_none() {
    let site = blocking(pool, Site::read_local_site).await?;

    // The site might not be set up yet
    if let Ok(site) = site {
      if site.private_instance {
        return Err(LemmyError::from_message("instance_is_private"));
      }
    }
  }
  Ok(())
}

#[tracing::instrument(skip_all)]
pub async fn build_federated_instances(
  pool: &DbPool,
  settings: &Settings,
) -> Result<Option<FederatedInstances>, LemmyError> {
  let federation_config = &settings.federation;
  let hostname = &settings.hostname;
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
      .map(|actor_id| Ok(actor_id.host_str().unwrap_or("").to_string()))
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
  subject: &str,
  body: &str,
  settings: &Settings,
) {
  if local_user_view.person.banned || !local_user_view.local_user.send_notifications_to_email {
    return;
  }

  if let Some(user_email) = &local_user_view.local_user.email {
    match send_email(
      subject,
      user_email,
      &local_user_view.person.name,
      body,
      settings,
    ) {
      Ok(_o) => _o,
      Err(e) => warn!("{}", e),
    };
  }
}

pub async fn send_password_reset_email(
  user: &LocalUserView,
  pool: &DbPool,
  settings: &Settings,
) -> Result<(), LemmyError> {
  // Generate a random token
  let token = generate_random_string();

  // Insert the row
  let token2 = token.clone();
  let local_user_id = user.local_user.id;
  blocking(pool, move |conn| {
    PasswordResetRequest::create_token(conn, local_user_id, &token2)
  })
  .await??;

  let email = &user.local_user.email.to_owned().expect("email");
  let lang = get_user_lang(user);
  let subject = &lang.password_reset_subject(&user.person.name);
  let protocol_and_hostname = settings.get_protocol_and_hostname();
  let reset_link = format!("{}/password_change/{}", protocol_and_hostname, &token);
  let body = &lang.password_reset_body(reset_link, &user.person.name);
  send_email(subject, email, &user.person.name, body, settings)
}

/// Send a verification email
pub async fn send_verification_email(
  user: &LocalUserView,
  new_email: &str,
  pool: &DbPool,
  settings: &Settings,
) -> Result<(), LemmyError> {
  let form = EmailVerificationForm {
    local_user_id: user.local_user.id,
    email: new_email.to_string(),
    verification_token: generate_random_string(),
  };
  let verify_link = format!(
    "{}/verify_email/{}",
    settings.get_protocol_and_hostname(),
    &form.verification_token
  );
  blocking(pool, move |conn| EmailVerification::create(conn, &form)).await??;

  let lang = get_user_lang(user);
  let subject = lang.verify_email_subject(&settings.hostname);
  let body = lang.verify_email_body(&settings.hostname, &user.person.name, verify_link);
  send_email(&subject, new_email, &user.person.name, &body, settings)?;

  Ok(())
}

pub fn send_email_verification_success(
  user: &LocalUserView,
  settings: &Settings,
) -> Result<(), LemmyError> {
  let email = &user.local_user.email.to_owned().expect("email");
  let lang = get_user_lang(user);
  let subject = &lang.email_verified_subject(&user.person.actor_id);
  let body = &lang.email_verified_body();
  send_email(subject, email, &user.person.name, body, settings)
}

pub fn get_user_lang(user: &LocalUserView) -> Lang {
  let user_lang = LanguageId::new(user.local_user.lang.clone());
  Lang::from_language_id(&user_lang).unwrap_or_else(|| {
    let en = LanguageId::new("en");
    Lang::from_language_id(&en).expect("default language")
  })
}

pub fn send_application_approved_email(
  user: &LocalUserView,
  settings: &Settings,
) -> Result<(), LemmyError> {
  let email = &user.local_user.email.to_owned().expect("email");
  let lang = get_user_lang(user);
  let subject = lang.registration_approved_subject(&user.person.actor_id);
  let body = lang.registration_approved_body(&settings.hostname);
  send_email(&subject, email, &user.person.name, &body, settings)
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
    if let Some(deny_reason) = registration.deny_reason {
      let lang = get_user_lang(local_user_view);
      let registration_denied_message = format!("{}: {}", lang.registration_denied(), &deny_reason);
      return Err(LemmyError::from_message(&registration_denied_message));
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
  let site_opt = blocking(pool, Site::read_local_site).await?;

  if let Ok(site) = site_opt {
    if site.private_instance && settings.federation.enabled {
      return Err(LemmyError::from_message(
        "Cannot have both private instance and federation enabled.",
      ));
    }
  }
  Ok(())
}

pub async fn purge_image_posts_for_person(
  banned_person_id: PersonId,
  pool: &DbPool,
  settings: &Settings,
  client: &ClientWithMiddleware,
) -> Result<(), LemmyError> {
  let posts = blocking(pool, move |conn: &'_ _| {
    Post::fetch_pictrs_posts_for_creator(conn, banned_person_id)
  })
  .await??;
  for post in posts {
    if let Some(url) = post.url {
      purge_image_from_pictrs(client, settings, &url).await.ok();
    }
    if let Some(thumbnail_url) = post.thumbnail_url {
      purge_image_from_pictrs(client, settings, &thumbnail_url)
        .await
        .ok();
    }
  }

  blocking(pool, move |conn| {
    Post::remove_pictrs_post_images_and_thumbnails_for_creator(conn, banned_person_id)
  })
  .await??;

  Ok(())
}

pub async fn purge_image_posts_for_community(
  banned_community_id: CommunityId,
  pool: &DbPool,
  settings: &Settings,
  client: &ClientWithMiddleware,
) -> Result<(), LemmyError> {
  let posts = blocking(pool, move |conn: &'_ _| {
    Post::fetch_pictrs_posts_for_community(conn, banned_community_id)
  })
  .await??;
  for post in posts {
    if let Some(url) = post.url {
      purge_image_from_pictrs(client, settings, &url).await.ok();
    }
    if let Some(thumbnail_url) = post.thumbnail_url {
      purge_image_from_pictrs(client, settings, &thumbnail_url)
        .await
        .ok();
    }
  }

  blocking(pool, move |conn| {
    Post::remove_pictrs_post_images_and_thumbnails_for_community(conn, banned_community_id)
  })
  .await??;

  Ok(())
}

pub async fn remove_user_data(
  banned_person_id: PersonId,
  pool: &DbPool,
  settings: &Settings,
  client: &ClientWithMiddleware,
) -> Result<(), LemmyError> {
  // Purge user images
  let person = blocking(pool, move |conn| Person::read(conn, banned_person_id)).await??;
  if let Some(avatar) = person.avatar {
    purge_image_from_pictrs(client, settings, &avatar)
      .await
      .ok();
  }
  if let Some(banner) = person.banner {
    purge_image_from_pictrs(client, settings, &banner)
      .await
      .ok();
  }

  // Update the fields to None
  blocking(pool, move |conn| {
    Person::remove_avatar_and_banner(conn, banned_person_id)
  })
  .await??;

  // Posts
  blocking(pool, move |conn: &'_ _| {
    Post::update_removed_for_creator(conn, banned_person_id, None, true)
  })
  .await??;

  // Purge image posts
  purge_image_posts_for_person(banned_person_id, pool, settings, client).await?;

  // Communities
  // Remove all communities where they're the top mod
  // for now, remove the communities manually
  let first_mod_communities = blocking(pool, move |conn: &'_ _| {
    CommunityModeratorView::get_community_first_mods(conn)
  })
  .await??;

  // Filter to only this banned users top communities
  let banned_user_first_communities: Vec<CommunityModeratorView> = first_mod_communities
    .into_iter()
    .filter(|fmc| fmc.moderator.id == banned_person_id)
    .collect();

  for first_mod_community in banned_user_first_communities {
    let community_id = first_mod_community.community.id;
    blocking(pool, move |conn: &'_ _| {
      Community::update_removed(conn, community_id, true)
    })
    .await??;

    // Delete the community images
    if let Some(icon) = first_mod_community.community.icon {
      purge_image_from_pictrs(client, settings, &icon).await.ok();
    }
    if let Some(banner) = first_mod_community.community.banner {
      purge_image_from_pictrs(client, settings, &banner)
        .await
        .ok();
    }
    // Update the fields to None
    blocking(pool, move |conn| {
      Community::remove_avatar_and_banner(conn, community_id)
    })
    .await??;
  }

  // Comments
  blocking(pool, move |conn: &'_ _| {
    Comment::update_removed_for_creator(conn, banned_person_id, true)
  })
  .await??;

  Ok(())
}

pub async fn remove_user_data_in_community(
  community_id: CommunityId,
  banned_person_id: PersonId,
  pool: &DbPool,
) -> Result<(), LemmyError> {
  // Posts
  blocking(pool, move |conn| {
    Post::update_removed_for_creator(conn, banned_person_id, Some(community_id), true)
  })
  .await??;

  // Comments
  // TODO Diesel doesn't allow updates with joins, so this has to be a loop
  let comments = blocking(pool, move |conn| {
    CommentQuery::builder()
      .conn(conn)
      .creator_id(Some(banned_person_id))
      .community_id(Some(community_id))
      .limit(Some(i64::MAX))
      .build()
      .list()
  })
  .await??;

  for comment_view in &comments {
    let comment_id = comment_view.comment.id;
    blocking(pool, move |conn| {
      Comment::update_removed(conn, comment_id, true)
    })
    .await??;
  }

  Ok(())
}

pub async fn delete_user_account(
  person_id: PersonId,
  pool: &DbPool,
  settings: &Settings,
  client: &ClientWithMiddleware,
) -> Result<(), LemmyError> {
  // Delete their images
  let person = blocking(pool, move |conn| Person::read(conn, person_id)).await??;
  if let Some(avatar) = person.avatar {
    purge_image_from_pictrs(client, settings, &avatar)
      .await
      .ok();
  }
  if let Some(banner) = person.banner {
    purge_image_from_pictrs(client, settings, &banner)
      .await
      .ok();
  }
  // No need to update avatar and banner, those are handled in Person::delete_account

  // Comments
  let permadelete = move |conn: &'_ _| Comment::permadelete_for_creator(conn, person_id);
  blocking(pool, permadelete)
    .await?
    .map_err(|e| LemmyError::from_error_message(e, "couldnt_update_comment"))?;

  // Posts
  let permadelete = move |conn: &'_ _| Post::permadelete_for_creator(conn, person_id);
  blocking(pool, permadelete)
    .await?
    .map_err(|e| LemmyError::from_error_message(e, "couldnt_update_post"))?;

  // Purge image posts
  purge_image_posts_for_person(person_id, pool, settings, client).await?;

  blocking(pool, move |conn| Person::delete_account(conn, person_id)).await??;

  Ok(())
}

pub async fn listing_type_with_site_default(
  listing_type: Option<ListingType>,
  pool: &DbPool,
) -> Result<ListingType, LemmyError> {
  Ok(match listing_type {
    Some(l) => l,
    None => {
      let site = blocking(pool, Site::read_local_site).await??;
      ListingType::from_str(&site.default_post_listing_type)?
    }
  })
}
