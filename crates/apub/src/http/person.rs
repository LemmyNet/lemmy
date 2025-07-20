use crate::protocol::collections::empty_outbox::EmptyOutbox;
use activitypub_federation::{
  actix_web::response::create_http_response,
  config::Data,
  traits::Object,
};
use actix_web::{web::Path, HttpResponse};
use lemmy_api_utils::{context::LemmyContext, utils::generate_outbox_url};
use lemmy_apub_objects::objects::person::ApubPerson;
use lemmy_db_schema::{source::person::Person, traits::ApubActor};
use lemmy_utils::{
  error::{LemmyErrorType, LemmyResult},
  FEDERATION_CONTEXT,
};
use serde::Deserialize;

#[derive(Deserialize)]
pub(crate) struct PersonQuery {
  user_name: String,
}

/// Return the ActivityPub json representation of a local person over HTTP.
pub(crate) async fn get_apub_person_http(
  info: Path<PersonQuery>,
  context: Data<LemmyContext>,
) -> LemmyResult<HttpResponse> {
  let user_name = info.into_inner().user_name;
  // This needs to be able to read deleted persons, so that it can send tombstones
  let person: ApubPerson = Person::read_from_name(&mut context.pool(), &user_name, true)
    .await?
    .ok_or(LemmyErrorType::NotFound)?
    .into();

  person.http_response(&FEDERATION_CONTEXT, &context).await
}

pub(crate) async fn get_apub_person_outbox(
  info: Path<PersonQuery>,
  context: Data<LemmyContext>,
) -> LemmyResult<HttpResponse> {
  let person = Person::read_from_name(&mut context.pool(), &info.user_name, false)
    .await?
    .ok_or(LemmyErrorType::NotFound)?;
  let outbox_id = generate_outbox_url(&person.ap_id)?.into();
  let outbox = EmptyOutbox::new(outbox_id)?;
  Ok(create_http_response(outbox, &FEDERATION_CONTEXT)?)
}
