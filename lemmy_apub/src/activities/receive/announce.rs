use crate::activities::receive::{
  create::receive_create,
  delete::receive_delete,
  dislike::receive_dislike,
  like::receive_like,
  receive_unhandled_activity,
  remove::receive_remove,
  undo::receive_undo,
  update::receive_update,
};
use activitystreams::{
  activity::*,
  base::{AnyBase, BaseExt},
  prelude::ExtendsExt,
};
use actix_web::HttpResponse;
use anyhow::Context;
use lemmy_utils::{location_info, LemmyError};
use lemmy_websocket::LemmyContext;

pub async fn receive_announce(
  activity: AnyBase,
  context: &LemmyContext,
) -> Result<HttpResponse, LemmyError> {
  let announce = Announce::from_any_base(activity)?.context(location_info!())?;

  // ensure that announce and community come from the same instance
  let community_id = announce
    .actor()?
    .to_owned()
    .single_xsd_any_uri()
    .context(location_info!())?;
  announce.id(community_id.domain().context(location_info!())?)?;

  let kind = announce.object().as_single_kind_str();
  let object = announce.object();
  let object2 = object.clone().one().context(location_info!())?;
  match kind {
    Some("Create") => receive_create(object2, context).await,
    Some("Update") => receive_update(object2, context).await,
    Some("Like") => receive_like(object2, context).await,
    Some("Dislike") => receive_dislike(object2, context).await,
    Some("Delete") => receive_delete(object2, context).await,
    Some("Remove") => receive_remove(object2, context).await,
    Some("Undo") => receive_undo(object2, context).await,
    _ => receive_unhandled_activity(announce),
  }
}
