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
  extensions::signatures::verify,
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

  let actor = activity
    .actor()?
    .to_owned()
    .single_xsd_any_uri()
    .context(location_info!())?;
  debug!(
    "Shared inbox received activity {:?} from {}",
    &activity.id_unchecked(),
    &actor
  );

  check_is_apub_id_valid(&actor)?;

  let actor = get_or_fetch_and_upsert_actor(&actor, &context).await?;
  verify(&request, actor.as_ref())?;

  let any_base = activity.clone().into_any_base()?;
  let kind = activity.kind().context(location_info!())?;
  let res = match kind {
    ValidTypes::Announce => receive_announce(any_base, &context).await,
    ValidTypes::Create => receive_create(any_base, &context).await,
    ValidTypes::Update => receive_update(any_base, &context).await,
    ValidTypes::Like => receive_like(any_base, &context).await,
    ValidTypes::Dislike => receive_dislike(any_base, &context).await,
    ValidTypes::Remove => receive_remove(any_base, &context).await,
    ValidTypes::Delete => receive_delete(any_base, &context).await,
    ValidTypes::Undo => receive_undo(any_base, &context).await,
  };

  insert_activity(actor.user_id(), activity.clone(), false, context.pool()).await?;
  res
}
