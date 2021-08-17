use crate::{
  activities::community::announce::AnnounceActivity,
  fetcher::{
    fetch::fetch_remote_object,
    is_deleted,
    person::get_or_fetch_and_upsert_person,
    should_refetch_actor,
  },
  objects::{community::Group, FromApub},
};
use activitystreams::collection::{CollectionExt, OrderedCollection};
use anyhow::Context;
use diesel::result::Error::NotFound;
use lemmy_api_common::blocking;
use lemmy_apub_lib::ActivityHandler;
use lemmy_db_queries::{source::community::Community_, ApubObject, Joinable};
use lemmy_db_schema::source::community::{Community, CommunityModerator, CommunityModeratorForm};
use lemmy_db_views_actor::community_moderator_view::CommunityModeratorView;
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
  request_counter: &mut i32,
) -> Result<Community, LemmyError> {
  let group = fetch_remote_object::<Group>(context.client(), apub_id, request_counter).await;

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
  let community = Community::from_apub(&group, context, apub_id, request_counter).await?;

  update_community_mods(&group, &community, context, request_counter).await?;

  // only fetch outbox for new communities, otherwise this can create an infinite loop
  if old_community.is_none() {
    fetch_community_outbox(context, &group.outbox, request_counter).await?
  }

  Ok(community)
}

async fn update_community_mods(
  group: &Group,
  community: &Community,
  context: &LemmyContext,
  request_counter: &mut i32,
) -> Result<(), LemmyError> {
  let new_moderators = fetch_community_mods(context, group, request_counter).await?;
  let community_id = community.id;
  let current_moderators = blocking(context.pool(), move |conn| {
    CommunityModeratorView::for_community(conn, community_id)
  })
  .await??;
  // Remove old mods from database which arent in the moderators collection anymore
  for mod_user in &current_moderators {
    if !new_moderators.contains(&mod_user.moderator.actor_id.clone().into()) {
      let community_moderator_form = CommunityModeratorForm {
        community_id: mod_user.community.id,
        person_id: mod_user.moderator.id,
      };
      blocking(context.pool(), move |conn| {
        CommunityModerator::leave(conn, &community_moderator_form)
      })
      .await??;
    }
  }

  // Add new mods to database which have been added to moderators collection
  for mod_uri in new_moderators {
    let mod_user = get_or_fetch_and_upsert_person(&mod_uri, context, request_counter).await?;

    if !current_moderators
      .clone()
      .iter()
      .map(|c| c.moderator.actor_id.clone())
      .any(|x| x == mod_user.actor_id)
    {
      let community_moderator_form = CommunityModeratorForm {
        community_id: community.id,
        person_id: mod_user.id,
      };
      blocking(context.pool(), move |conn| {
        CommunityModerator::join(conn, &community_moderator_form)
      })
      .await??;
    }
  }

  Ok(())
}

async fn fetch_community_outbox(
  context: &LemmyContext,
  outbox: &Url,
  recursion_counter: &mut i32,
) -> Result<(), LemmyError> {
  let outbox =
    fetch_remote_object::<OrderedCollection>(context.client(), outbox, recursion_counter).await?;
  let outbox_activities = outbox.items().context(location_info!())?.clone();
  let mut outbox_activities = outbox_activities.many().context(location_info!())?;
  if outbox_activities.len() > 20 {
    outbox_activities = outbox_activities[0..20].to_vec();
  }

  for announce in outbox_activities {
    // TODO: instead of converting like this, we should create a struct CommunityOutbox with
    //       AnnounceActivity as inner type, but that gives me stackoverflow
    let ser = serde_json::to_string(&announce)?;
    let announce: AnnounceActivity = serde_json::from_str(&ser)?;
    announce.receive(context, recursion_counter).await?;
  }

  Ok(())
}

pub(crate) async fn fetch_community_mods(
  context: &LemmyContext,
  group: &Group,
  recursion_counter: &mut i32,
) -> Result<Vec<Url>, LemmyError> {
  if let Some(mods_url) = &group.moderators {
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
