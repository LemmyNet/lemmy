use crate::{
  check_community_or_site_ban,
  check_is_apub_id_valid,
  fetcher::{community::get_or_fetch_and_upsert_community, person::get_or_fetch_and_upsert_person},
  generate_moderators_url,
};
use anyhow::anyhow;
use lemmy_api_common::blocking;
use lemmy_apub_lib::{verify_domains_match, ActivityFields};
use lemmy_db_queries::ApubObject;
use lemmy_db_schema::{
  source::{community::Community, person::Person},
  DbUrl,
};
use lemmy_db_views_actor::community_view::CommunityView;
use lemmy_utils::{settings::structs::Settings, LemmyError};
use lemmy_websocket::LemmyContext;
use serde::{Deserialize, Serialize};
use strum_macros::ToString;
use url::{ParseError, Url};
use uuid::Uuid;

pub mod comment;
pub mod community;
pub mod deletion;
pub mod following;
pub mod post;
pub mod private_message;
pub mod send;
pub mod undo_remove;
pub mod voting;

#[derive(Clone, Debug, ToString, Deserialize, Serialize)]
pub enum CreateOrUpdateType {
  Create,
  Update,
}

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

pub(crate) async fn extract_community(
  cc: &[Url],
  context: &LemmyContext,
  request_counter: &mut i32,
) -> Result<Community, LemmyError> {
  let mut cc_iter = cc.iter();
  loop {
    if let Some(cid) = cc_iter.next() {
      if let Ok(c) = get_or_fetch_and_upsert_community(cid, context, request_counter).await {
        break Ok(c);
      }
    } else {
      return Err(anyhow!("No community found in cc").into());
    }
  }
}

/// Fetches the person and community to verify their type, then checks if person is banned from site
/// or community.
pub(crate) async fn verify_person_in_community(
  person_id: &Url,
  community_id: &Url,
  context: &LemmyContext,
  request_counter: &mut i32,
) -> Result<(), LemmyError> {
  let community = get_or_fetch_and_upsert_community(community_id, context, request_counter).await?;
  let person = get_or_fetch_and_upsert_person(person_id, context, request_counter).await?;
  check_community_or_site_ban(&person, community.id, context.pool()).await
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

fn verify_activity(activity: &dyn ActivityFields) -> Result<(), LemmyError> {
  check_is_apub_id_valid(activity.actor(), false)?;
  verify_domains_match(activity.id_unchecked(), activity.actor())?;
  Ok(())
}

/// Verify that the actor is a community mod. This check is only run if the community is local,
/// because in case of remote communities, admins can also perform mod actions. As admin status
/// is not federated, we cant verify their actions remotely.
pub(crate) async fn verify_mod_action(
  actor_id: &Url,
  community: Url,
  context: &LemmyContext,
) -> Result<(), LemmyError> {
  let community = blocking(context.pool(), move |conn| {
    Community::read_from_apub_id(conn, &community.into())
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

/// Generate a unique ID for an activity, in the format:
/// `http(s)://example.com/receive/create/202daf0a-1489-45df-8d2e-c8a3173fed36`
fn generate_activity_id<T>(kind: T) -> Result<Url, ParseError>
where
  T: ToString,
{
  let id = format!(
    "{}/activities/{}/{}",
    Settings::get().get_protocol_and_hostname(),
    kind.to_string().to_lowercase(),
    Uuid::new_v4()
  );
  Url::parse(&id)
}
