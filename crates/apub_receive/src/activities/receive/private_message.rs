use activitystreams::{activity::ActorAndObjectRefExt, base::AsBase, object::AsObject, public};
use anyhow::{anyhow, Context};
use lemmy_apub::{
  check_is_apub_id_valid,
  fetcher::person::get_or_fetch_and_upsert_person,
  get_activity_to_and_cc,
};
use lemmy_utils::{location_info, LemmyError};
use lemmy_websocket::LemmyContext;

// TODO: which of this do we still need?
async fn check_private_message_activity_valid<T, Kind>(
  activity: &T,
  context: &LemmyContext,
  request_counter: &mut i32,
) -> Result<(), LemmyError>
where
  T: AsBase<Kind> + AsObject<Kind> + ActorAndObjectRefExt,
{
  let to_and_cc = get_activity_to_and_cc(activity);
  if to_and_cc.len() != 1 {
    return Err(anyhow!("Private message can only be addressed to one person").into());
  }
  if to_and_cc.contains(&public()) {
    return Err(anyhow!("Private message cant be public").into());
  }
  let person_id = activity
    .actor()?
    .to_owned()
    .single_xsd_any_uri()
    .context(location_info!())?;
  check_is_apub_id_valid(&person_id, false)?;
  // check that the sender is a person, not a community
  get_or_fetch_and_upsert_person(&person_id, &context, request_counter).await?;

  Ok(())
}
