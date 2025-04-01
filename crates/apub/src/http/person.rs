use crate::{
  http::{create_apub_response, create_apub_tombstone_response},
  objects::person::ApubPerson,
  protocol::collections::empty_outbox::EmptyOutbox,
};
use activitypub_federation::{config::Data, traits::Object};
use actix_web::{web, HttpResponse};
use lemmy_api_common::{context::LemmyContext, utils::generate_outbox_url};
use lemmy_db_schema::{source::person::Person, traits::ApubActor};
use lemmy_utils::error::{LemmyErrorType, LemmyResult};
use serde::Deserialize;

#[derive(Deserialize)]
pub struct PersonQuery {
  user_name: String,
}

/// Return the ActivityPub json representation of a local person over HTTP.
pub(crate) async fn get_apub_person_http(
  info: web::Path<PersonQuery>,
  context: Data<LemmyContext>,
) -> LemmyResult<HttpResponse> {
  let user_name = info.into_inner().user_name;
  // TODO: this needs to be able to read deleted persons, so that it can send tombstones
  let person: ApubPerson = Person::read_from_name(&mut context.pool(), &user_name, true)
    .await?
    .ok_or(LemmyErrorType::NotFound)?
    .into();

  if !person.deleted {
    let apub = person.into_json(&context).await?;

    create_apub_response(&apub)
  } else {
    create_apub_tombstone_response(person.ap_id.clone())
  }
}

pub(crate) async fn get_apub_person_outbox(
  info: web::Path<PersonQuery>,
  context: Data<LemmyContext>,
) -> LemmyResult<HttpResponse> {
  let person = Person::read_from_name(&mut context.pool(), &info.user_name, false)
    .await?
    .ok_or(LemmyErrorType::NotFound)?;
  let outbox_id = generate_outbox_url(&person.ap_id)?.into();
  let outbox = EmptyOutbox::new(outbox_id)?;
  create_apub_response(&outbox)
}
