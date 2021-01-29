use crate::{
  fetcher::{
    fetch::fetch_remote_object,
    get_or_fetch_and_upsert_user,
    is_deleted,
    should_refetch_actor,
  },
  inbox::user_inbox::receive_announce,
  objects::FromApub,
  ActorType,
  GroupExt,
};
use activitystreams::{
  collection::{CollectionExt, OrderedCollection},
  object::ObjectExt,
};
use anyhow::Context;
use diesel::result::Error::NotFound;
use lemmy_db_queries::{source::community::Community_, ApubObject, Joinable};
use lemmy_db_schema::source::community::{Community, CommunityModerator, CommunityModeratorForm};
use lemmy_structs::blocking;
use lemmy_utils::{location_info, LemmyError};
use lemmy_websocket::LemmyContext;
use log::debug;
use url::Url;

/// Get a community from its apub ID.
///
/// If it exists locally and `!should_refetch_actor()`, it is returned directly from the database.
/// Otherwise it is fetched from the remote instance, stored and returned.
pub(crate) async fn get_or_fetch_and_upsert_community(
  apub_id: &Url,
  context: &LemmyContext,
  recursion_counter: &mut i32,
) -> Result<Community, LemmyError> {
  let apub_id_owned = apub_id.to_owned();
  let community = blocking(context.pool(), move |conn| {
    Community::read_from_apub_id(conn, &apub_id_owned.into())
  })
  .await?;

  match community {
    Ok(c) if !c.local && should_refetch_actor(c.last_refreshed_at) => {
      debug!("Fetching and updating from remote community: {}", apub_id);
      fetch_remote_community(apub_id, context, Some(c), recursion_counter).await
    }
    Ok(c) => Ok(c),
    Err(NotFound {}) => {
      debug!("Fetching and creating remote community: {}", apub_id);
      fetch_remote_community(apub_id, context, None, recursion_counter).await
    }
    Err(e) => Err(e.into()),
  }
}

/// Request a community by apub ID from a remote instance, including moderators. If `old_community`,
/// is set, this is an update for a community which is already known locally. If not, we don't know
/// the community yet and also pull the outbox, to get some initial posts.
async fn fetch_remote_community(
  apub_id: &Url,
  context: &LemmyContext,
  old_community: Option<Community>,
  recursion_counter: &mut i32,
) -> Result<Community, LemmyError> {
  let group = fetch_remote_object::<GroupExt>(context.client(), apub_id, recursion_counter).await;

  if let Some(c) = old_community.to_owned() {
    if is_deleted(&group) {
      blocking(context.pool(), move |conn| {
        Community::update_deleted(conn, c.id, true)
      })
      .await??;
    } else if group.is_err() {
      // If fetching failed, return the existing data.
      return Ok(c);
    }
  }

  let group = group?;
  let community =
    Community::from_apub(&group, context, apub_id.to_owned(), recursion_counter).await?;

  // Also add the community moderators too
  let attributed_to = group.inner.attributed_to().context(location_info!())?;
  let creator_and_moderator_uris: Vec<&Url> = attributed_to
    .as_many()
    .context(location_info!())?
    .iter()
    .map(|a| a.as_xsd_any_uri().context(""))
    .collect::<Result<Vec<&Url>, anyhow::Error>>()?;

  let mut creator_and_moderators = Vec::new();

  for uri in creator_and_moderator_uris {
    let c_or_m = get_or_fetch_and_upsert_user(uri, context, recursion_counter).await?;

    creator_and_moderators.push(c_or_m);
  }

  // TODO: need to make this work to update mods of existing communities
  if old_community.is_none() {
    let community_id = community.id;
    blocking(context.pool(), move |conn| {
      for mod_ in creator_and_moderators {
        let community_moderator_form = CommunityModeratorForm {
          community_id,
          user_id: mod_.id,
        };

        CommunityModerator::join(conn, &community_moderator_form)?;
      }
      Ok(()) as Result<(), LemmyError>
    })
    .await??;
  }

  // only fetch outbox for new communities, otherwise this can create an infinite loop
  if old_community.is_none() {
    fetch_community_outbox(context, &community, recursion_counter).await?
  }

  Ok(community)
}

async fn fetch_community_outbox(
  context: &LemmyContext,
  community: &Community,
  recursion_counter: &mut i32,
) -> Result<(), LemmyError> {
  let outbox = fetch_remote_object::<OrderedCollection>(
    context.client(),
    &community.get_outbox_url()?,
    recursion_counter,
  )
  .await?;
  let outbox_activities = outbox.items().context(location_info!())?.clone();
  let mut outbox_activities = outbox_activities.many().context(location_info!())?;
  if outbox_activities.len() > 20 {
    outbox_activities = outbox_activities[0..20].to_vec();
  }

  for activity in outbox_activities {
    receive_announce(context, activity, community, recursion_counter).await?;
  }

  Ok(())
}
