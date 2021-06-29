use crate::{
  activities::receive::verify_activity_domains_valid,
  inbox::verify_is_addressed_to_public,
};
use activitystreams::{
  activity::{ActorAndObjectRef, Add, Announce, OptTargetRef},
  base::AnyBase,
  object::AsObject,
  prelude::*,
};
use anyhow::{anyhow, Context};
use lemmy_api_common::blocking;
use lemmy_apub::{
  fetcher::person::get_or_fetch_and_upsert_person,
  generate_moderators_url,
  CommunityType,
};
use lemmy_db_queries::{source::community::CommunityModerator_, ApubObject, Joinable};
use lemmy_db_schema::{
  source::{
    community::{Community, CommunityModerator, CommunityModeratorForm},
    person::Person,
  },
  DbUrl,
};
use lemmy_db_views_actor::community_view::CommunityView;
use lemmy_utils::{location_info, LemmyError};
use lemmy_websocket::LemmyContext;
use strum_macros::EnumString;

#[derive(EnumString)]
enum PageOrNote {
  Page,
  Note,
}

#[derive(EnumString)]
enum ObjectTypes {
  Page,
  Note,
  Group,
  Person,
}

#[derive(EnumString)]
enum UndoableActivities {
  Delete,
  Remove,
  Like,
  Dislike,
  Block,
}

/// Add a new mod to the community (can only be done by an existing mod).
pub(in crate::inbox) async fn receive_add_for_community(
  context: &LemmyContext,
  add_any_base: AnyBase,
  announce: Option<Announce>,
  request_counter: &mut i32,
) -> Result<(), LemmyError> {
  let add = Add::from_any_base(add_any_base.to_owned())?.context(location_info!())?;
  let community = extract_community_from_cc(&add, context).await?;

  verify_mod_activity(&add, announce, &community, context).await?;
  verify_is_addressed_to_public(&add)?;
  verify_add_remove_moderator_target(&add, &community)?;

  let new_mod = add
    .object()
    .as_single_xsd_any_uri()
    .context(location_info!())?;
  let new_mod = get_or_fetch_and_upsert_person(&new_mod, context, request_counter).await?;

  // If we had to refetch the community while parsing the activity, then the new mod has already
  // been added. Skip it here as it would result in a duplicate key error.
  let new_mod_id = new_mod.id;
  let moderated_communities = blocking(context.pool(), move |conn| {
    CommunityModerator::get_person_moderated_communities(conn, new_mod_id)
  })
  .await??;
  if !moderated_communities.contains(&community.id) {
    let form = CommunityModeratorForm {
      community_id: community.id,
      person_id: new_mod.id,
    };
    blocking(context.pool(), move |conn| {
      CommunityModerator::join(conn, &form)
    })
    .await??;
  }
  if community.local {
    community
      .send_announce(
        add_any_base,
        add.object().clone().single_xsd_any_uri(),
        context,
      )
      .await?;
  }
  // TODO: send websocket notification about added mod
  Ok(())
}

/// Searches the activity's cc field for a Community ID, and returns the community.
async fn extract_community_from_cc<T, Kind>(
  activity: &T,
  context: &LemmyContext,
) -> Result<Community, LemmyError>
where
  T: AsObject<Kind>,
{
  let cc = activity
    .cc()
    .map(|c| c.as_many())
    .flatten()
    .context(location_info!())?;
  let community_id = cc
    .first()
    .map(|c| c.as_xsd_any_uri())
    .flatten()
    .context(location_info!())?;
  let community_id: DbUrl = community_id.to_owned().into();
  let community = blocking(&context.pool(), move |conn| {
    Community::read_from_apub_id(&conn, &community_id)
  })
  .await??;
  Ok(community)
}

/// Checks that a moderation activity was sent by a user who is listed as mod for the community.
/// This is only used in the case of remote mods, as local mod actions don't go through the
/// community inbox.
///
/// This method should only be used for activities received by the community, not for activities
/// used by community followers.
pub(crate) async fn verify_actor_is_community_mod<T, Kind>(
  activity: &T,
  community: &Community,
  context: &LemmyContext,
) -> Result<(), LemmyError>
where
  T: ActorAndObjectRef + BaseExt<Kind>,
{
  let actor = activity
    .actor()?
    .as_single_xsd_any_uri()
    .context(location_info!())?
    .to_owned();
  let actor = blocking(&context.pool(), move |conn| {
    Person::read_from_apub_id(&conn, &actor.into())
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

  Ok(())
}

/// This method behaves differently, depending if it is called via community inbox (activity
/// received by community from a remote user), or via user inbox (activity received by user from
/// community). We distinguish the cases by checking if the activity is wrapper in an announce
/// (only true when sent from user to community).
///
/// In the first case, we check that the actor is listed as community mod. In the second case, we
/// only check that the announce comes from the same domain as the activity. We trust the
/// community's instance to have validated the inner activity correctly. We can't do this validation
/// here, because we don't know who the instance admins are. Plus this allows for compatibility with
/// software that uses different rules for mod actions.
pub(crate) async fn verify_mod_activity<T, Kind>(
  mod_action: &T,
  announce: Option<Announce>,
  community: &Community,
  context: &LemmyContext,
) -> Result<(), LemmyError>
where
  T: ActorAndObjectRef + BaseExt<Kind>,
{
  match announce {
    None => verify_actor_is_community_mod(mod_action, community, context).await?,
    Some(a) => verify_activity_domains_valid(&a, &community.actor_id.to_owned().into(), false)?,
  }

  Ok(())
}

/// For Add/Remove community moderator activities, check that the target field actually contains
/// /c/community/moderators. Any different values are unsupported.
fn verify_add_remove_moderator_target<T, Kind>(
  activity: &T,
  community: &Community,
) -> Result<(), LemmyError>
where
  T: ActorAndObjectRef + BaseExt<Kind> + OptTargetRef,
{
  let target = activity
    .target()
    .map(|t| t.as_single_xsd_any_uri())
    .flatten()
    .context(location_info!())?;
  if target != &generate_moderators_url(&community.actor_id)?.into_inner() {
    return Err(anyhow!("Unkown target url").into());
  }
  Ok(())
}
