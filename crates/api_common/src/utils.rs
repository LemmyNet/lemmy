use crate::{request::purge_image_from_pictrs, sensitive::Sensitive, site::FederatedInstances};
use chrono::NaiveDateTime;
use lemmy_db_schema::{
  impls::person::is_banned,
  newtypes::{CommunityId, LocalUserId, PersonId, PostId},
  source::{
    comment::{Comment, CommentUpdateForm},
    community::{Community, CommunityUpdateForm},
    email_verification::{EmailVerification, EmailVerificationForm},
    instance::Instance,
    local_site::LocalSite,
    local_site_rate_limit::LocalSiteRateLimit,
    password_reset_request::PasswordResetRequest,
    person::{Person, PersonUpdateForm},
    person_block::PersonBlock,
    post::{Post, PostRead, PostReadForm},
    registration_application::RegistrationApplication,
    secret::Secret,
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
  rate_limit::RateLimitConfig,
  settings::structs::Settings,
  utils::{build_slur_regex, generate_random_string},
};
use regex::Regex;
use reqwest_middleware::ClientWithMiddleware;
use rosetta_i18n::{Language, LanguageId};
use std::str::FromStr;
use tracing::warn;

#[tracing::instrument(skip_all)]
pub async fn is_mod_or_admin(
  pool: &DbPool,
  person_id: PersonId,
  community_id: CommunityId,
) -> Result<(), LemmyError> {
  let is_mod_or_admin = CommunityView::is_mod_or_admin(pool, person_id, community_id).await?;
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
  Post::read(pool, post_id)
    .await
    .map_err(|e| LemmyError::from_error_message(e, "couldnt_find_post"))
}

#[tracing::instrument(skip_all)]
pub async fn mark_post_as_read(
  person_id: PersonId,
  post_id: PostId,
  pool: &DbPool,
) -> Result<PostRead, LemmyError> {
  let post_read_form = PostReadForm { post_id, person_id };

  PostRead::mark_as_read(pool, &post_read_form)
    .await
    .map_err(|e| LemmyError::from_error_message(e, "couldnt_mark_post_as_read"))
}

#[tracing::instrument(skip_all)]
pub async fn mark_post_as_unread(
  person_id: PersonId,
  post_id: PostId,
  pool: &DbPool,
) -> Result<usize, LemmyError> {
  let post_read_form = PostReadForm { post_id, person_id };

  PostRead::mark_as_unread(pool, &post_read_form)
    .await
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
  let local_user_view = LocalUserView::read(pool, local_user_id).await?;
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
      let local_user_view = LocalUserSettingsView::read(pool, local_user_id).await?;
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
  let is_banned = CommunityPersonBanView::get(pool, person_id, community_id)
    .await
    .is_ok();
  if is_banned {
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
  let community = Community::read(pool, community_id)
    .await
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
  let is_blocked = PersonBlock::read(pool, potential_blocker_id, my_id)
    .await
    .is_ok();
  if is_blocked {
    Err(LemmyError::from_message("person_block"))
  } else {
    Ok(())
  }
}

#[tracing::instrument(skip_all)]
pub fn check_downvotes_enabled(score: i16, local_site: &LocalSite) -> Result<(), LemmyError> {
  if score == -1 && !local_site.enable_downvotes {
    return Err(LemmyError::from_message("downvotes_disabled"));
  }
  Ok(())
}

#[tracing::instrument(skip_all)]
pub fn check_private_instance(
  local_user_view: &Option<LocalUserView>,
  local_site: &LocalSite,
) -> Result<(), LemmyError> {
  if local_user_view.is_none() && local_site.private_instance {
    return Err(LemmyError::from_message("instance_is_private"));
  }
  Ok(())
}

#[tracing::instrument(skip_all)]
pub async fn build_federated_instances(
  local_site: &LocalSite,
  pool: &DbPool,
) -> Result<Option<FederatedInstances>, LemmyError> {
  if local_site.federation_enabled {
    // TODO I hate that this requires 3 queries
    let linked = Instance::linked(pool).await?;
    let allowed = Instance::allowlist(pool).await?;
    let blocked = Instance::blocklist(pool).await?;

    // These can return empty vectors, so convert them to options
    let allowed = (!allowed.is_empty()).then_some(allowed);
    let blocked = (!blocked.is_empty()).then_some(blocked);

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
  if !(10..=60).contains(&pass.chars().count()) {
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
  PasswordResetRequest::create_token(pool, local_user_id, &token2).await?;

  let email = &user.local_user.email.clone().expect("email");
  let lang = get_interface_language(user);
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
  EmailVerification::create(pool, &form).await?;

  let lang = get_interface_language(user);
  let subject = lang.verify_email_subject(&settings.hostname);
  let body = lang.verify_email_body(&settings.hostname, &user.person.name, verify_link);
  send_email(&subject, new_email, &user.person.name, &body, settings)?;

  Ok(())
}

pub fn send_email_verification_success(
  user: &LocalUserView,
  settings: &Settings,
) -> Result<(), LemmyError> {
  let email = &user.local_user.email.clone().expect("email");
  let lang = get_interface_language(user);
  let subject = &lang.email_verified_subject(&user.person.actor_id);
  let body = &lang.email_verified_body();
  send_email(subject, email, &user.person.name, body, settings)
}

pub fn get_interface_language(user: &LocalUserView) -> Lang {
  lang_str_to_lang(&user.local_user.interface_language)
}

pub fn get_interface_language_from_settings(user: &LocalUserSettingsView) -> Lang {
  lang_str_to_lang(&user.local_user.interface_language)
}

fn lang_str_to_lang(lang: &str) -> Lang {
  let lang_id = LanguageId::new(lang);
  Lang::from_language_id(&lang_id).unwrap_or_else(|| {
    let en = LanguageId::new("en");
    Lang::from_language_id(&en).expect("default language")
  })
}

pub fn local_site_rate_limit_to_rate_limit_config(
  local_site_rate_limit: &LocalSiteRateLimit,
) -> RateLimitConfig {
  let l = local_site_rate_limit;
  RateLimitConfig {
    message: l.message,
    message_per_second: l.message_per_second,
    post: l.post,
    post_per_second: l.post_per_second,
    register: l.register,
    register_per_second: l.register_per_second,
    image: l.image,
    image_per_second: l.image_per_second,
    comment: l.comment,
    comment_per_second: l.comment_per_second,
    search: l.search,
    search_per_second: l.search_per_second,
  }
}

pub fn local_site_to_slur_regex(local_site: &LocalSite) -> Option<Regex> {
  build_slur_regex(local_site.slur_filter_regex.as_deref())
}

pub fn local_site_opt_to_slur_regex(local_site: &Option<LocalSite>) -> Option<Regex> {
  local_site
    .as_ref()
    .map(local_site_to_slur_regex)
    .unwrap_or(None)
}

pub fn send_application_approved_email(
  user: &LocalUserView,
  settings: &Settings,
) -> Result<(), LemmyError> {
  let email = &user.local_user.email.clone().expect("email");
  let lang = get_interface_language(user);
  let subject = lang.registration_approved_subject(&user.person.actor_id);
  let body = lang.registration_approved_body(&settings.hostname);
  send_email(&subject, email, &user.person.name, &body, settings)
}

/// Send a new applicant email notification to all admins
pub async fn send_new_applicant_email_to_admins(
  applicant_username: &str,
  pool: &DbPool,
  settings: &Settings,
) -> Result<(), LemmyError> {
  // Collect the admins with emails
  let admins = LocalUserSettingsView::list_admins_with_emails(pool).await?;

  let applications_link = &format!(
    "{}/registration_applications",
    settings.get_protocol_and_hostname(),
  );

  for admin in &admins {
    let email = &admin.local_user.email.clone().expect("email");
    let lang = get_interface_language_from_settings(admin);
    let subject = lang.new_application_subject(applicant_username, &settings.hostname);
    let body = lang.new_application_body(applications_link);
    send_email(&subject, email, &admin.person.name, &body, settings)?;
  }
  Ok(())
}

pub async fn check_registration_application(
  local_user_view: &LocalUserView,
  local_site: &LocalSite,
  pool: &DbPool,
) -> Result<(), LemmyError> {
  if local_site.require_application
    && !local_user_view.local_user.accepted_application
    && !local_user_view.person.admin
  {
    // Fetch the registration, see if its denied
    let local_user_id = local_user_view.local_user.id;
    let registration = RegistrationApplication::find_by_local_user_id(pool, local_user_id).await?;
    if let Some(deny_reason) = registration.deny_reason {
      let lang = get_interface_language(local_user_view);
      let registration_denied_message = format!("{}: {}", lang.registration_denied(), &deny_reason);
      return Err(LemmyError::from_message(&registration_denied_message));
    } else {
      return Err(LemmyError::from_message("registration_application_pending"));
    }
  }
  Ok(())
}

pub fn check_private_instance_and_federation_enabled(
  local_site: &LocalSite,
) -> Result<(), LemmyError> {
  if local_site.private_instance && local_site.federation_enabled {
    return Err(LemmyError::from_message(
      "Cannot have both private instance and federation enabled.",
    ));
  }
  Ok(())
}

pub async fn purge_image_posts_for_person(
  banned_person_id: PersonId,
  pool: &DbPool,
  settings: &Settings,
  client: &ClientWithMiddleware,
) -> Result<(), LemmyError> {
  let posts = Post::fetch_pictrs_posts_for_creator(pool, banned_person_id).await?;
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

  Post::remove_pictrs_post_images_and_thumbnails_for_creator(pool, banned_person_id).await?;

  Ok(())
}

pub async fn purge_image_posts_for_community(
  banned_community_id: CommunityId,
  pool: &DbPool,
  settings: &Settings,
  client: &ClientWithMiddleware,
) -> Result<(), LemmyError> {
  let posts = Post::fetch_pictrs_posts_for_community(pool, banned_community_id).await?;
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

  Post::remove_pictrs_post_images_and_thumbnails_for_community(pool, banned_community_id).await?;

  Ok(())
}

pub async fn remove_user_data(
  banned_person_id: PersonId,
  pool: &DbPool,
  settings: &Settings,
  client: &ClientWithMiddleware,
) -> Result<(), LemmyError> {
  // Purge user images
  let person = Person::read(pool, banned_person_id).await?;
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
  Person::update(
    pool,
    banned_person_id,
    &PersonUpdateForm::builder()
      .avatar(Some(None))
      .banner(Some(None))
      .build(),
  )
  .await?;

  // Posts
  Post::update_removed_for_creator(pool, banned_person_id, None, true).await?;

  // Purge image posts
  purge_image_posts_for_person(banned_person_id, pool, settings, client).await?;

  // Communities
  // Remove all communities where they're the top mod
  // for now, remove the communities manually
  let first_mod_communities = CommunityModeratorView::get_community_first_mods(pool).await?;

  // Filter to only this banned users top communities
  let banned_user_first_communities: Vec<CommunityModeratorView> = first_mod_communities
    .into_iter()
    .filter(|fmc| fmc.moderator.id == banned_person_id)
    .collect();

  for first_mod_community in banned_user_first_communities {
    let community_id = first_mod_community.community.id;
    Community::update(
      pool,
      community_id,
      &CommunityUpdateForm::builder().removed(Some(true)).build(),
    )
    .await?;

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
    Community::update(
      pool,
      community_id,
      &CommunityUpdateForm::builder()
        .icon(Some(None))
        .banner(Some(None))
        .build(),
    )
    .await?;
  }

  // Comments
  Comment::update_removed_for_creator(pool, banned_person_id, true).await?;

  Ok(())
}

pub async fn remove_user_data_in_community(
  community_id: CommunityId,
  banned_person_id: PersonId,
  pool: &DbPool,
) -> Result<(), LemmyError> {
  // Posts
  Post::update_removed_for_creator(pool, banned_person_id, Some(community_id), true).await?;

  // Comments
  // TODO Diesel doesn't allow updates with joins, so this has to be a loop
  let comments = CommentQuery::builder()
    .pool(pool)
    .creator_id(Some(banned_person_id))
    .community_id(Some(community_id))
    .limit(Some(i64::MAX))
    .build()
    .list()
    .await?;

  for comment_view in &comments {
    let comment_id = comment_view.comment.id;
    Comment::update(
      pool,
      comment_id,
      &CommentUpdateForm::builder().removed(Some(true)).build(),
    )
    .await?;
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
  let person = Person::read(pool, person_id).await?;
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
  Comment::permadelete_for_creator(pool, person_id)
    .await
    .map_err(|e| LemmyError::from_error_message(e, "couldnt_update_comment"))?;

  // Posts
  Post::permadelete_for_creator(pool, person_id)
    .await
    .map_err(|e| LemmyError::from_error_message(e, "couldnt_update_post"))?;

  // Purge image posts
  purge_image_posts_for_person(person_id, pool, settings, client).await?;

  Person::delete_account(pool, person_id).await?;

  Ok(())
}

pub fn listing_type_with_site_default(
  listing_type: Option<ListingType>,
  local_site: &LocalSite,
) -> Result<ListingType, LemmyError> {
  Ok(listing_type.unwrap_or(ListingType::from_str(
    &local_site.default_post_listing_type,
  )?))
}

#[cfg(test)]
mod tests {
  use crate::utils::password_length_check;

  #[test]
  #[rustfmt::skip]
  fn password_length() {
    assert!(password_length_check("Õ¼¾°3yË,o¸ãtÌÈú|ÇÁÙAøüÒI©·¤(T]/ð>æºWæ[C¤bªWöaÃÎñ·{=û³&§½K/c").is_ok());
    assert!(password_length_check("1234567890").is_ok());
    assert!(password_length_check("short").is_err());
    assert!(password_length_check("looooooooooooooooooooooooooooooooooooooooooooooooooooooooooong").is_err());
  }
}
