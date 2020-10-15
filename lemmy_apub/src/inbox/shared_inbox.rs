use crate::{
  activities::receive::{
    announce::receive_announce,
    create::receive_create,
    delete::receive_delete,
    dislike::receive_dislike,
    like::receive_like,
    remove::receive_remove,
    undo::receive_undo,
    update::receive_update,
  },
  check_is_apub_id_valid,
  extensions::signatures::verify_signature,
  fetcher::get_or_fetch_and_upsert_actor,
  insert_activity,
};
use activitystreams::{activity::ActorAndObject, prelude::*};
use actix_web::{web, HttpRequest, HttpResponse};
use anyhow::Context;
use lemmy_utils::{location_info, LemmyError};
use lemmy_websocket::LemmyContext;
use log::debug;
use serde::{Deserialize, Serialize};
use std::fmt::Debug;

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd, Deserialize, Serialize)]
#[serde(rename_all = "PascalCase")]
pub enum ValidTypes {
  Create,
  Update,
  Like,
  Dislike,
  Delete,
  Undo,
  Remove,
  Announce,
}

// TODO: this isnt entirely correct, cause some of these receive are not ActorAndObject,
//       but it might still work due to the anybase conversion
pub type AcceptedActivities = ActorAndObject<ValidTypes>;

/// Handler for all incoming receive to user inboxes.
pub async fn shared_inbox(
  request: HttpRequest,
  input: web::Json<AcceptedActivities>,
  context: web::Data<LemmyContext>,
) -> Result<HttpResponse, LemmyError> {
  let activity = input.into_inner();

  let actor_id = activity
    .actor()?
    .to_owned()
    .single_xsd_any_uri()
    .context(location_info!())?;
  debug!(
    "Shared inbox received activity {:?} from {}",
    &activity.id_unchecked(),
    &actor_id
  );

  check_is_apub_id_valid(&actor_id)?;

  let actor = get_or_fetch_and_upsert_actor(&actor_id, &context).await?;
  verify_signature(&request, actor.as_ref())?;

  let any_base = activity.clone().into_any_base()?;
  let kind = activity.kind().context(location_info!())?;
  let res = match kind {
    ValidTypes::Announce => receive_announce(&context, any_base, actor.as_ref()).await,
    ValidTypes::Create => receive_create(&context, any_base, actor_id).await,
    ValidTypes::Update => receive_update(&context, any_base, actor_id).await,
    ValidTypes::Like => receive_like(&context, any_base, actor_id).await,
    ValidTypes::Dislike => receive_dislike(&context, any_base, actor_id).await,
    ValidTypes::Remove => receive_remove(&context, any_base, actor_id).await,
    ValidTypes::Delete => receive_delete(&context, any_base, actor_id).await,
    ValidTypes::Undo => receive_undo(&context, any_base, actor_id).await,
  };

  insert_activity(actor.user_id(), activity.clone(), false, context.pool()).await?;
  res
}
