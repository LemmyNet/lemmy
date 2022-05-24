use crate::{
  activity_lists::PersonInboxActivities,
  context::WithContext,
  generate_outbox_url,
  http::{
    create_apub_response,
    create_apub_tombstone_response,
    payload_to_string,
    receive_activity,
    ActivityCommonFields,
  },
  objects::person::ApubPerson,
  protocol::collections::empty_outbox::EmptyOutbox,
};
use actix_web::{web, web::Payload, HttpRequest, HttpResponse};
use lemmy_api_common::utils::blocking;
use lemmy_apub_lib::traits::ApubObject;
use lemmy_db_schema::{source::person::Person, traits::ApubActor};
use lemmy_utils::LemmyError;
use lemmy_websocket::LemmyContext;
use serde::Deserialize;
use tracing::info;

#[derive(Deserialize)]
pub struct PersonQuery {
  user_name: String,
}

/// Return the ActivityPub json representation of a local person over HTTP.
#[tracing::instrument(skip_all)]
pub(crate) async fn get_apub_person_http(
  info: web::Path<PersonQuery>,
  context: web::Data<LemmyContext>,
) -> Result<HttpResponse, LemmyError> {
  let user_name = info.into_inner().user_name;
  // TODO: this needs to be able to read deleted persons, so that it can send tombstones
  let person: ApubPerson = blocking(context.pool(), move |conn| {
    Person::read_from_name(conn, &user_name)
  })
  .await??
  .into();

  if !person.deleted {
    let apub = person.into_apub(&context).await?;

    Ok(create_apub_response(&apub))
  } else {
    Ok(create_apub_tombstone_response(&person.to_tombstone()?))
  }
}

#[tracing::instrument(skip_all)]
pub async fn person_inbox(
  request: HttpRequest,
  payload: Payload,
  _path: web::Path<String>,
  context: web::Data<LemmyContext>,
) -> Result<HttpResponse, LemmyError> {
  let unparsed = payload_to_string(payload).await?;
  info!("Received person inbox activity {}", unparsed);
  let activity_data: ActivityCommonFields = serde_json::from_str(&unparsed)?;
  let activity = serde_json::from_str::<WithContext<PersonInboxActivities>>(&unparsed)?;
  receive_person_inbox(activity.inner(), activity_data, request, &context).await
}

pub(in crate::http) async fn receive_person_inbox(
  activity: PersonInboxActivities,
  activity_data: ActivityCommonFields,
  request: HttpRequest,
  context: &LemmyContext,
) -> Result<HttpResponse, LemmyError> {
  receive_activity(request, activity, activity_data, context).await
}

#[tracing::instrument(skip_all)]
pub(crate) async fn get_apub_person_outbox(
  info: web::Path<PersonQuery>,
  context: web::Data<LemmyContext>,
) -> Result<HttpResponse, LemmyError> {
  let person = blocking(context.pool(), move |conn| {
    Person::read_from_name(conn, &info.user_name)
  })
  .await??;
  let outbox_id = generate_outbox_url(&person.actor_id)?.into();
  let outbox = EmptyOutbox::new(outbox_id).await?;
  Ok(create_apub_response(&outbox))
}
