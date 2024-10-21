use crate::{
  context::LemmyContext,
  request::{
    delete_image_from_pictrs,
    fetch_pictrs_proxied_image_details,
    purge_image_from_pictrs,
  },
  site::{FederatedInstances, InstanceWithFederationState},
};
use chrono::{DateTime, Days, Local, TimeZone, Utc};
use enum_map::{enum_map, EnumMap};
use lemmy_db_schema::{
  aggregates::structs::{PersonPostAggregates, PersonPostAggregatesForm},
  newtypes::{CommunityId, DbUrl, InstanceId, PersonId, PostId},
  source::{
    comment::{Comment, CommentUpdateForm},
    community::{Community, CommunityModerator, CommunityUpdateForm},
    community_block::CommunityBlock,
    email_verification::{EmailVerification, EmailVerificationForm},
    images::RemoteImage,
    instance::Instance,
    instance_block::InstanceBlock,
    local_site::LocalSite,
    local_site_rate_limit::LocalSiteRateLimit,
    local_site_url_blocklist::LocalSiteUrlBlocklist,
    password_reset_request::PasswordResetRequest,
    person::{Person, PersonUpdateForm},
    person_block::PersonBlock,
    post::{Post, PostRead},
    site::Site,
  },
  traits::Crud,
  utils::DbPool,
};
use lemmy_db_views::{
  comment_view::CommentQuery,
  structs::{LocalImageView, LocalUserView},
};
use lemmy_db_views_actor::structs::{
  CommunityModeratorView,
  CommunityPersonBanView,
  CommunityView,
};
use lemmy_utils::{
  email::{send_email, translations::Lang},
  error::{LemmyError, LemmyErrorExt, LemmyErrorType, LemmyResult},
  rate_limit::{ActionType, BucketConfig},
  settings::structs::{PictrsImageMode, Settings},
  utils::{
    markdown::{markdown_check_for_blocked_urls, markdown_rewrite_image_links},
    slurs::{build_slur_regex, remove_slurs},
    validation::clean_urls_in_text,
  },
  CACHE_DURATION_FEDERATION,
};
use moka::future::Cache;
use regex::{escape, Regex, RegexSet};
use rosetta_i18n::{Language, LanguageId};
use std::{collections::HashSet, sync::LazyLock};
use tracing::warn;
use url::{ParseError, Url};
use urlencoding::encode;

pub static AUTH_COOKIE_NAME: &str = "jwt";

#[tracing::instrument(skip_all)]
pub async fn is_mod_or_admin(
  pool: &mut DbPool<'_>,
  person: &Person,
  community_id: CommunityId,
) -> LemmyResult<()> {
  check_user_valid(person)?;

  let is_mod_or_admin = CommunityView::is_mod_or_admin(pool, person.id, community_id).await?;
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
) -> LemmyResult<()> {
  if let Some(local_user_view) = local_user_view {
    if let Some(community_id) = community_id {
      is_mod_or_admin(pool, &local_user_view.person, community_id).await
    } else {
      is_admin(local_user_view)
    }
  } else {
    Err(LemmyErrorType::NotAModOrAdmin)?
  }
}

/// Check that a person is either a mod of any community, or an admin
///
/// Should only be used for read operations
#[tracing::instrument(skip_all)]
pub async fn check_community_mod_of_any_or_admin_action(
  local_user_view: &LocalUserView,
  pool: &mut DbPool<'_>,
) -> LemmyResult<()> {
  let person = &local_user_view.person;

  check_user_valid(person)?;

  let is_mod_of_any_or_admin = CommunityView::is_mod_of_any_or_admin(pool, person.id).await?;
  if !is_mod_of_any_or_admin {
    Err(LemmyErrorType::NotAModOrAdmin)?
  } else {
    Ok(())
  }
}

pub fn is_admin(local_user_view: &LocalUserView) -> LemmyResult<()> {
  check_user_valid(&local_user_view.person)?;
  if !local_user_view.local_user.admin {
    Err(LemmyErrorType::NotAnAdmin)?
  } else if local_user_view.person.banned {
    Err(LemmyErrorType::Banned)?
  } else {
    Ok(())
  }
}

pub fn is_top_mod(
  local_user_view: &LocalUserView,
  community_mods: &[CommunityModeratorView],
) -> LemmyResult<()> {
  check_user_valid(&local_user_view.person)?;
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

/// Marks a post as read for a given person.
#[tracing::instrument(skip_all)]
pub async fn mark_post_as_read(
  person_id: PersonId,
  post_id: PostId,
  pool: &mut DbPool<'_>,
) -> LemmyResult<()> {
  PostRead::mark_as_read(pool, HashSet::from([post_id]), person_id)
    .await
    .with_lemmy_type(LemmyErrorType::CouldntMarkPostAsRead)?;
  Ok(())
}

/// Updates the read comment count for a post. Usually done when reading or creating a new comment.
#[tracing::instrument(skip_all)]
pub async fn update_read_comments(
  person_id: PersonId,
  post_id: PostId,
  read_comments: i64,
  pool: &mut DbPool<'_>,
) -> LemmyResult<()> {
  let person_post_agg_form = PersonPostAggregatesForm {
    person_id,
    post_id,
    read_comments,
    ..PersonPostAggregatesForm::default()
  };

  PersonPostAggregates::upsert(pool, &person_post_agg_form)
    .await
    .with_lemmy_type(LemmyErrorType::CouldntFindPost)?;

  Ok(())
}

pub fn check_user_valid(person: &Person) -> LemmyResult<()> {
  // Check for a site ban
  if person.banned {
    Err(LemmyErrorType::SiteBan)?
  }
  // check for account deletion
  else if person.deleted {
    Err(LemmyErrorType::Deleted)?
  } else {
    Ok(())
  }
}

/// Checks that a normal user action (eg posting or voting) is allowed in a given community.
///
/// In particular it checks that neither the user nor community are banned or deleted, and that
/// the user isn't banned.
pub async fn check_community_user_action(
  person: &Person,
  community_id: CommunityId,
  pool: &mut DbPool<'_>,
) -> LemmyResult<()> {
  check_user_valid(person)?;
  check_community_deleted_removed(community_id, pool).await?;
  check_community_ban(person, community_id, pool).await?;
  Ok(())
}

async fn check_community_deleted_removed(
  community_id: CommunityId,
  pool: &mut DbPool<'_>,
) -> LemmyResult<()> {
  let community = Community::read(pool, community_id)
    .await?
    .ok_or(LemmyErrorType::CouldntFindCommunity)?;
  if community.deleted || community.removed {
    Err(LemmyErrorType::Deleted)?
  }
  Ok(())
}

async fn check_community_ban(
  person: &Person,
  community_id: CommunityId,
  pool: &mut DbPool<'_>,
) -> LemmyResult<()> {
  // check if user was banned from site or community
  let is_banned = CommunityPersonBanView::get(pool, person.id, community_id).await?;
  if is_banned {
    Err(LemmyErrorType::BannedFromCommunity)?
  }
  Ok(())
}

/// Check that the given user can perform a mod action in the community.
///
/// In particular it checks that he is an admin or mod, wasn't banned and the community isn't
/// removed/deleted.
pub async fn check_community_mod_action(
  person: &Person,
  community_id: CommunityId,
  allow_deleted: bool,
  pool: &mut DbPool<'_>,
) -> LemmyResult<()> {
  is_mod_or_admin(pool, person, community_id).await?;
  check_community_ban(person, community_id, pool).await?;

  // it must be possible to restore deleted community
  if !allow_deleted {
    check_community_deleted_removed(community_id, pool).await?;
  }
  Ok(())
}

/// Don't allow creating reports for removed / deleted posts
pub fn check_post_deleted_or_removed(post: &Post) -> LemmyResult<()> {
  if post.deleted || post.removed {
    Err(LemmyErrorType::Deleted)?
  } else {
    Ok(())
  }
}

pub fn check_comment_deleted_or_removed(comment: &Comment) -> LemmyResult<()> {
  if comment.deleted || comment.removed {
    Err(LemmyErrorType::Deleted)?
  } else {
    Ok(())
  }
}

/// Throws an error if a recipient has blocked a person.
#[tracing::instrument(skip_all)]
pub async fn check_person_block(
  my_id: PersonId,
  potential_blocker_id: PersonId,
  pool: &mut DbPool<'_>,
) -> LemmyResult<()> {
  let is_blocked = PersonBlock::read(pool, potential_blocker_id, my_id).await?;
  if is_blocked {
    Err(LemmyErrorType::PersonIsBlocked)?
  } else {
    Ok(())
  }
}

/// Throws an error if a recipient has blocked a community.
#[tracing::instrument(skip_all)]
async fn check_community_block(
  community_id: CommunityId,
  person_id: PersonId,
  pool: &mut DbPool<'_>,
) -> LemmyResult<()> {
  let is_blocked = CommunityBlock::read(pool, person_id, community_id).await?;
  if is_blocked {
    Err(LemmyErrorType::CommunityIsBlocked)?
  } else {
    Ok(())
  }
}

/// Throws an error if a recipient has blocked an instance.
#[tracing::instrument(skip_all)]
async fn check_instance_block(
  instance_id: InstanceId,
  person_id: PersonId,
  pool: &mut DbPool<'_>,
) -> LemmyResult<()> {
  let is_blocked = InstanceBlock::read(pool, person_id, instance_id).await?;
  if is_blocked {
    Err(LemmyErrorType::InstanceIsBlocked)?
  } else {
    Ok(())
  }
}

#[tracing::instrument(skip_all)]
pub async fn check_person_instance_community_block(
  my_id: PersonId,
  potential_blocker_id: PersonId,
  community_instance_id: InstanceId,
  community_id: CommunityId,
  pool: &mut DbPool<'_>,
) -> LemmyResult<()> {
  check_person_block(my_id, potential_blocker_id, pool).await?;
  check_instance_block(community_instance_id, potential_blocker_id, pool).await?;
  check_community_block(community_id, potential_blocker_id, pool).await?;
  Ok(())
}

#[tracing::instrument(skip_all)]
pub fn check_downvotes_enabled(score: i16, local_site: &LocalSite) -> LemmyResult<()> {
  if score == -1 && !local_site.enable_downvotes {
    Err(LemmyErrorType::DownvotesAreDisabled)?
  } else {
    Ok(())
  }
}

/// Dont allow bots to do certain actions, like voting
#[tracing::instrument(skip_all)]
pub fn check_bot_account(person: &Person) -> LemmyResult<()> {
  if person.bot_account {
    Err(LemmyErrorType::InvalidBotAction)?
  } else {
    Ok(())
  }
}

#[tracing::instrument(skip_all)]
pub fn check_private_instance(
  local_user_view: &Option<LocalUserView>,
  local_site: &LocalSite,
) -> LemmyResult<()> {
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
) -> LemmyResult<Option<FederatedInstances>> {
  if local_site.federation_enabled {
    let mut linked = Vec::new();
    let mut allowed = Vec::new();
    let mut blocked = Vec::new();

    let all = Instance::read_all_with_fed_state(pool).await?;
    for (instance, federation_state, is_blocked, is_allowed) in all {
      let i = InstanceWithFederationState {
        instance,
        federation_state: federation_state.map(std::convert::Into::into),
      };
      if is_blocked {
        // blocked instances will only have an entry here if they had been federated with in the
        // past.
        blocked.push(i);
      } else if is_allowed {
        allowed.push(i.clone());
        linked.push(i);
      } else {
        // not explicitly allowed but implicitly linked
        linked.push(i);
      }
    }

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
pub fn password_length_check(pass: &str) -> LemmyResult<()> {
  if !(10..=60).contains(&pass.chars().count()) {
    Err(LemmyErrorType::InvalidPassword)?
  } else {
    Ok(())
  }
}

/// Checks for a honeypot. If this field is filled, fail the rest of the function
pub fn honeypot_check(honeypot: &Option<String>) -> LemmyResult<()> {
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
) -> LemmyResult<()> {
  // Generate a random token
  let token = uuid::Uuid::new_v4().to_string();

  let email = &user.local_user.email.clone().expect("email");
  let lang = get_interface_language(user);
  let subject = &lang.password_reset_subject(&user.person.name);
  let protocol_and_hostname = settings.get_protocol_and_hostname();
  let reset_link = format!("{}/password_change/{}", protocol_and_hostname, &token);
  let body = &lang.password_reset_body(reset_link, &user.person.name);
  send_email(subject, email, &user.person.name, body, settings).await?;

  // Insert the row after successful send, to avoid using daily reset limit while
  // email sending is broken.
  let local_user_id = user.local_user.id;
  PasswordResetRequest::create(pool, local_user_id, token.clone()).await?;
  Ok(())
}

/// Send a verification email
pub async fn send_verification_email(
  user: &LocalUserView,
  new_email: &str,
  pool: &mut DbPool<'_>,
  settings: &Settings,
) -> LemmyResult<()> {
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
  l: &LocalSiteRateLimit,
) -> EnumMap<ActionType, BucketConfig> {
  enum_map! {
    ActionType::Message => (l.message, l.message_per_second),
    ActionType::Post => (l.post, l.post_per_second),
    ActionType::Register => (l.register, l.register_per_second),
    ActionType::Image => (l.image, l.image_per_second),
    ActionType::Comment => (l.comment, l.comment_per_second),
    ActionType::Search => (l.search, l.search_per_second),
    ActionType::ImportUserSettings => (l.import_user_settings, l.import_user_settings_per_second),
  }
  .map(|_key, (capacity, secs_to_refill)| BucketConfig {
    capacity: u32::try_from(capacity).unwrap_or(0),
    secs_to_refill: u32::try_from(secs_to_refill).unwrap_or(0),
  })
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

pub async fn get_url_blocklist(context: &LemmyContext) -> LemmyResult<RegexSet> {
  static URL_BLOCKLIST: LazyLock<Cache<(), RegexSet>> = LazyLock::new(|| {
    Cache::builder()
      .max_capacity(1)
      .time_to_live(CACHE_DURATION_FEDERATION)
      .build()
  });

  Ok(
    URL_BLOCKLIST
      .try_get_with::<_, LemmyError>((), async {
        let urls = LocalSiteUrlBlocklist::get_all(&mut context.pool()).await?;

        // The urls are already validated on saving, so just escape them.
        let regexes = urls.iter().map(|url| escape(&url.url));

        let set = RegexSet::new(regexes)?;
        Ok(set)
      })
      .await
      .map_err(|e| anyhow::anyhow!("Failed to build URL blocklist due to `{}`", e))?,
  )
}

pub async fn send_application_approved_email(
  user: &LocalUserView,
  settings: &Settings,
) -> LemmyResult<()> {
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
) -> LemmyResult<()> {
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
) -> LemmyResult<()> {
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

pub fn check_private_instance_and_federation_enabled(local_site: &LocalSite) -> LemmyResult<()> {
  if local_site.private_instance && local_site.federation_enabled {
    Err(LemmyErrorType::CantEnablePrivateInstanceAndFederationTogether)?
  } else {
    Ok(())
  }
}

/// Read the site for an actor_id.
///
/// Used for GetCommunityResponse and GetPersonDetails
pub async fn read_site_for_actor(
  actor_id: DbUrl,
  context: &LemmyContext,
) -> LemmyResult<Option<Site>> {
  let site_id = Site::instance_actor_id_from_url(actor_id.clone().into());
  let site = Site::read_from_apub_id(&mut context.pool(), &site_id.into()).await?;
  Ok(site)
}

pub async fn purge_image_posts_for_person(
  banned_person_id: PersonId,
  context: &LemmyContext,
) -> LemmyResult<()> {
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

/// Delete a local_user's images
async fn delete_local_user_images(person_id: PersonId, context: &LemmyContext) -> LemmyResult<()> {
  if let Ok(Some(local_user)) = LocalUserView::read_person(&mut context.pool(), person_id).await {
    let pictrs_uploads =
      LocalImageView::get_all_by_local_user_id(&mut context.pool(), local_user.local_user.id)
        .await?;

    // Delete their images
    for upload in pictrs_uploads {
      delete_image_from_pictrs(
        &upload.local_image.pictrs_alias,
        &upload.local_image.pictrs_delete_token,
        context,
      )
      .await
      .ok();
    }
  }
  Ok(())
}

pub async fn purge_image_posts_for_community(
  banned_community_id: CommunityId,
  context: &LemmyContext,
) -> LemmyResult<()> {
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
) -> LemmyResult<()> {
  let pool = &mut context.pool();
  // Purge user images
  let person = Person::read(pool, banned_person_id)
    .await?
    .ok_or(LemmyErrorType::CouldntFindPerson)?;
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
      bio: Some(None),
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
) -> LemmyResult<()> {
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

pub async fn purge_user_account(person_id: PersonId, context: &LemmyContext) -> LemmyResult<()> {
  let pool = &mut context.pool();

  let person = Person::read(pool, person_id)
    .await?
    .ok_or(LemmyErrorType::CouldntFindPerson)?;

  // Delete their local images, if they're a local user
  delete_local_user_images(person_id, context).await.ok();

  // No need to update avatar and banner, those are handled in Person::delete_account
  if let Some(avatar) = person.avatar {
    purge_image_from_pictrs(&avatar, context).await.ok();
  }
  if let Some(banner) = person.banner {
    purge_image_from_pictrs(&banner, context).await.ok();
  }

  // Purge image posts
  purge_image_posts_for_person(person_id, context).await.ok();

  // Comments
  Comment::permadelete_for_creator(pool, person_id)
    .await
    .with_lemmy_type(LemmyErrorType::CouldntUpdateComment)?;

  // Posts
  Post::permadelete_for_creator(pool, person_id)
    .await
    .with_lemmy_type(LemmyErrorType::CouldntUpdatePost)?;

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

pub fn generate_shared_inbox_url(settings: &Settings) -> LemmyResult<DbUrl> {
  let url = format!("{}/inbox", settings.get_protocol_and_hostname());
  Ok(Url::parse(&url)?.into())
}

pub fn generate_outbox_url(actor_id: &DbUrl) -> Result<DbUrl, ParseError> {
  Ok(Url::parse(&format!("{actor_id}/outbox"))?.into())
}

pub fn generate_featured_url(actor_id: &DbUrl) -> Result<DbUrl, ParseError> {
  Ok(Url::parse(&format!("{actor_id}/featured"))?.into())
}

pub fn generate_moderators_url(community_id: &DbUrl) -> LemmyResult<DbUrl> {
  Ok(Url::parse(&format!("{community_id}/moderators"))?.into())
}

/// Ensure that ban/block expiry is in valid range. If its in past, throw error. If its more
/// than 10 years in future, convert to permanent ban. Otherwise return the same value.
pub fn check_expire_time(expires_unix_opt: Option<i64>) -> LemmyResult<Option<DateTime<Utc>>> {
  if let Some(expires_unix) = expires_unix_opt {
    let expires = Utc
      .timestamp_opt(expires_unix, 0)
      .single()
      .ok_or(LemmyErrorType::InvalidUnixTime)?;

    limit_expire_time(expires)
  } else {
    Ok(None)
  }
}

fn limit_expire_time(expires: DateTime<Utc>) -> LemmyResult<Option<DateTime<Utc>>> {
  const MAX_BAN_TERM: Days = Days::new(10 * 365);

  if expires < Local::now() {
    Err(LemmyErrorType::BanExpirationInPast)?
  } else if expires > Local::now() + MAX_BAN_TERM {
    Ok(None)
  } else {
    Ok(Some(expires))
  }
}

pub async fn process_markdown(
  text: &str,
  slur_regex: &Option<Regex>,
  url_blocklist: &RegexSet,
  context: &LemmyContext,
) -> LemmyResult<String> {
  let text = remove_slurs(text, slur_regex);
  let text = clean_urls_in_text(&text);

  markdown_check_for_blocked_urls(&text, url_blocklist)?;

  if context.settings().pictrs_config()?.image_mode() == PictrsImageMode::ProxyAllImages {
    let (text, links) = markdown_rewrite_image_links(text);

    // Create images and image detail rows
    for link in links {
      // Insert image details for the remote image
      let details_res = fetch_pictrs_proxied_image_details(&link, context).await;
      if let Ok(details) = details_res {
        let proxied =
          build_proxied_image_url(&link, &context.settings().get_protocol_and_hostname())?;
        let details_form = details.build_image_details_form(&proxied);
        RemoteImage::create(&mut context.pool(), &details_form).await?;
      }
    }
    Ok(text)
  } else {
    Ok(text)
  }
}

pub async fn process_markdown_opt(
  text: &Option<String>,
  slur_regex: &Option<Regex>,
  url_blocklist: &RegexSet,
  context: &LemmyContext,
) -> LemmyResult<Option<String>> {
  match text {
    Some(t) => process_markdown(t, slur_regex, url_blocklist, context)
      .await
      .map(Some),
    None => Ok(None),
  }
}

/// A wrapper for `proxy_image_link` for use in tests.
///
/// The parameter `force_image_proxy` is the config value of `pictrs.image_proxy`. Its necessary to
/// pass as separate parameter so it can be changed in tests.
async fn proxy_image_link_internal(
  link: Url,
  image_mode: PictrsImageMode,
  context: &LemmyContext,
) -> LemmyResult<DbUrl> {
  // Dont rewrite links pointing to local domain.
  if link.domain() == Some(&context.settings().hostname) {
    Ok(link.into())
  } else if image_mode == PictrsImageMode::ProxyAllImages {
    let proxied = build_proxied_image_url(&link, &context.settings().get_protocol_and_hostname())?;
    // This should fail softly, since pictrs might not even be running
    let details_res = fetch_pictrs_proxied_image_details(&link, context).await;

    if let Ok(details) = details_res {
      let details_form = details.build_image_details_form(&proxied);
      RemoteImage::create(&mut context.pool(), &details_form).await?;
    };

    Ok(proxied.into())
  } else {
    Ok(link.into())
  }
}

/// Rewrite a link to go through `/api/v3/image_proxy` endpoint. This is only for remote urls and
/// if image_proxy setting is enabled.
pub(crate) async fn proxy_image_link(link: Url, context: &LemmyContext) -> LemmyResult<DbUrl> {
  proxy_image_link_internal(
    link,
    context.settings().pictrs_config()?.image_mode(),
    context,
  )
  .await
}

pub async fn proxy_image_link_opt_api(
  link: Option<Option<DbUrl>>,
  context: &LemmyContext,
) -> LemmyResult<Option<Option<DbUrl>>> {
  if let Some(Some(link)) = link {
    proxy_image_link(link.into(), context)
      .await
      .map(Some)
      .map(Some)
  } else {
    Ok(link)
  }
}

pub async fn proxy_image_link_api(
  link: Option<DbUrl>,
  context: &LemmyContext,
) -> LemmyResult<Option<DbUrl>> {
  if let Some(link) = link {
    proxy_image_link(link.into(), context).await.map(Some)
  } else {
    Ok(link)
  }
}

pub async fn proxy_image_link_opt_apub(
  link: Option<Url>,
  context: &LemmyContext,
) -> LemmyResult<Option<DbUrl>> {
  if let Some(l) = link {
    proxy_image_link(l, context).await.map(Some)
  } else {
    Ok(None)
  }
}

fn build_proxied_image_url(
  link: &Url,
  protocol_and_hostname: &str,
) -> Result<Url, url::ParseError> {
  Url::parse(&format!(
    "{}/api/v3/image_proxy?url={}",
    protocol_and_hostname,
    encode(link.as_str())
  ))
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
#[allow(clippy::indexing_slicing)]
mod tests {

  use super::*;
  use pretty_assertions::assert_eq;
  use serial_test::serial;

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

  #[test]
  fn test_limit_ban_term() {
    // Ban expires in past, should throw error
    assert!(limit_expire_time(Utc::now() - Days::new(5)).is_err());

    // Legitimate ban term, return same value
    let fourteen_days = Utc::now() + Days::new(14);
    assert_eq!(
      limit_expire_time(fourteen_days).unwrap(),
      Some(fourteen_days)
    );
    let nine_years = Utc::now() + Days::new(365 * 9);
    assert_eq!(limit_expire_time(nine_years).unwrap(), Some(nine_years));

    // Too long ban term, changes to None (permanent ban)
    assert_eq!(
      limit_expire_time(Utc::now() + Days::new(365 * 11)).unwrap(),
      None
    );
  }

  #[tokio::test]
  #[serial]
  async fn test_proxy_image_link() {
    let context = LemmyContext::init_test_context().await;

    // image from local domain is unchanged
    let local_url = Url::parse("http://lemmy-alpha/image.png").unwrap();
    let proxied =
      proxy_image_link_internal(local_url.clone(), PictrsImageMode::ProxyAllImages, &context)
        .await
        .unwrap();
    assert_eq!(&local_url, proxied.inner());

    // image from remote domain is proxied
    let remote_image = Url::parse("http://lemmy-beta/image.png").unwrap();
    let proxied = proxy_image_link_internal(
      remote_image.clone(),
      PictrsImageMode::ProxyAllImages,
      &context,
    )
    .await
    .unwrap();
    assert_eq!(
      "https://lemmy-alpha/api/v3/image_proxy?url=http%3A%2F%2Flemmy-beta%2Fimage.png",
      proxied.as_str()
    );

    // This fails, because the details can't be fetched without pictrs running,
    // And a remote image won't be inserted.
    assert!(
      RemoteImage::validate(&mut context.pool(), remote_image.into())
        .await
        .is_err()
    );
  }
}
