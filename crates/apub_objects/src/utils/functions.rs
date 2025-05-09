use super::protocol::Source;
use crate::{
  objects::{community::ApubCommunity, person::ApubPerson},
  protocol::page::Attachment,
};
use activitypub_federation::{
  config::Data,
  fetch::object_id::ObjectId,
  kinds::public,
  protocol::values::MediaTypeMarkdownOrHtml,
};
use html2md::parse_html;
use lemmy_api_common::context::LemmyContext;
use lemmy_db_schema::{
  source::{
    community::{Community, CommunityActions, CommunityModeratorForm},
    instance::{Instance, InstanceActions},
    local_site::LocalSite,
  },
  traits::Joinable,
  utils::DbPool,
};
use lemmy_db_schema_file::enums::{ActorType, CommunityVisibility};
use lemmy_db_views_community_moderator::CommunityModeratorView;
use lemmy_db_views_community_person_ban::CommunityPersonBanView;
use lemmy_db_views_site::SiteView;
use lemmy_utils::{
  error::{FederationError, LemmyError, LemmyResult},
  CacheLock,
  CACHE_DURATION_FEDERATION,
};
use moka::future::Cache;
use std::sync::{Arc, LazyLock};
use url::Url;

pub fn read_from_string_or_source(
  content: &str,
  media_type: &Option<MediaTypeMarkdownOrHtml>,
  source: &Option<Source>,
) -> String {
  if let Some(s) = source {
    // markdown sent by lemmy in source field
    s.content.clone()
  } else if media_type == &Some(MediaTypeMarkdownOrHtml::Markdown) {
    // markdown sent by peertube in content field
    content.to_string()
  } else {
    // otherwise, convert content html to markdown
    parse_html(content)
  }
}

pub fn read_from_string_or_source_opt(
  content: &Option<String>,
  media_type: &Option<MediaTypeMarkdownOrHtml>,
  source: &Option<Source>,
) -> Option<String> {
  content
    .as_ref()
    .map(|content| read_from_string_or_source(content, media_type, source))
}

#[derive(Clone)]
pub struct LocalSiteData {
  local_site: Option<LocalSite>,
  allowed_instances: Vec<Instance>,
  blocked_instances: Vec<Instance>,
}

pub async fn local_site_data_cached(pool: &mut DbPool<'_>) -> LemmyResult<Arc<LocalSiteData>> {
  // All incoming and outgoing federation actions read the blocklist/allowlist and slur filters
  // multiple times. This causes a huge number of database reads if we hit the db directly. So we
  // cache these values for a short time, which will already make a huge difference and ensures that
  // changes take effect quickly.
  static CACHE: CacheLock<Arc<LocalSiteData>> = LazyLock::new(|| {
    Cache::builder()
      .max_capacity(1)
      .time_to_live(CACHE_DURATION_FEDERATION)
      .build()
  });
  Ok(
    CACHE
      .try_get_with((), async {
        let (local_site, allowed_instances, blocked_instances) =
          lemmy_db_schema::try_join_with_pool!(pool => (
            // LocalSite may be missing
            |pool| async {
              Ok(SiteView::read_local(pool).await.ok().map(|s| s.local_site))
            },
            Instance::allowlist,
            Instance::blocklist
          ))?;

        Ok::<_, LemmyError>(Arc::new(LocalSiteData {
          local_site,
          allowed_instances,
          blocked_instances,
        }))
      })
      .await.map_err(|e| anyhow::anyhow!("err getting activity: {e:?}"))?
  )
}

pub async fn check_apub_id_valid_with_strictness(
  apub_id: &Url,
  is_strict: bool,
  context: &LemmyContext,
) -> LemmyResult<()> {
  let domain = apub_id
    .domain()
    .ok_or(FederationError::UrlWithoutDomain)?
    .to_string();
  let local_instance = context.settings().get_hostname_without_port()?;
  if domain == local_instance {
    return Ok(());
  }

  let local_site_data = local_site_data_cached(&mut context.pool()).await?;
  check_apub_id_valid(apub_id, &local_site_data)?;

  // Only check allowlist if this is a community, and there are instances in the allowlist
  if is_strict && !local_site_data.allowed_instances.is_empty() {
    // need to allow this explicitly because apub receive might contain objects from our local
    // instance.
    let mut allowed_and_local = local_site_data
      .allowed_instances
      .iter()
      .map(|i| i.domain.clone())
      .collect::<Vec<String>>();
    let local_instance = context.settings().get_hostname_without_port()?;
    allowed_and_local.push(local_instance);

    let domain = apub_id
      .domain()
      .ok_or(FederationError::UrlWithoutDomain)?
      .to_string();
    if !allowed_and_local.contains(&domain) {
      Err(FederationError::FederationDisabledByStrictAllowList)?
    }
  }
  Ok(())
}

/// Checks if the ID is allowed for sending or receiving.
///
/// In particular, it checks for:
/// - federation being enabled (if its disabled, only local URLs are allowed)
/// - the correct scheme (either http or https)
/// - URL being in the allowlist (if it is active)
/// - URL not being in the blocklist (if it is active)
pub fn check_apub_id_valid(apub_id: &Url, local_site_data: &LocalSiteData) -> LemmyResult<()> {
  let domain = apub_id
    .domain()
    .ok_or(FederationError::UrlWithoutDomain)?
    .to_string();

  if !local_site_data
    .local_site
    .as_ref()
    .map(|l| l.federation_enabled)
    .unwrap_or(true)
  {
    Err(FederationError::FederationDisabled)?
  }

  if local_site_data
    .blocked_instances
    .iter()
    .any(|i| domain.to_lowercase().eq(&i.domain.to_lowercase()))
  {
    Err(FederationError::DomainBlocked(domain.clone()))?
  }

  // Only check this if there are instances in the allowlist
  if !local_site_data.allowed_instances.is_empty()
    && !local_site_data
      .allowed_instances
      .iter()
      .any(|i| domain.to_lowercase().eq(&i.domain.to_lowercase()))
  {
    Err(FederationError::DomainNotInAllowList(domain))?
  }

  Ok(())
}

pub trait GetActorType {
  fn actor_type(&self) -> ActorType;
}

pub async fn handle_community_moderators(
  new_mods: &Vec<ObjectId<ApubPerson>>,
  community: &ApubCommunity,
  context: &Data<LemmyContext>,
) -> LemmyResult<()> {
  let community_id = community.id;
  let current_moderators =
    CommunityModeratorView::for_community(&mut context.pool(), community_id).await?;
  // Remove old mods from database which arent in the moderators collection anymore
  for mod_user in &current_moderators {
    let mod_id = ObjectId::from(mod_user.moderator.ap_id.clone());
    if !new_mods.contains(&mod_id) {
      let community_moderator_form =
        CommunityModeratorForm::new(mod_user.community.id, mod_user.moderator.id);
      CommunityActions::leave(&mut context.pool(), &community_moderator_form).await?;
    }
  }

  // Add new mods to database which have been added to moderators collection
  for mod_id in new_mods {
    // Ignore errors as mod accounts might be deleted or instances unavailable.
    let mod_user: Option<ApubPerson> = mod_id.dereference(context).await.ok();
    if let Some(mod_user) = mod_user {
      if !current_moderators
        .iter()
        .any(|x| x.moderator.ap_id == mod_user.ap_id)
      {
        let community_moderator_form = CommunityModeratorForm::new(community.id, mod_user.id);
        CommunityActions::join(&mut context.pool(), &community_moderator_form).await?;
      }
    }
  }
  Ok(())
}

/// Marks object as public only if the community is public
pub fn generate_to(community: &Community) -> LemmyResult<Vec<Url>> {
  let ap_id = community.ap_id.clone().into();
  if community.visibility == CommunityVisibility::Public {
    Ok(vec![ap_id, public()])
  } else {
    Ok(vec![
      ap_id.clone(),
      Url::parse(&format!("{}/followers", ap_id))?,
    ])
  }
}

/// Fetches the person and community to verify their type, then checks if person is banned from site
/// or community.
pub async fn verify_person_in_community(
  person_id: &ObjectId<ApubPerson>,
  community: &ApubCommunity,
  context: &Data<LemmyContext>,
) -> LemmyResult<()> {
  let person = person_id.dereference(context).await?;
  InstanceActions::check_ban(&mut context.pool(), person.id, person.instance_id).await?;
  let person_id = person.id;
  let community_id = community.id;
  CommunityPersonBanView::check(&mut context.pool(), person_id, community_id).await
}

pub fn verify_is_public(to: &[Url], cc: &[Url]) -> LemmyResult<()> {
  if ![to, cc].iter().any(|set| set.contains(&public())) {
    Err(FederationError::ObjectIsNotPublic)?
  } else {
    Ok(())
  }
}

/// Returns an error if object visibility doesnt match community visibility
/// (ie content in private community must also be private).
pub fn verify_visibility(to: &[Url], cc: &[Url], community: &ApubCommunity) -> LemmyResult<()> {
  use CommunityVisibility::*;
  let object_is_public = [to, cc].iter().any(|set| set.contains(&public()));
  match community.visibility {
    Public | Unlisted if !object_is_public => Err(FederationError::ObjectIsNotPublic)?,
    Private if object_is_public => Err(FederationError::ObjectIsNotPrivate)?,
    _ => Ok(()),
  }
}

pub async fn append_attachments_to_comment(
  content: String,
  attachments: &[Attachment],
  context: &Data<LemmyContext>,
) -> LemmyResult<String> {
  let mut content = content;
  // Don't modify comments with no attachments
  if !attachments.is_empty() {
    content += "\n";
    for attachment in attachments {
      content = content + "\n" + &attachment.as_markdown(context).await?;
    }
  }

  Ok(content)
}
