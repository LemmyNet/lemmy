use crate::{
  context::LemmyContext,
  request::purge_image_from_pictrs,
  sensitive::Sensitive,
  site::FederatedInstances,
};
use actix_web::cookie::{Cookie, SameSite};
use anyhow::Context;
use chrono::{DateTime, Utc};
use lemmy_db_schema::{
  impls::person::is_banned,
  newtypes::{CommunityId, DbUrl, PersonId, PostId},
  source::{
    comment::{Comment, CommentUpdateForm},
    community::{Community, CommunityModerator, CommunityUpdateForm},
    email_verification::{EmailVerification, EmailVerificationForm},
    instance::Instance,
    local_site::LocalSite,
    local_site_rate_limit::LocalSiteRateLimit,
    password_reset_request::PasswordResetRequest,
    person::{Person, PersonUpdateForm},
    person_block::PersonBlock,
    post::{Post, PostRead, PostReadForm},
  },
  traits::{Crud, Readable},
  utils::DbPool,
};
use lemmy_db_views::{comment_view::CommentQuery, structs::LocalUserView};
use lemmy_db_views_actor::structs::{
  CommunityModeratorView,
  CommunityPersonBanView,
  CommunityView,
};
use lemmy_utils::{
  email::{send_email, translations::Lang},
  error::{LemmyError, LemmyErrorExt, LemmyErrorType},
  location_info,
  rate_limit::RateLimitConfig,
  settings::structs::Settings,
  utils::slurs::build_slur_regex,
};
use regex::Regex;
use rosetta_i18n::{Language, LanguageId};
use tracing::warn;
use url::{ParseError, Url};

pub static AUTH_COOKIE_NAME: &str = "auth";

#[tracing::instrument(skip_all)]
pub async fn is_mod_or_admin(
  pool: &mut DbPool<'_>,
  person_id: PersonId,
  community_id: CommunityId,
) -> Result<(), LemmyError> {
  let is_mod_or_admin = CommunityView::is_mod_or_admin(pool, person_id, community_id).await?;
  if !is_mod_or_admin {
    Err(LemmyErrorType::NotAModOrAdmin)?
  } else {
    Ok(())
  }
}

#[tracing::instrument(skip_all)]
pub async fn is_mod_or_admin_opt(
  pool: &mut DbPool<'_>,
  local_user_view: Option<&LocalUserView>,
  community_id: Option<CommunityId>,
) -> Result<(), LemmyError> {
  if let Some(local_user_view) = local_user_view {
    if let Some(community_id) = community_id {
      is_mod_or_admin(pool, local_user_view.person.id, community_id).await
    } else {
      is_admin(local_user_view)
    }
  } else {
    Err(LemmyErrorType::NotAModOrAdmin)?
  }
}

pub fn is_admin(local_user_view: &LocalUserView) -> Result<(), LemmyError> {
  if !local_user_view.local_user.admin {
    Err(LemmyErrorType::NotAnAdmin)?
  } else {
    Ok(())
  }
}

pub fn is_top_mod(
  local_user_view: &LocalUserView,
  community_mods: &[CommunityModeratorView],
) -> Result<(), LemmyError> {
  if local_user_view.person.id
    != community_mods
      .first()
      .map(|cm| cm.moderator.id)
      .unwrap_or(PersonId(0))
  {
    Err(LemmyErrorType::NotTopMod)?
  } else {
    Ok(())
  }
}

#[tracing::instrument(skip_all)]
pub async fn get_post(post_id: PostId, pool: &mut DbPool<'_>) -> Result<Post, LemmyError> {
  Post::read(pool, post_id)
    .await
    .with_lemmy_type(LemmyErrorType::CouldntFindPost)
}

#[tracing::instrument(skip_all)]
pub async fn mark_post_as_read(
  person_id: PersonId,
  post_id: PostId,
  pool: &mut DbPool<'_>,
) -> Result<PostRead, LemmyError> {
  let post_read_form = PostReadForm { post_id, person_id };

  PostRead::mark_as_read(pool, &post_read_form)
    .await
    .with_lemmy_type(LemmyErrorType::CouldntMarkPostAsRead)
}

#[tracing::instrument(skip_all)]
pub async fn mark_post_as_unread(
  person_id: PersonId,
  post_id: PostId,
  pool: &mut DbPool<'_>,
) -> Result<usize, LemmyError> {
  let post_read_form = PostReadForm { post_id, person_id };

  PostRead::mark_as_unread(pool, &post_read_form)
    .await
    .with_lemmy_type(LemmyErrorType::CouldntMarkPostAsRead)
}

pub fn check_user_valid(
  banned: bool,
  ban_expires: Option<DateTime<Utc>>,
  deleted: bool,
) -> Result<(), LemmyError> {
  // Check for a site ban
  if is_banned(banned, ban_expires) {
    Err(LemmyErrorType::SiteBan)?
  }
  // check for account deletion
  else if deleted {
    Err(LemmyErrorType::Deleted)?
  } else {
    Ok(())
  }
}

#[tracing::instrument(skip_all)]
pub async fn check_community_ban(
  person_id: PersonId,
  community_id: CommunityId,
  pool: &mut DbPool<'_>,
) -> Result<(), LemmyError> {
  let is_banned = CommunityPersonBanView::get(pool, person_id, community_id)
    .await
    .is_ok();
  if is_banned {
    Err(LemmyErrorType::BannedFromCommunity)?
  } else {
    Ok(())
  }
}

#[tracing::instrument(skip_all)]
pub async fn check_community_deleted_or_removed(
  community_id: CommunityId,
  pool: &mut DbPool<'_>,
) -> Result<(), LemmyError> {
  let community = Community::read(pool, community_id)
    .await
    .with_lemmy_type(LemmyErrorType::CouldntFindCommunity)?;
  if community.deleted || community.removed {
    Err(LemmyErrorType::Deleted)?
  } else {
    Ok(())
  }
}

pub fn check_post_deleted_or_removed(post: &Post) -> Result<(), LemmyError> {
  if post.deleted || post.removed {
    Err(LemmyErrorType::Deleted)?
  } else {
    Ok(())
  }
}

#[tracing::instrument(skip_all)]
pub async fn check_person_block(
  my_id: PersonId,
  potential_blocker_id: PersonId,
  pool: &mut DbPool<'_>,
) -> Result<(), LemmyError> {
  let is_blocked = PersonBlock::read(pool, potential_blocker_id, my_id)
    .await
    .is_ok();
  if is_blocked {
    Err(LemmyErrorType::PersonIsBlocked)?
  } else {
    Ok(())
  }
}

#[tracing::instrument(skip_all)]
pub fn check_downvotes_enabled(score: i16, local_site: &LocalSite) -> Result<(), LemmyError> {
  if score == -1 && !local_site.enable_downvotes {
    Err(LemmyErrorType::DownvotesAreDisabled)?
  } else {
    Ok(())
  }
}

#[tracing::instrument(skip_all)]
pub fn check_private_instance(
  local_user_view: &Option<LocalUserView>,
  local_site: &LocalSite,
) -> Result<(), LemmyError> {
  if local_user_view.is_none() && local_site.private_instance {
    Err(LemmyErrorType::InstanceIsPrivate)?
  } else {
    Ok(())
  }
}

#[tracing::instrument(skip_all)]
pub async fn build_federated_instances(
  local_site: &LocalSite,
  pool: &mut DbPool<'_>,
) -> Result<Option<FederatedInstances>, LemmyError> {
  if local_site.federation_enabled {
    // TODO I hate that this requires 3 queries
    let (linked, allowed, blocked) = lemmy_db_schema::try_join_with_pool!(pool => (
      Instance::linked,
      Instance::allowlist,
      Instance::blocklist
    ))?;

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
    Err(LemmyErrorType::InvalidPassword)?
  } else {
    Ok(())
  }
}

/// Checks for a honeypot. If this field is filled, fail the rest of the function
pub fn honeypot_check(honeypot: &Option<String>) -> Result<(), LemmyError> {
  if honeypot.is_some() && honeypot != &Some(String::new()) {
    Err(LemmyErrorType::HoneypotFailed)?
  } else {
    Ok(())
  }
}

pub async fn send_email_to_user(
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
    )
    .await
    {
      Ok(_o) => _o,
      Err(e) => warn!("{}", e),
    };
  }
}

pub async fn send_password_reset_email(
  user: &LocalUserView,
  pool: &mut DbPool<'_>,
  settings: &Settings,
) -> Result<(), LemmyError> {
  // Generate a random token
  let token = uuid::Uuid::new_v4().to_string();

  // Insert the row
  let local_user_id = user.local_user.id;
  PasswordResetRequest::create_token(pool, local_user_id, token.clone()).await?;

  let email = &user.local_user.email.clone().expect("email");
  let lang = get_interface_language(user);
  let subject = &lang.password_reset_subject(&user.person.name);
  let protocol_and_hostname = settings.get_protocol_and_hostname();
  let reset_link = format!("{}/password_change/{}", protocol_and_hostname, &token);
  let body = &lang.password_reset_body(reset_link, &user.person.name);
  send_email(subject, email, &user.person.name, body, settings).await
}

/// Send a verification email
pub async fn send_verification_email(
  user: &LocalUserView,
  new_email: &str,
  pool: &mut DbPool<'_>,
  settings: &Settings,
) -> Result<(), LemmyError> {
  let form = EmailVerificationForm {
    local_user_id: user.local_user.id,
    email: new_email.to_string(),
    verification_token: uuid::Uuid::new_v4().to_string(),
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
  send_email(&subject, new_email, &user.person.name, &body, settings).await?;

  Ok(())
}

pub fn get_interface_language(user: &LocalUserView) -> Lang {
  lang_str_to_lang(&user.local_user.interface_language)
}

pub fn get_interface_language_from_settings(user: &LocalUserView) -> Lang {
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

pub fn local_site_opt_to_sensitive(local_site: &Option<LocalSite>) -> bool {
  local_site
    .as_ref()
    .map(|site| site.enable_nsfw)
    .unwrap_or(false)
}

pub async fn send_application_approved_email(
  user: &LocalUserView,
  settings: &Settings,
) -> Result<(), LemmyError> {
  let email = &user.local_user.email.clone().expect("email");
  let lang = get_interface_language(user);
  let subject = lang.registration_approved_subject(&user.person.actor_id);
  let body = lang.registration_approved_body(&settings.hostname);
  send_email(&subject, email, &user.person.name, &body, settings).await
}

/// Send a new applicant email notification to all admins
pub async fn send_new_applicant_email_to_admins(
  applicant_username: &str,
  pool: &mut DbPool<'_>,
  settings: &Settings,
) -> Result<(), LemmyError> {
  // Collect the admins with emails
  let admins = LocalUserView::list_admins_with_emails(pool).await?;

  let applications_link = &format!(
    "{}/registration_applications",
    settings.get_protocol_and_hostname(),
  );

  for admin in &admins {
    let email = &admin.local_user.email.clone().expect("email");
    let lang = get_interface_language_from_settings(admin);
    let subject = lang.new_application_subject(&settings.hostname, applicant_username);
    let body = lang.new_application_body(applications_link);
    send_email(&subject, email, &admin.person.name, &body, settings).await?;
  }
  Ok(())
}

/// Send a report to all admins
pub async fn send_new_report_email_to_admins(
  reporter_username: &str,
  reported_username: &str,
  pool: &mut DbPool<'_>,
  settings: &Settings,
) -> Result<(), LemmyError> {
  // Collect the admins with emails
  let admins = LocalUserView::list_admins_with_emails(pool).await?;

  let reports_link = &format!("{}/reports", settings.get_protocol_and_hostname(),);

  for admin in &admins {
    let email = &admin.local_user.email.clone().expect("email");
    let lang = get_interface_language_from_settings(admin);
    let subject = lang.new_report_subject(&settings.hostname, reported_username, reporter_username);
    let body = lang.new_report_body(reports_link);
    send_email(&subject, email, &admin.person.name, &body, settings).await?;
  }
  Ok(())
}

pub fn check_private_instance_and_federation_enabled(
  local_site: &LocalSite,
) -> Result<(), LemmyError> {
  if local_site.private_instance && local_site.federation_enabled {
    Err(LemmyErrorType::CantEnablePrivateInstanceAndFederationTogether)?
  } else {
    Ok(())
  }
}

pub async fn purge_image_posts_for_person(
  banned_person_id: PersonId,
  context: &LemmyContext,
) -> Result<(), LemmyError> {
  let pool = &mut context.pool();
  let posts = Post::fetch_pictrs_posts_for_creator(pool, banned_person_id).await?;
  for post in posts {
    if let Some(url) = post.url {
      purge_image_from_pictrs(&url, context).await.ok();
    }
    if let Some(thumbnail_url) = post.thumbnail_url {
      purge_image_from_pictrs(&thumbnail_url, context).await.ok();
    }
  }

  Post::remove_pictrs_post_images_and_thumbnails_for_creator(pool, banned_person_id).await?;

  Ok(())
}

pub async fn purge_image_posts_for_community(
  banned_community_id: CommunityId,
  context: &LemmyContext,
) -> Result<(), LemmyError> {
  let pool = &mut context.pool();
  let posts = Post::fetch_pictrs_posts_for_community(pool, banned_community_id).await?;
  for post in posts {
    if let Some(url) = post.url {
      purge_image_from_pictrs(&url, context).await.ok();
    }
    if let Some(thumbnail_url) = post.thumbnail_url {
      purge_image_from_pictrs(&thumbnail_url, context).await.ok();
    }
  }

  Post::remove_pictrs_post_images_and_thumbnails_for_community(pool, banned_community_id).await?;

  Ok(())
}

pub async fn remove_user_data(
  banned_person_id: PersonId,
  context: &LemmyContext,
) -> Result<(), LemmyError> {
  let pool = &mut context.pool();
  // Purge user images
  let person = Person::read(pool, banned_person_id).await?;
  if let Some(avatar) = person.avatar {
    purge_image_from_pictrs(&avatar, context).await.ok();
  }
  if let Some(banner) = person.banner {
    purge_image_from_pictrs(&banner, context).await.ok();
  }

  // Update the fields to None
  Person::update(
    pool,
    banned_person_id,
    &PersonUpdateForm {
      avatar: Some(None),
      banner: Some(None),
      ..Default::default()
    },
  )
  .await?;

  // Posts
  Post::update_removed_for_creator(pool, banned_person_id, None, true).await?;

  // Purge image posts
  purge_image_posts_for_person(banned_person_id, context).await?;

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
      &CommunityUpdateForm {
        removed: Some(true),
        ..Default::default()
      },
    )
    .await?;

    // Delete the community images
    if let Some(icon) = first_mod_community.community.icon {
      purge_image_from_pictrs(&icon, context).await.ok();
    }
    if let Some(banner) = first_mod_community.community.banner {
      purge_image_from_pictrs(&banner, context).await.ok();
    }
    // Update the fields to None
    Community::update(
      pool,
      community_id,
      &CommunityUpdateForm {
        icon: Some(None),
        banner: Some(None),
        ..Default::default()
      },
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
  pool: &mut DbPool<'_>,
) -> Result<(), LemmyError> {
  // Posts
  Post::update_removed_for_creator(pool, banned_person_id, Some(community_id), true).await?;

  // Comments
  // TODO Diesel doesn't allow updates with joins, so this has to be a loop
  let comments = CommentQuery {
    creator_id: Some(banned_person_id),
    community_id: Some(community_id),
    ..Default::default()
  }
  .list(pool)
  .await?;

  for comment_view in &comments {
    let comment_id = comment_view.comment.id;
    Comment::update(
      pool,
      comment_id,
      &CommentUpdateForm {
        removed: Some(true),
        ..Default::default()
      },
    )
    .await?;
  }

  Ok(())
}

pub async fn purge_user_account(
  person_id: PersonId,
  context: &LemmyContext,
) -> Result<(), LemmyError> {
  let pool = &mut context.pool();
  // Delete their images
  let person = Person::read(pool, person_id).await?;
  if let Some(avatar) = person.avatar {
    purge_image_from_pictrs(&avatar, context).await.ok();
  }
  if let Some(banner) = person.banner {
    purge_image_from_pictrs(&banner, context).await.ok();
  }
  // No need to update avatar and banner, those are handled in Person::delete_account

  // Comments
  Comment::permadelete_for_creator(pool, person_id)
    .await
    .with_lemmy_type(LemmyErrorType::CouldntUpdateComment)?;

  // Posts
  Post::permadelete_for_creator(pool, person_id)
    .await
    .with_lemmy_type(LemmyErrorType::CouldntUpdatePost)?;

  // Purge image posts
  purge_image_posts_for_person(person_id, context).await?;

  // Leave communities they mod
  CommunityModerator::leave_all_communities(pool, person_id).await?;

  Person::delete_account(pool, person_id).await?;

  Ok(())
}

pub enum EndpointType {
  Community,
  Person,
  Post,
  Comment,
  PrivateMessage,
}

/// Generates an apub endpoint for a given domain, IE xyz.tld
pub fn generate_local_apub_endpoint(
  endpoint_type: EndpointType,
  name: &str,
  domain: &str,
) -> Result<DbUrl, ParseError> {
  let point = match endpoint_type {
    EndpointType::Community => "c",
    EndpointType::Person => "u",
    EndpointType::Post => "post",
    EndpointType::Comment => "comment",
    EndpointType::PrivateMessage => "private_message",
  };

  Ok(Url::parse(&format!("{domain}/{point}/{name}"))?.into())
}

pub fn generate_followers_url(actor_id: &DbUrl) -> Result<DbUrl, ParseError> {
  Ok(Url::parse(&format!("{actor_id}/followers"))?.into())
}

pub fn generate_inbox_url(actor_id: &DbUrl) -> Result<DbUrl, ParseError> {
  Ok(Url::parse(&format!("{actor_id}/inbox"))?.into())
}

pub fn generate_site_inbox_url(actor_id: &DbUrl) -> Result<DbUrl, ParseError> {
  let mut actor_id: Url = actor_id.clone().into();
  actor_id.set_path("site_inbox");
  Ok(actor_id.into())
}

pub fn generate_shared_inbox_url(actor_id: &DbUrl) -> Result<DbUrl, LemmyError> {
  let actor_id: Url = actor_id.clone().into();
  let url = format!(
    "{}://{}{}/inbox",
    &actor_id.scheme(),
    &actor_id.host_str().context(location_info!())?,
    if let Some(port) = actor_id.port() {
      format!(":{port}")
    } else {
      String::new()
    },
  );
  Ok(Url::parse(&url)?.into())
}

pub fn generate_outbox_url(actor_id: &DbUrl) -> Result<DbUrl, ParseError> {
  Ok(Url::parse(&format!("{actor_id}/outbox"))?.into())
}

pub fn generate_featured_url(actor_id: &DbUrl) -> Result<DbUrl, ParseError> {
  Ok(Url::parse(&format!("{actor_id}/featured"))?.into())
}

pub fn generate_moderators_url(community_id: &DbUrl) -> Result<DbUrl, LemmyError> {
  Ok(Url::parse(&format!("{community_id}/moderators"))?.into())
}

/// Replace special HTML characters in API parameters to prevent XSS attacks.
///
/// Taken from https://github.com/OWASP/CheatSheetSeries/blob/master/cheatsheets/Cross_Site_Scripting_Prevention_Cheat_Sheet.md#output-encoding-for-html-contexts
///
/// `>` is left in place because it is interpreted as markdown quote.
pub fn sanitize_html_api(data: &str) -> String {
  data
    .replace('&', "&amp;")
    .replace('<', "&lt;")
    .replace('\"', "&quot;")
    .replace('\'', "&#x27;")
}

pub fn sanitize_html_api_opt(data: &Option<String>) -> Option<String> {
  data.as_ref().map(|d| sanitize_html_api(d))
}

/// Replace special HTML characters in federation parameters to prevent XSS attacks.
///
/// Unlike [sanitize_html_api()] it leaves `&` in place to avoid double escaping.
pub fn sanitize_html_federation(data: &str) -> String {
  data
    .replace('<', "&lt;")
    .replace('\"', "&quot;")
    .replace('\'', "&#x27;")
}

pub fn sanitize_html_federation_opt(data: &Option<String>) -> Option<String> {
  data.as_ref().map(|d| sanitize_html_federation(d))
}

pub fn create_login_cookie(jwt: Sensitive<String>) -> Cookie<'static> {
  let mut cookie = Cookie::new(AUTH_COOKIE_NAME, jwt.into_inner());
  cookie.set_secure(true);
  cookie.set_same_site(SameSite::Strict);
  cookie.set_http_only(true);
  cookie
}

#[cfg(test)]
mod tests {
  #![allow(clippy::unwrap_used)]
  #![allow(clippy::indexing_slicing)]

  use crate::utils::{honeypot_check, password_length_check};

  #[test]
  #[rustfmt::skip]
  fn password_length() {
    assert!(password_length_check("Õ¼¾°3yË,o¸ãtÌÈú|ÇÁÙAøüÒI©·¤(T]/ð>æºWæ[C¤bªWöaÃÎñ·{=û³&§½K/c").is_ok());
    assert!(password_length_check("1234567890").is_ok());
    assert!(password_length_check("short").is_err());
    assert!(password_length_check("looooooooooooooooooooooooooooooooooooooooooooooooooooooooooong").is_err());
  }

  #[test]
  fn honeypot() {
    assert!(honeypot_check(&None).is_ok());
    assert!(honeypot_check(&Some(String::new())).is_ok());
    assert!(honeypot_check(&Some("1".to_string())).is_err());
    assert!(honeypot_check(&Some("message".to_string())).is_err());
  }
}
