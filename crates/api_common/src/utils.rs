use crate::{
  claims::Claims,
  context::LemmyContext,
  request::{
    delete_image_from_pictrs,
    fetch_pictrs_proxied_image_details,
    purge_image_from_pictrs_url,
  },
  site::{FederatedInstances, InstanceWithFederationState},
};
use actix_web::{http::header::Header, HttpRequest};
use actix_web_httpauth::headers::authorization::{Authorization, Bearer};
use chrono::{DateTime, Days, Local, TimeZone, Utc};
use enum_map::{enum_map, EnumMap};
use lemmy_db_schema::{
  newtypes::{CommentId, CommunityId, DbUrl, InstanceId, PersonId, PostId, PostOrCommentId},
  source::{
    comment::{Comment, CommentActions},
    community::{Community, CommunityActions, CommunityUpdateForm},
    images::{ImageDetails, RemoteImage},
    instance::{Instance, InstanceActions},
    local_site::LocalSite,
    local_site_rate_limit::LocalSiteRateLimit,
    local_site_url_blocklist::LocalSiteUrlBlocklist,
    mod_log::moderator::{
      ModRemoveComment,
      ModRemoveCommentForm,
      ModRemovePost,
      ModRemovePostForm,
    },
    oauth_account::OAuthAccount,
    person::{Person, PersonActions, PersonUpdateForm},
    post::{Post, PostActions, PostReadCommentsForm},
    private_message::PrivateMessage,
    registration_application::RegistrationApplication,
    site::Site,
  },
  traits::{Blockable, Crud, Likeable, ReadComments},
  utils::DbPool,
};
use lemmy_db_schema_file::enums::{FederationMode, RegistrationMode};
use lemmy_db_views_community_follower::CommunityFollowerView;
use lemmy_db_views_community_moderator::CommunityModeratorView;
use lemmy_db_views_community_person_ban::CommunityPersonBanView;
use lemmy_db_views_local_image::LocalImageView;
use lemmy_db_views_local_user::LocalUserView;
use lemmy_db_views_person::PersonView;
use lemmy_db_views_site::SiteView;
use lemmy_utils::{
  error::{LemmyError, LemmyErrorExt, LemmyErrorExt2, LemmyErrorType, LemmyResult},
  rate_limit::{ActionType, BucketConfig},
  settings::{structs::PictrsImageMode, SETTINGS},
  spawn_try_task,
  utils::{
    markdown::{image_links::markdown_rewrite_image_links, markdown_check_for_blocked_urls},
    slurs::remove_slurs,
    validation::{build_and_check_regex, clean_urls_in_text},
  },
  CacheLock,
  CACHE_DURATION_FEDERATION,
};
use moka::future::Cache;
use regex::{escape, Regex, RegexSet};
use std::sync::LazyLock;
use tracing::Instrument;
use url::{ParseError, Url};
use urlencoding::encode;
use webmention::{Webmention, WebmentionError};

pub const AUTH_COOKIE_NAME: &str = "jwt";

pub async fn check_is_mod_or_admin(
  pool: &mut DbPool<'_>,
  person_id: PersonId,
  community_id: CommunityId,
  local_instance_id: InstanceId,
) -> LemmyResult<()> {
  let is_mod =
    CommunityModeratorView::check_is_community_moderator(pool, community_id, person_id).await;
  if is_mod.is_ok()
    || PersonView::read(pool, person_id, local_instance_id, false)
      .await
      .is_ok_and(|t| t.is_admin)
  {
    Ok(())
  } else {
    Err(LemmyErrorType::NotAModOrAdmin)?
  }
}

/// Checks if a person is an admin, or moderator of any community.
pub(crate) async fn check_is_mod_of_any_or_admin(
  pool: &mut DbPool<'_>,
  person_id: PersonId,
  local_instance_id: InstanceId,
) -> LemmyResult<()> {
  let is_mod_of_any = CommunityModeratorView::is_community_moderator_of_any(pool, person_id).await;
  if is_mod_of_any.is_ok()
    || PersonView::read(pool, person_id, local_instance_id, false)
      .await
      .is_ok_and(|t| t.is_admin)
  {
    Ok(())
  } else {
    Err(LemmyErrorType::NotAModOrAdmin)?
  }
}

pub async fn is_mod_or_admin(
  pool: &mut DbPool<'_>,
  local_user_view: &LocalUserView,
  community_id: CommunityId,
) -> LemmyResult<()> {
  check_local_user_valid(local_user_view)?;
  check_is_mod_or_admin(
    pool,
    local_user_view.person.id,
    community_id,
    local_user_view.person.instance_id,
  )
  .await
}

pub async fn is_mod_or_admin_opt(
  pool: &mut DbPool<'_>,
  local_user_view: Option<&LocalUserView>,
  community_id: Option<CommunityId>,
) -> LemmyResult<()> {
  if let Some(local_user_view) = local_user_view {
    if let Some(community_id) = community_id {
      is_mod_or_admin(pool, local_user_view, community_id).await
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
pub async fn check_community_mod_of_any_or_admin_action(
  local_user_view: &LocalUserView,
  pool: &mut DbPool<'_>,
) -> LemmyResult<()> {
  let person = &local_user_view.person;

  check_local_user_valid(local_user_view)?;
  check_is_mod_of_any_or_admin(pool, person.id, person.instance_id).await
}

pub fn is_admin(local_user_view: &LocalUserView) -> LemmyResult<()> {
  check_local_user_valid(local_user_view)?;
  if !local_user_view.local_user.admin {
    Err(LemmyErrorType::NotAnAdmin)?
  } else {
    Ok(())
  }
}

pub fn is_top_mod(
  local_user_view: &LocalUserView,
  community_mods: &[CommunityModeratorView],
) -> LemmyResult<()> {
  check_local_user_valid(local_user_view)?;
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

/// Updates the read comment count for a post. Usually done when reading or creating a new comment.
pub async fn update_read_comments(
  person_id: PersonId,
  post_id: PostId,
  read_comments: i64,
  pool: &mut DbPool<'_>,
) -> LemmyResult<()> {
  let person_post_agg_form = PostReadCommentsForm::new(post_id, person_id, read_comments);
  PostActions::update_read_comments(pool, &person_post_agg_form).await?;

  Ok(())
}

pub fn check_local_user_valid(local_user_view: &LocalUserView) -> LemmyResult<()> {
  // Check for a site ban
  if local_user_view.banned() {
    Err(LemmyErrorType::SiteBan)?
  }
  // check for account deletion
  else if local_user_view.person.deleted {
    Err(LemmyErrorType::Deleted)?
  } else {
    Ok(())
  }
}

pub fn check_person_valid(person_view: &PersonView) -> LemmyResult<()> {
  // Check for a site ban
  if person_view.creator_banned {
    Err(LemmyErrorType::SiteBan)?
  }
  // check for account deletion
  else if person_view.person.deleted {
    Err(LemmyErrorType::Deleted)?
  } else {
    Ok(())
  }
}

/// Check if the user's email is verified if email verification is turned on
/// However, skip checking verification if the user is an admin
pub fn check_email_verified(
  local_user_view: &LocalUserView,
  site_view: &SiteView,
) -> LemmyResult<()> {
  if !local_user_view.local_user.admin
    && site_view.local_site.require_email_verification
    && !local_user_view.local_user.email_verified
  {
    Err(LemmyErrorType::EmailNotVerified)?
  }
  Ok(())
}

pub async fn check_registration_application(
  local_user_view: &LocalUserView,
  local_site: &LocalSite,
  pool: &mut DbPool<'_>,
) -> LemmyResult<()> {
  if (local_site.registration_mode == RegistrationMode::RequireApplication
    || local_site.registration_mode == RegistrationMode::Closed)
    && !local_user_view.local_user.accepted_application
    && !local_user_view.local_user.admin
  {
    // Fetch the registration application. If no admin id is present its still pending. Otherwise it
    // was processed (either accepted or denied).
    let local_user_id = local_user_view.local_user.id;
    let registration = RegistrationApplication::find_by_local_user_id(pool, local_user_id).await?;
    if registration.admin_id.is_some() {
      Err(LemmyErrorType::RegistrationDenied {
        reason: registration.deny_reason,
      })?
    } else {
      Err(LemmyErrorType::RegistrationApplicationIsPending)?
    }
  }
  Ok(())
}

/// Checks that a normal user action (eg posting or voting) is allowed in a given community.
///
/// In particular it checks that neither the user nor community are banned or deleted, and that
/// the user isn't banned.
pub async fn check_community_user_action(
  local_user_view: &LocalUserView,
  community: &Community,
  pool: &mut DbPool<'_>,
) -> LemmyResult<()> {
  check_local_user_valid(local_user_view)?;
  check_community_deleted_removed(community)?;
  CommunityPersonBanView::check(pool, local_user_view.person.id, community.id).await?;
  CommunityFollowerView::check_private_community_action(pool, local_user_view.person.id, community)
    .await?;
  InstanceActions::check_ban(pool, local_user_view.person.id, community.instance_id).await?;
  Ok(())
}

pub fn check_community_deleted_removed(community: &Community) -> LemmyResult<()> {
  if community.deleted || community.removed {
    Err(LemmyErrorType::Deleted)?
  }
  Ok(())
}

/// Check that the given user can perform a mod action in the community.
///
/// In particular it checks that he is an admin or mod, wasn't banned and the community isn't
/// removed/deleted.
pub async fn check_community_mod_action(
  local_user_view: &LocalUserView,
  community: &Community,
  allow_deleted: bool,
  pool: &mut DbPool<'_>,
) -> LemmyResult<()> {
  is_mod_or_admin(pool, local_user_view, community.id).await?;
  CommunityPersonBanView::check(pool, local_user_view.person.id, community.id).await?;

  // it must be possible to restore deleted community
  if !allow_deleted {
    check_community_deleted_removed(community)?;
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

pub async fn check_person_instance_community_block(
  my_id: PersonId,
  potential_blocker_id: PersonId,
  community_instance_id: InstanceId,
  community_id: CommunityId,
  pool: &mut DbPool<'_>,
) -> LemmyResult<()> {
  PersonActions::read_block(pool, potential_blocker_id, my_id).await?;
  InstanceActions::read_block(pool, potential_blocker_id, community_instance_id).await?;
  CommunityActions::read_block(pool, potential_blocker_id, community_id).await?;
  Ok(())
}

pub async fn check_local_vote_mode(
  score: i16,
  post_or_comment_id: PostOrCommentId,
  local_site: &LocalSite,
  person_id: PersonId,
  pool: &mut DbPool<'_>,
) -> LemmyResult<()> {
  let (downvote_setting, upvote_setting) = match post_or_comment_id {
    PostOrCommentId::Post(_) => (local_site.post_downvotes, local_site.post_upvotes),
    PostOrCommentId::Comment(_) => (local_site.comment_downvotes, local_site.comment_upvotes),
  };

  let downvote_fail = score == -1 && downvote_setting == FederationMode::Disable;
  let upvote_fail = score == 1 && upvote_setting == FederationMode::Disable;

  // Undo previous vote for item if new vote fails
  if downvote_fail || upvote_fail {
    match post_or_comment_id {
      PostOrCommentId::Post(post_id) => PostActions::remove_like(pool, person_id, post_id).await?,
      PostOrCommentId::Comment(comment_id) => {
        CommentActions::remove_like(pool, person_id, comment_id).await?
      }
    };
  }
  Ok(())
}

/// Dont allow bots to do certain actions, like voting
pub fn check_bot_account(person: &Person) -> LemmyResult<()> {
  if person.bot_account {
    Err(LemmyErrorType::InvalidBotAction)?
  } else {
    Ok(())
  }
}

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

/// If private messages are disabled, dont allow them to be sent / received
pub fn check_private_messages_enabled(local_user_view: &LocalUserView) -> Result<(), LemmyError> {
  if !local_user_view.local_user.enable_private_messages {
    Err(LemmyErrorType::CouldntCreatePrivateMessage)?
  } else {
    Ok(())
  }
}

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

pub async fn slur_regex(context: &LemmyContext) -> LemmyResult<Regex> {
  static CACHE: CacheLock<Regex> = LazyLock::new(|| {
    Cache::builder()
      .max_capacity(1)
      .time_to_live(CACHE_DURATION_FEDERATION)
      .build()
  });
  Ok(
    CACHE
      .try_get_with((), async {
        let local_site = SiteView::read_local(&mut context.pool())
          .await
          .ok()
          .map(|s| s.local_site);
        build_and_check_regex(local_site.and_then(|s| s.slur_filter_regex).as_deref())
      })
      .await
      .map_err(|e| anyhow::anyhow!("Failed to construct regex: {e}"))?,
  )
}

pub async fn get_url_blocklist(context: &LemmyContext) -> LemmyResult<RegexSet> {
  static URL_BLOCKLIST: CacheLock<RegexSet> = LazyLock::new(|| {
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
        // If this regex creation changes it must be synced with
        // lemmy_utils::utils::markdown::create_url_blocklist_test_regex_set.
        let regexes = urls.iter().map(|url| format!(r"\b{}\b", escape(&url.url)));

        let set = RegexSet::new(regexes)?;
        Ok(set)
      })
      .await
      .map_err(|e| anyhow::anyhow!("Failed to build URL blocklist due to `{}`", e))?,
  )
}

pub fn check_nsfw_allowed(nsfw: Option<bool>, local_site: Option<&LocalSite>) -> LemmyResult<()> {
  let is_nsfw = nsfw.unwrap_or_default();
  let nsfw_disallowed = local_site.is_some_and(|s| s.disallow_nsfw_content);

  if nsfw_disallowed && is_nsfw {
    Err(LemmyErrorType::NsfwNotAllowed)?
  }

  Ok(())
}

/// Read the site for an ap_id.
///
/// Used for GetCommunityResponse and GetPersonDetails
pub async fn read_site_for_actor(
  ap_id: DbUrl,
  context: &LemmyContext,
) -> LemmyResult<Option<Site>> {
  let site_id = Site::instance_ap_id_from_url(ap_id.clone().into());
  let site = Site::read_from_apub_id(&mut context.pool(), &site_id.into()).await?;
  Ok(site)
}

pub async fn purge_post_images(
  url: Option<DbUrl>,
  thumbnail_url: Option<DbUrl>,
  context: &LemmyContext,
) {
  if let Some(url) = url {
    purge_image_from_pictrs_url(&url, context).await.ok();
  }
  if let Some(thumbnail_url) = thumbnail_url {
    purge_image_from_pictrs_url(&thumbnail_url, context)
      .await
      .ok();
  }
}

/// Delete a local_user's images
async fn delete_local_user_images(person_id: PersonId, context: &LemmyContext) -> LemmyResult<()> {
  if let Ok(local_user) = LocalUserView::read_person(&mut context.pool(), person_id).await {
    let pictrs_uploads =
      LocalImageView::get_all_by_local_user_id(&mut context.pool(), local_user.local_user.id)
        .await?;

    // Delete their images
    for upload in pictrs_uploads {
      delete_image_from_pictrs(&upload.local_image.pictrs_alias, context)
        .await
        .ok();
    }
  }
  Ok(())
}

/// Removes or restores user data.
pub async fn remove_or_restore_user_data(
  mod_person_id: PersonId,
  banned_person_id: PersonId,
  removed: bool,
  reason: &Option<String>,
  context: &LemmyContext,
) -> LemmyResult<()> {
  let pool = &mut context.pool();

  // These actions are only possible when removing, not restoring
  if removed {
    delete_local_user_images(banned_person_id, context).await?;

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
          removed: Some(removed),
          ..Default::default()
        },
      )
      .await?;

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
  }

  // Posts
  let removed_or_restored_posts =
    Post::update_removed_for_creator(pool, banned_person_id, None, None, removed).await?;
  create_modlog_entries_for_removed_or_restored_posts(
    pool,
    mod_person_id,
    removed_or_restored_posts.iter().map(|r| r.id).collect(),
    removed,
    reason,
  )
  .await?;

  // Comments
  let removed_or_restored_comments =
    Comment::update_removed_for_creator(pool, banned_person_id, removed).await?;
  create_modlog_entries_for_removed_or_restored_comments(
    pool,
    mod_person_id,
    removed_or_restored_comments.iter().map(|r| r.id).collect(),
    removed,
    reason,
  )
  .await?;

  // Private messages
  PrivateMessage::update_removed_for_creator(pool, banned_person_id, removed).await?;

  Ok(())
}

async fn create_modlog_entries_for_removed_or_restored_posts(
  pool: &mut DbPool<'_>,
  mod_person_id: PersonId,
  post_ids: Vec<PostId>,
  removed: bool,
  reason: &Option<String>,
) -> LemmyResult<()> {
  // Build the forms
  let forms = post_ids
    .iter()
    .map(|&post_id| ModRemovePostForm {
      mod_person_id,
      post_id,
      removed: Some(removed),
      reason: reason.clone(),
    })
    .collect();

  ModRemovePost::create_multiple(pool, &forms).await?;

  Ok(())
}

async fn create_modlog_entries_for_removed_or_restored_comments(
  pool: &mut DbPool<'_>,
  mod_person_id: PersonId,
  comment_ids: Vec<CommentId>,
  removed: bool,
  reason: &Option<String>,
) -> LemmyResult<()> {
  // Build the forms
  let forms = comment_ids
    .iter()
    .map(|&comment_id| ModRemoveCommentForm {
      mod_person_id,
      comment_id,
      removed: Some(removed),
      reason: reason.clone(),
    })
    .collect();

  ModRemoveComment::create_multiple(pool, &forms).await?;

  Ok(())
}

pub async fn remove_or_restore_user_data_in_community(
  community_id: CommunityId,
  mod_person_id: PersonId,
  banned_person_id: PersonId,
  remove: bool,
  reason: &Option<String>,
  pool: &mut DbPool<'_>,
) -> LemmyResult<()> {
  // Posts
  let posts =
    Post::update_removed_for_creator(pool, banned_person_id, Some(community_id), None, remove)
      .await?;
  create_modlog_entries_for_removed_or_restored_posts(
    pool,
    mod_person_id,
    posts.iter().map(|r| r.id).collect(),
    remove,
    reason,
  )
  .await?;

  // Comments
  let removed_comment_ids =
    Comment::update_removed_for_creator_and_community(pool, banned_person_id, community_id, remove)
      .await?;

  create_modlog_entries_for_removed_or_restored_comments(
    pool,
    mod_person_id,
    removed_comment_ids,
    remove,
    reason,
  )
  .await?;

  Ok(())
}

pub async fn purge_user_account(person_id: PersonId, context: &LemmyContext) -> LemmyResult<()> {
  let pool = &mut context.pool();

  // Delete their local images, if they're a local user
  // No need to update avatar and banner, those are handled in Person::delete_account
  delete_local_user_images(person_id, context).await.ok();

  // Comments
  Comment::permadelete_for_creator(pool, person_id)
    .await
    .with_lemmy_type(LemmyErrorType::CouldntUpdateComment)?;

  // Posts
  Post::permadelete_for_creator(pool, person_id)
    .await
    .with_lemmy_type(LemmyErrorType::CouldntUpdatePost)?;

  // Leave communities they mod
  CommunityActions::leave_mod_team_for_all_communities(pool, person_id).await?;

  // Delete the oauth accounts linked to the local user
  if let Ok(local_user) = LocalUserView::read_person(pool, person_id).await {
    OAuthAccount::delete_user_accounts(pool, local_user.local_user.id).await?;
  }

  Person::delete_account(pool, person_id).await?;

  Ok(())
}

pub fn generate_followers_url(ap_id: &DbUrl) -> Result<DbUrl, ParseError> {
  Ok(Url::parse(&format!("{ap_id}/followers"))?.into())
}

pub fn generate_inbox_url() -> LemmyResult<DbUrl> {
  let url = format!("{}/inbox", SETTINGS.get_protocol_and_hostname());
  Ok(Url::parse(&url)?.into())
}

pub fn generate_outbox_url(ap_id: &DbUrl) -> Result<DbUrl, ParseError> {
  Ok(Url::parse(&format!("{ap_id}/outbox"))?.into())
}

pub fn generate_featured_url(ap_id: &DbUrl) -> Result<DbUrl, ParseError> {
  Ok(Url::parse(&format!("{ap_id}/featured"))?.into())
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

pub fn check_conflicting_like_filters(
  liked_only: Option<bool>,
  disliked_only: Option<bool>,
) -> LemmyResult<()> {
  if liked_only.unwrap_or_default() && disliked_only.unwrap_or_default() {
    Err(LemmyErrorType::ContradictingFilters)?
  } else {
    Ok(())
  }
}

pub async fn process_markdown(
  text: &str,
  slur_regex: &Regex,
  url_blocklist: &RegexSet,
  context: &LemmyContext,
) -> LemmyResult<String> {
  let text = remove_slurs(text, slur_regex);
  let text = clean_urls_in_text(&text);

  markdown_check_for_blocked_urls(&text, url_blocklist)?;

  if context.settings().pictrs()?.image_mode == PictrsImageMode::ProxyAllImages {
    let (text, links) = markdown_rewrite_image_links(text);
    RemoteImage::create(&mut context.pool(), links.clone()).await?;

    // Create images and image detail rows
    for link in links {
      // Insert image details for the remote image
      let details_res = fetch_pictrs_proxied_image_details(&link, context).await;
      if let Ok(details) = details_res {
        let proxied =
          build_proxied_image_url(&link, &context.settings().get_protocol_and_hostname())?;
        let details_form = details.build_image_details_form(&proxied);
        ImageDetails::create(&mut context.pool(), &details_form).await?;
      }
    }
    Ok(text)
  } else {
    Ok(text)
  }
}

pub async fn process_markdown_opt(
  text: &Option<String>,
  slur_regex: &Regex,
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
    RemoteImage::create(&mut context.pool(), vec![link.clone()]).await?;

    let proxied = build_proxied_image_url(&link, &context.settings().get_protocol_and_hostname())?;
    // This should fail softly, since pictrs might not even be running
    let details_res = fetch_pictrs_proxied_image_details(&link, context).await;

    if let Ok(details) = details_res {
      let details_form = details.build_image_details_form(&proxied);
      ImageDetails::create(&mut context.pool(), &details_form).await?;
    };

    Ok(proxied.into())
  } else {
    Ok(link.into())
  }
}

/// Rewrite a link to go through `/api/v4/image_proxy` endpoint. This is only for remote urls and
/// if image_proxy setting is enabled.
pub async fn proxy_image_link(link: Url, context: &LemmyContext) -> LemmyResult<DbUrl> {
  proxy_image_link_internal(link, context.settings().pictrs()?.image_mode, context).await
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
    "{}/api/v4/image/proxy?url={}",
    protocol_and_hostname,
    encode(link.as_str())
  ))
}

pub async fn local_user_view_from_jwt(
  jwt: &str,
  context: &LemmyContext,
) -> LemmyResult<LocalUserView> {
  let local_user_id = Claims::validate(jwt, context)
    .await
    .with_lemmy_type(LemmyErrorType::NotLoggedIn)?;
  let local_user_view = LocalUserView::read(&mut context.pool(), local_user_id).await?;
  check_local_user_valid(&local_user_view)?;

  Ok(local_user_view)
}

pub fn read_auth_token(req: &HttpRequest) -> LemmyResult<Option<String>> {
  // Try reading jwt from auth header
  if let Ok(header) = Authorization::<Bearer>::parse(req) {
    Ok(Some(header.as_ref().token().to_string()))
  }
  // If that fails, try to read from cookie
  else if let Some(cookie) = &req.cookie(AUTH_COOKIE_NAME) {
    Ok(Some(cookie.value().to_string()))
  }
  // Otherwise, there's no auth
  else {
    Ok(None)
  }
}

pub fn send_webmention(post: Post, community: &Community) {
  if let Some(url) = post.url.clone() {
    if community.visibility.can_view_without_login() {
      spawn_try_task(async move {
        let mut webmention = Webmention::new::<Url>(post.ap_id.clone().into(), url.clone().into())?;
        webmention.set_checked(true);
        match webmention
          .send()
          .instrument(tracing::info_span!("Sending webmention"))
          .await
        {
          Err(WebmentionError::NoEndpointDiscovered(_)) => Ok(()),
          Ok(_) => Ok(()),
          Err(e) => Err(e).with_lemmy_type(LemmyErrorType::CouldntSendWebmention),
        }
      });
    }
  };
}

#[cfg(test)]
mod tests {
  use super::*;
  use lemmy_db_schema::{
    source::{
      comment::CommentInsertForm,
      community::CommunityInsertForm,
      person::PersonInsertForm,
      post::PostInsertForm,
    },
    ModlogActionType,
  };
  use lemmy_db_views_modlog_combined::{
    impls::ModlogCombinedQuery,
    ModRemoveCommentView,
    ModRemovePostView,
    ModlogCombinedView,
  };
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
  fn test_limit_ban_term() -> LemmyResult<()> {
    // Ban expires in past, should throw error
    assert!(limit_expire_time(Utc::now() - Days::new(5)).is_err());

    // Legitimate ban term, return same value
    let fourteen_days = Utc::now() + Days::new(14);
    assert_eq!(limit_expire_time(fourteen_days)?, Some(fourteen_days));
    let nine_years = Utc::now() + Days::new(365 * 9);
    assert_eq!(limit_expire_time(nine_years)?, Some(nine_years));

    // Too long ban term, changes to None (permanent ban)
    assert_eq!(limit_expire_time(Utc::now() + Days::new(365 * 11))?, None);

    Ok(())
  }

  #[tokio::test]
  #[serial]
  async fn test_proxy_image_link() -> LemmyResult<()> {
    let context = LemmyContext::init_test_context().await;

    // image from local domain is unchanged
    let local_url = Url::parse("http://lemmy-alpha/image.png")?;
    let proxied =
      proxy_image_link_internal(local_url.clone(), PictrsImageMode::ProxyAllImages, &context)
        .await?;
    assert_eq!(&local_url, proxied.inner());

    // image from remote domain is proxied
    let remote_image = Url::parse("http://lemmy-beta/image.png")?;
    let proxied = proxy_image_link_internal(
      remote_image.clone(),
      PictrsImageMode::ProxyAllImages,
      &context,
    )
    .await?;
    assert_eq!(
      "https://lemmy-alpha/api/v4/image/proxy?url=http%3A%2F%2Flemmy-beta%2Fimage.png",
      proxied.as_str()
    );

    // This fails, because the details can't be fetched without pictrs running,
    // And a remote image won't be inserted.
    assert!(
      RemoteImage::validate(&mut context.pool(), remote_image.into())
        .await
        .is_ok()
    );

    Ok(())
  }

  #[tokio::test]
  #[serial]
  async fn test_mod_remove_or_restore_data() -> LemmyResult<()> {
    let context = LemmyContext::init_test_context().await;
    let pool = &mut context.pool();

    let inserted_instance = Instance::read_or_create(pool, "my_domain.tld".to_string()).await?;

    let new_mod = PersonInsertForm::test_form(inserted_instance.id, "modder");
    let inserted_mod = Person::create(pool, &new_mod).await?;

    let new_person = PersonInsertForm::test_form(inserted_instance.id, "chrimbus");
    let inserted_person = Person::create(pool, &new_person).await?;

    let new_community = CommunityInsertForm::new(
      inserted_instance.id,
      "mod_community crepes".to_string(),
      "nada".to_owned(),
      "pubkey".to_string(),
    );
    let inserted_community = Community::create(pool, &new_community).await?;

    let post_form_1 = PostInsertForm::new(
      "A test post tubular".into(),
      inserted_person.id,
      inserted_community.id,
    );
    let inserted_post_1 = Post::create(pool, &post_form_1).await?;

    let post_form_2 = PostInsertForm::new(
      "A test post radical".into(),
      inserted_person.id,
      inserted_community.id,
    );
    let inserted_post_2 = Post::create(pool, &post_form_2).await?;

    let comment_form_1 = CommentInsertForm::new(
      inserted_person.id,
      inserted_post_1.id,
      "A test comment tubular".into(),
    );
    let _inserted_comment_1 = Comment::create(pool, &comment_form_1, None).await?;

    let comment_form_2 = CommentInsertForm::new(
      inserted_person.id,
      inserted_post_2.id,
      "A test comment radical".into(),
    );
    let _inserted_comment_2 = Comment::create(pool, &comment_form_2, None).await?;

    // Remove the user data
    remove_or_restore_user_data(
      inserted_mod.id,
      inserted_person.id,
      true,
      &Some("a remove reason".to_string()),
      &context,
    )
    .await?;

    // Verify that their posts and comments are removed.
    // Posts
    let post_modlog = ModlogCombinedQuery {
      type_: Some(ModlogActionType::ModRemovePost),
      ..Default::default()
    }
    .list(pool)
    .await?;
    assert_eq!(2, post_modlog.len());

    assert!(matches!(
      &post_modlog[..],
      [
        ModlogCombinedView::ModRemovePost(ModRemovePostView {
          mod_remove_post: ModRemovePost { removed: true, .. },
          post: Post { removed: true, .. },
          ..
        }),
        ModlogCombinedView::ModRemovePost(ModRemovePostView {
          mod_remove_post: ModRemovePost { removed: true, .. },
          post: Post { removed: true, .. },
          ..
        }),
      ],
    ));

    // Comments
    let comment_modlog = ModlogCombinedQuery {
      type_: Some(ModlogActionType::ModRemoveComment),
      ..Default::default()
    }
    .list(pool)
    .await?;
    assert_eq!(2, comment_modlog.len());

    assert!(matches!(
      &comment_modlog[..],
      [
        ModlogCombinedView::ModRemoveComment(ModRemoveCommentView {
          mod_remove_comment: ModRemoveComment { removed: true, .. },
          comment: Comment { removed: true, .. },
          ..
        }),
        ModlogCombinedView::ModRemoveComment(ModRemoveCommentView {
          mod_remove_comment: ModRemoveComment { removed: true, .. },
          comment: Comment { removed: true, .. },
          ..
        }),
      ],
    ));

    // Now restore the content, and make sure it got appended
    remove_or_restore_user_data(
      inserted_mod.id,
      inserted_person.id,
      false,
      &Some("a restore reason".to_string()),
      &context,
    )
    .await?;

    // Posts
    let post_modlog = ModlogCombinedQuery {
      type_: Some(ModlogActionType::ModRemovePost),
      ..Default::default()
    }
    .list(pool)
    .await?;
    assert_eq!(4, post_modlog.len());

    assert!(matches!(
      &post_modlog[..],
      [
        ModlogCombinedView::ModRemovePost(ModRemovePostView {
          mod_remove_post: ModRemovePost { removed: false, .. },
          post: Post { removed: false, .. },
          ..
        }),
        ModlogCombinedView::ModRemovePost(ModRemovePostView {
          mod_remove_post: ModRemovePost { removed: false, .. },
          post: Post { removed: false, .. },
          ..
        }),
        ModlogCombinedView::ModRemovePost(ModRemovePostView {
          mod_remove_post: ModRemovePost { removed: true, .. },
          post: Post { removed: false, .. },
          ..
        }),
        ModlogCombinedView::ModRemovePost(ModRemovePostView {
          mod_remove_post: ModRemovePost { removed: true, .. },
          post: Post { removed: false, .. },
          ..
        }),
      ],
    ));

    // Comments
    let comment_modlog = ModlogCombinedQuery {
      type_: Some(ModlogActionType::ModRemoveComment),
      ..Default::default()
    }
    .list(pool)
    .await?;
    assert_eq!(4, comment_modlog.len());

    assert!(matches!(
      &comment_modlog[..],
      [
        ModlogCombinedView::ModRemoveComment(ModRemoveCommentView {
          mod_remove_comment: ModRemoveComment { removed: false, .. },
          comment: Comment { removed: false, .. },
          ..
        }),
        ModlogCombinedView::ModRemoveComment(ModRemoveCommentView {
          mod_remove_comment: ModRemoveComment { removed: false, .. },
          comment: Comment { removed: false, .. },
          ..
        }),
        ModlogCombinedView::ModRemoveComment(ModRemoveCommentView {
          mod_remove_comment: ModRemoveComment { removed: true, .. },
          comment: Comment { removed: false, .. },
          ..
        }),
        ModlogCombinedView::ModRemoveComment(ModRemoveCommentView {
          mod_remove_comment: ModRemoveComment { removed: true, .. },
          comment: Comment { removed: false, .. },
          ..
        }),
      ],
    ));

    Instance::delete(pool, inserted_instance.id).await?;

    Ok(())
  }
}
