use crate::{
  activities::receive::{
    create::receive_create,
    delete::receive_delete,
    dislike::receive_dislike,
    like::receive_like,
    receive_unhandled_activity,
    remove::receive_remove,
    undo::receive_undo,
    update::receive_update,
    verify_activity_domains_valid,
  },
  check_is_apub_id_valid,
  ActorType,
};
use activitystreams::{activity::*, base::AnyBase, prelude::ExtendsExt};
use actix_web::HttpResponse;
use anyhow::Context;
use lemmy_utils::{location_info, LemmyError};
use lemmy_websocket::LemmyContext;

pub async fn receive_announce(
  context: &LemmyContext,
  activity: AnyBase,
  actor: &dyn ActorType,
) -> Result<HttpResponse, LemmyError> {
  let announce = Announce::from_any_base(activity)?.context(location_info!())?;
  verify_activity_domains_valid(&announce, actor.actor_id()?, false)?;

  let kind = announce.object().as_single_kind_str();
  let object = announce
    .object()
    .to_owned()
    .one()
    .context(location_info!())?;

  let inner_id = object.id().context(location_info!())?.to_owned();
  check_is_apub_id_valid(&inner_id)?;

  match kind {
    Some("Create") => receive_create(context, object, inner_id).await,
    Some("Update") => receive_update(context, object, inner_id).await,
    Some("Like") => receive_like(context, object, inner_id).await,
    Some("Dislike") => receive_dislike(context, object, inner_id).await,
    Some("Delete") => receive_delete(context, object, inner_id).await,
    Some("Remove") => receive_remove(context, object, inner_id).await,
    Some("Undo") => receive_undo(context, object, inner_id).await,
    _ => receive_unhandled_activity(announce),
  }
}
