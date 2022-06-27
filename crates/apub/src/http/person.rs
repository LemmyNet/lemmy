use crate::{
  activity_lists::PersonInboxActivities,
  fetcher::user_or_community::UserOrCommunity,
  generate_outbox_url,
  http::{create_apub_response, create_apub_tombstone_response, receive_lemmy_activity},
  objects::person::ApubPerson,
  protocol::collections::empty_outbox::EmptyOutbox,
};
use activitypub_federation::{deser::context::WithContext, traits::ApubObject};
use actix_web::{web, HttpRequest, HttpResponse};
use lemmy_api_common::utils::blocking;
use lemmy_db_schema::{source::person::Person, traits::ApubActor};
use lemmy_utils::error::LemmyError;
use lemmy_websocket::LemmyContext;
use serde::Deserialize;

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
    Person::read_from_name(conn, &user_name, true)
  })
  .await??
  .into();

  if !person.deleted {
    let apub = person.into_apub(&context).await?;

    Ok(create_apub_response(&apub))
  } else {
    Ok(create_apub_tombstone_response(person.actor_id.clone()))
  }
}

#[tracing::instrument(skip_all)]
pub async fn person_inbox(
  request: HttpRequest,
  payload: String,
  context: web::Data<LemmyContext>,
) -> Result<HttpResponse, LemmyError> {
  receive_lemmy_activity::<WithContext<PersonInboxActivities>, UserOrCommunity>(
    request, payload, context,
  )
  .await
}

#[tracing::instrument(skip_all)]
pub(crate) async fn get_apub_person_outbox(
  info: web::Path<PersonQuery>,
  context: web::Data<LemmyContext>,
) -> Result<HttpResponse, LemmyError> {
  let person = blocking(context.pool(), move |conn| {
    Person::read_from_name(conn, &info.user_name, false)
  })
  .await??;
  let outbox_id = generate_outbox_url(&person.actor_id)?.into();
  let outbox = EmptyOutbox::new(outbox_id).await?;
  Ok(create_apub_response(&outbox))
}
