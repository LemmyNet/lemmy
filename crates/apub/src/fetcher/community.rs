use crate::{
  fetcher::{fetch::fetch_remote_object, is_deleted, should_refetch_actor},
  inbox::user_inbox::receive_announce,
  objects::FromApub,
  GroupExt,
};
use activitystreams::{
  actor::ApActorExt,
  collection::{CollectionExt, OrderedCollection},
};
use anyhow::Context;
use diesel::result::Error::NotFound;
use lemmy_api_structs::blocking;
use lemmy_db_queries::{source::community::Community_, ApubObject};
use lemmy_db_schema::source::community::Community;
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
  let community = Community::from_apub(
    &group,
    context,
    apub_id.to_owned(),
    recursion_counter,
    false,
  )
  .await?;

  // only fetch outbox for new communities, otherwise this can create an infinite loop
  if old_community.is_none() {
    let outbox = group.inner.outbox()?.context(location_info!())?;
    fetch_community_outbox(context, outbox, &community, recursion_counter).await?
  }

  Ok(community)
}

async fn fetch_community_outbox(
  context: &LemmyContext,
  outbox: &Url,
  community: &Community,
  recursion_counter: &mut i32,
) -> Result<(), LemmyError> {
  let outbox =
    fetch_remote_object::<OrderedCollection>(context.client(), outbox, recursion_counter).await?;
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

pub(crate) async fn fetch_community_mods(
  context: &LemmyContext,
  group: &GroupExt,
  recursion_counter: &mut i32,
) -> Result<Vec<Url>, LemmyError> {
  if let Some(mods_url) = &group.ext_one.moderators {
    let mods =
      fetch_remote_object::<OrderedCollection>(context.client(), mods_url, recursion_counter)
        .await?;
    let mods = mods
      .items()
      .map(|i| i.as_many())
      .flatten()
      .context(location_info!())?
      .iter()
      .filter_map(|i| i.as_xsd_any_uri())
      .map(|u| u.to_owned())
      .collect();
    Ok(mods)
  } else {
    Ok(vec![])
  }
}
