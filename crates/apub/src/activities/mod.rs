use crate::{
  generate_moderators_url,
  insert_activity,
  local_instance,
  objects::{community::ApubCommunity, person::ApubPerson},
  ActorType,
  CONTEXT,
};
use activitypub_federation::{
  core::{activity_queue::send_activity, object_id::ObjectId},
  deser::context::WithContext,
  traits::{ActivityHandler, Actor},
};
use activitystreams_kinds::public;
use anyhow::anyhow;
use lemmy_api_common::utils::blocking;
use lemmy_db_schema::source::community::Community;
use lemmy_db_views_actor::structs::{CommunityPersonBanView, CommunityView};
use lemmy_utils::error::LemmyError;
use lemmy_websocket::LemmyContext;
use serde::Serialize;
use std::ops::Deref;
use tracing::info;
use url::{ParseError, Url};
use uuid::Uuid;

pub mod block;
pub mod community;
pub mod create_or_update;
pub mod deletion;
pub mod following;
pub mod voting;

/// Checks that the specified Url actually identifies a Person (by fetching it), and that the person
/// doesn't have a site ban.
#[tracing::instrument(skip_all)]
async fn verify_person(
  person_id: &ObjectId<ApubPerson>,
  context: &LemmyContext,
  request_counter: &mut i32,
) -> Result<(), LemmyError> {
  let person = person_id
    .dereference(context, local_instance(context), request_counter)
    .await?;
  if person.banned {
    let err = anyhow!("Person {} is banned", person_id);
    return Err(LemmyError::from_error_message(err, "banned"));
  }
  Ok(())
}

/// Fetches the person and community to verify their type, then checks if person is banned from site
/// or community.
#[tracing::instrument(skip_all)]
pub(crate) async fn verify_person_in_community(
  person_id: &ObjectId<ApubPerson>,
  community: &ApubCommunity,
  context: &LemmyContext,
  request_counter: &mut i32,
) -> Result<(), LemmyError> {
  let person = person_id
    .dereference(context, local_instance(context), request_counter)
    .await?;
  if person.banned {
    return Err(LemmyError::from_message("Person is banned from site"));
  }
  let person_id = person.id;
  let community_id = community.id;
  let is_banned =
    move |conn: &'_ _| CommunityPersonBanView::get(conn, person_id, community_id).is_ok();
  if blocking(context.pool(), is_banned).await? {
    return Err(LemmyError::from_message("Person is banned from community"));
  }

  Ok(())
}

/// Verify that the actor is a community mod. This check is only run if the community is local,
/// because in case of remote communities, admins can also perform mod actions. As admin status
/// is not federated, we cant verify their actions remotely.
///
/// * `mod_id` - Activitypub ID of the mod or admin who performed the action
/// * `object_id` - Activitypub ID of the actor or object that is being moderated
/// * `community` - The community inside which moderation is happening
#[tracing::instrument(skip_all)]
pub(crate) async fn verify_mod_action(
  mod_id: &ObjectId<ApubPerson>,
  object_id: &Url,
  community: &ApubCommunity,
  context: &LemmyContext,
  request_counter: &mut i32,
) -> Result<(), LemmyError> {
  if community.local {
    let actor = mod_id
      .dereference(context, local_instance(context), request_counter)
      .await?;

    // Note: this will also return true for admins in addition to mods, but as we dont know about
    //       remote admins, it doesnt make any difference.
    let community_id = community.id;
    let actor_id = actor.id;

    let is_mod_or_admin = blocking(context.pool(), move |conn| {
      CommunityView::is_mod_or_admin(conn, actor_id, community_id)
    })
    .await?;

    // mod action was done either by a community mod or a local admin, so its allowed
    if is_mod_or_admin {
      return Ok(());
    }

    // mod action comes from the same instance as the moderated object, so it was presumably done
    // by an instance admin and is legitimate (admin status is not federated).
    if mod_id.inner().domain() == object_id.domain() {
      return Ok(());
    }

    // the user is not a valid mod
    return Err(LemmyError::from_message("Not a mod"));
  }
  Ok(())
}

/// For Add/Remove community moderator activities, check that the target field actually contains
/// /c/community/moderators. Any different values are unsupported.
fn verify_add_remove_moderator_target(
  target: &Url,
  community: &ApubCommunity,
) -> Result<(), LemmyError> {
  if target != &generate_moderators_url(&community.actor_id)?.into() {
    return Err(LemmyError::from_message("Unkown target url"));
  }
  Ok(())
}

pub(crate) fn verify_is_public(to: &[Url], cc: &[Url]) -> Result<(), LemmyError> {
  if ![to, cc].iter().any(|set| set.contains(&public())) {
    return Err(LemmyError::from_message("Object is not public"));
  }
  Ok(())
}

pub(crate) fn check_community_deleted_or_removed(community: &Community) -> Result<(), LemmyError> {
  if community.deleted || community.removed {
    Err(LemmyError::from_message(
      "New post or comment cannot be created in deleted or removed community",
    ))
  } else {
    Ok(())
  }
}

/// Generate a unique ID for an activity, in the format:
/// `http(s)://example.com/receive/create/202daf0a-1489-45df-8d2e-c8a3173fed36`
fn generate_activity_id<T>(kind: T, protocol_and_hostname: &str) -> Result<Url, ParseError>
where
  T: ToString,
{
  let id = format!(
    "{}/activities/{}/{}",
    protocol_and_hostname,
    kind.to_string().to_lowercase(),
    Uuid::new_v4()
  );
  Url::parse(&id)
}

#[tracing::instrument(skip_all)]
async fn send_lemmy_activity<Activity, ActorT>(
  context: &LemmyContext,
  activity: Activity,
  actor: &ActorT,
  inbox: Vec<Url>,
  sensitive: bool,
) -> Result<(), LemmyError>
where
  Activity: ActivityHandler + Serialize,
  ActorT: Actor + ActorType,
  Activity: ActivityHandler<Error = LemmyError>,
{
  info!("Sending activity {}", activity.id().to_string());
  let activity = WithContext::new(activity, CONTEXT.deref().clone());

  let object_value = serde_json::to_value(&activity)?;
  insert_activity(activity.id(), object_value, true, sensitive, context.pool()).await?;

  send_activity(
    activity,
    actor.get_public_key(),
    actor.private_key().expect("actor has private key"),
    inbox,
    local_instance(context),
  )
  .await?;

  Ok(())
}
