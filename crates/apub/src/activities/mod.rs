use crate::{
  check_community_or_site_ban,
  check_is_apub_id_valid,
  fetcher::{community::get_or_fetch_and_upsert_community, person::get_or_fetch_and_upsert_person},
  generate_moderators_url,
};
use anyhow::anyhow;
use lemmy_api_common::blocking;
use lemmy_apub_lib::{verify_domains_match, ActivityCommonFields};
use lemmy_db_queries::ApubObject;
use lemmy_db_schema::{
  source::{community::Community, person::Person},
  DbUrl,
};
use lemmy_db_views_actor::community_view::CommunityView;
use lemmy_utils::LemmyError;
use lemmy_websocket::LemmyContext;
use url::Url;

pub mod comment;
pub mod community;
pub mod deletion;
pub mod following;
pub mod post;
pub mod private_message;
pub mod removal;
pub mod send;
pub mod voting;

/// Checks that the specified Url actually identifies a Person (by fetching it), and that the person
/// doesn't have a site ban.
async fn verify_person(
  person_id: &Url,
  context: &LemmyContext,
  request_counter: &mut i32,
) -> Result<(), LemmyError> {
  let person = get_or_fetch_and_upsert_person(person_id, context, request_counter).await?;
  if person.banned {
    return Err(anyhow!("Person {} is banned", person_id).into());
  }
  Ok(())
}

/// Fetches the person and community to verify their type, then checks if person is banned from site
/// or community.
async fn verify_person_in_community(
  person_id: &Url,
  cc: &[Url],
  context: &LemmyContext,
  request_counter: &mut i32,
) -> Result<Community, LemmyError> {
  let person = get_or_fetch_and_upsert_person(person_id, context, request_counter).await?;
  let mut cc_iter = cc.iter();
  let community: Community = loop {
    if let Some(cid) = cc_iter.next() {
      if let Ok(c) = get_or_fetch_and_upsert_community(cid, context, request_counter).await {
        break c;
      }
    } else {
      return Err(anyhow!("No community found in cc").into());
    }
  };
  check_community_or_site_ban(&person, community.id, context.pool()).await?;
  Ok(community)
}

/// Simply check that the url actually refers to a valid group.
async fn verify_community(
  community_id: &Url,
  context: &LemmyContext,
  request_counter: &mut i32,
) -> Result<(), LemmyError> {
  get_or_fetch_and_upsert_community(community_id, context, request_counter).await?;
  Ok(())
}

fn verify_activity(common: &ActivityCommonFields) -> Result<(), LemmyError> {
  check_is_apub_id_valid(&common.actor, false)?;
  verify_domains_match(common.id_unchecked(), &common.actor)?;
  Ok(())
}

async fn verify_mod_action(
  actor_id: &Url,
  activity_cc: Url,
  context: &LemmyContext,
) -> Result<(), LemmyError> {
  let community = blocking(context.pool(), move |conn| {
    Community::read_from_apub_id(conn, &activity_cc.into())
  })
  .await??;

  if community.local {
    let actor_id: DbUrl = actor_id.clone().into();
    let actor = blocking(context.pool(), move |conn| {
      Person::read_from_apub_id(conn, &actor_id)
    })
    .await??;

    // Note: this will also return true for admins in addition to mods, but as we dont know about
    //       remote admins, it doesnt make any difference.
    let community_id = community.id;
    let actor_id = actor.id;
    let is_mod_or_admin = blocking(context.pool(), move |conn| {
      CommunityView::is_mod_or_admin(conn, actor_id, community_id)
    })
    .await?;
    if !is_mod_or_admin {
      return Err(anyhow!("Not a mod").into());
    }
  }
  Ok(())
}

/// For Add/Remove community moderator activities, check that the target field actually contains
/// /c/community/moderators. Any different values are unsupported.
fn verify_add_remove_moderator_target(target: &Url, community: Url) -> Result<(), LemmyError> {
  if target != &generate_moderators_url(&community.into())?.into_inner() {
    return Err(anyhow!("Unkown target url").into());
  }
  Ok(())
}
