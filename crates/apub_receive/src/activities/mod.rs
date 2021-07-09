use anyhow::anyhow;
use lemmy_api_common::blocking;
use lemmy_apub::{
  check_is_apub_id_valid,
  fetcher::{community::get_or_fetch_and_upsert_community, person::get_or_fetch_and_upsert_person},
};
use lemmy_apub_lib::{verify_domains_match, ActivityCommonFields};
use lemmy_db_queries::ApubObject;
use lemmy_db_schema::source::{community::Community, person::Person};
use lemmy_db_views_actor::community_view::CommunityView;
use lemmy_utils::LemmyError;
use lemmy_websocket::LemmyContext;
use url::Url;

pub mod comment;
pub mod community;
pub mod following;
pub mod post;
pub mod private_message;

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
  actor_id: Url,
  activity_cc: Url,
  context: &LemmyContext,
) -> Result<(), LemmyError> {
  let community = blocking(context.pool(), move |conn| {
    Community::read_from_apub_id(conn, &activity_cc.into())
  })
  .await??;

  if community.local {
    let actor = blocking(context.pool(), move |conn| {
      Person::read_from_apub_id(conn, &actor_id.into())
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
