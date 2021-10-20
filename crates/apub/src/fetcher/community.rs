use crate::{
  activities::community::announce::AnnounceActivity,
  fetcher::{fetch::fetch_remote_object, object_id::ObjectId},
  objects::{community::Group, person::ApubPerson},
};
use activitystreams::{
  base::AnyBase,
  collection::{CollectionExt, OrderedCollection},
};
use anyhow::Context;
use lemmy_api_common::blocking;
use lemmy_apub_lib::{data::Data, traits::ActivityHandler};
use lemmy_db_schema::{
  source::community::{Community, CommunityModerator, CommunityModeratorForm},
  traits::Joinable,
};
use lemmy_db_views_actor::community_moderator_view::CommunityModeratorView;
use lemmy_utils::{location_info, LemmyError};
use lemmy_websocket::LemmyContext;
use url::Url;

pub(crate) async fn update_community_mods(
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
  for mod_id in new_moderators {
    let mod_id = ObjectId::new(mod_id);
    let mod_user: ApubPerson = mod_id.dereference(context, request_counter).await?;

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

pub(crate) async fn fetch_community_outbox(
  context: &LemmyContext,
  outbox: &Url,
  recursion_counter: &mut i32,
) -> Result<(), LemmyError> {
  let outbox = fetch_remote_object::<OrderedCollection>(
    context.client(),
    &context.settings(),
    outbox,
    recursion_counter,
  )
  .await?;
  let outbox_activities = outbox.items().context(location_info!())?.clone();
  let mut outbox_activities = outbox_activities.many().context(location_info!())?;
  if outbox_activities.len() > 20 {
    outbox_activities = outbox_activities[0..20].to_vec();
  }

  // We intentionally ignore errors here. This is because the outbox might contain posts from old
  // Lemmy versions, or from other software which we cant parse. In that case, we simply skip the
  // item and only parse the ones that work.
  for activity in outbox_activities {
    parse_outbox_item(activity, context, recursion_counter)
      .await
      .ok();
  }

  Ok(())
}

async fn parse_outbox_item(
  announce: AnyBase,
  context: &LemmyContext,
  request_counter: &mut i32,
) -> Result<(), LemmyError> {
  // TODO: instead of converting like this, we should create a struct CommunityOutbox with
  //       AnnounceActivity as inner type, but that gives me stackoverflow
  let ser = serde_json::to_string(&announce)?;
  let announce: AnnounceActivity = serde_json::from_str(&ser)?;
  announce
    .receive(&Data::new(context.clone()), request_counter)
    .await?;
  Ok(())
}

async fn fetch_community_mods(
  context: &LemmyContext,
  group: &Group,
  recursion_counter: &mut i32,
) -> Result<Vec<Url>, LemmyError> {
  if let Some(mods_url) = &group.moderators {
    let mods = fetch_remote_object::<OrderedCollection>(
      context.client(),
      &context.settings(),
      mods_url,
      recursion_counter,
    )
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
