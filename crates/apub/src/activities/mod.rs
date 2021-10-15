use crate::{
  check_community_or_site_ban,
  check_is_apub_id_valid,
  fetcher::object_id::ObjectId,
  generate_moderators_url,
};
use anyhow::anyhow;
use lemmy_api_common::blocking;
use lemmy_apub_lib::{traits::ActivityFields, verify::verify_domains_match};
use lemmy_db_schema::source::{community::Community, person::Person};
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
pub mod report;
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
  person_id: &ObjectId<Person>,
  context: &LemmyContext,
  request_counter: &mut i32,
) -> Result<(), LemmyError> {
  let person = person_id.dereference(context, request_counter).await?;
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
      let cid = ObjectId::new(cid.clone());
      if let Ok(c) = cid.dereference(context, request_counter).await {
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
  person_id: &ObjectId<Person>,
  community_id: &ObjectId<Community>,
  context: &LemmyContext,
  request_counter: &mut i32,
) -> Result<(), LemmyError> {
  let community = community_id.dereference(context, request_counter).await?;
  let person = person_id.dereference(context, request_counter).await?;
  check_community_or_site_ban(&person, community.id, context.pool()).await
}

/// Simply check that the url actually refers to a valid group.
async fn verify_community(
  community_id: &ObjectId<Community>,
  context: &LemmyContext,
  request_counter: &mut i32,
) -> Result<(), LemmyError> {
  community_id.dereference(context, request_counter).await?;
  Ok(())
}

fn verify_activity(activity: &dyn ActivityFields, settings: &Settings) -> Result<(), LemmyError> {
  check_is_apub_id_valid(activity.actor(), false, settings)?;
  verify_domains_match(activity.id_unchecked(), activity.actor())?;
  Ok(())
}

/// Verify that the actor is a community mod. This check is only run if the community is local,
/// because in case of remote communities, admins can also perform mod actions. As admin status
/// is not federated, we cant verify their actions remotely.
pub(crate) async fn verify_mod_action(
  actor_id: &ObjectId<Person>,
  community_id: ObjectId<Community>,
  context: &LemmyContext,
  request_counter: &mut i32,
) -> Result<(), LemmyError> {
  let community = community_id.dereference_local(context).await?;

  if community.local {
    let actor = actor_id.dereference(context, request_counter).await?;

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
fn verify_add_remove_moderator_target(
  target: &Url,
  community: &ObjectId<Community>,
) -> Result<(), LemmyError> {
  if target != &generate_moderators_url(&community.clone().into())?.into_inner() {
    return Err(anyhow!("Unkown target url").into());
  }
  Ok(())
}

pub(crate) fn check_community_deleted_or_removed(community: &Community) -> Result<(), LemmyError> {
  if community.deleted || community.removed {
    Err(anyhow!("New post or comment cannot be created in deleted or removed community").into())
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
