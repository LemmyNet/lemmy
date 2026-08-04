use super::ActorPath;
use crate::protocol::collections::url_collection::UrlCollection;
use activitypub_federation::{config::Data, traits::Object};
use actix_web::{HttpResponse, web::Path};
use lemmy_api_utils::{context::LemmyContext, utils::generate_outbox_url};
use lemmy_apub_objects::objects::person::ApubPerson;
use lemmy_db_schema::{source::person::Person, traits::ApubActor};
use lemmy_utils::{
  FEDERATION_CONTEXT,
  error::{LemmyErrorType, LemmyResult},
};

/// Return the ActivityPub json representation of a person over HTTP.
pub(crate) async fn get_apub_person_http(
  info: Path<ActorPath>,
  context: Data<LemmyContext>,
) -> LemmyResult<HttpResponse> {
  let (name, domain) = info.split_name();
  // This needs to be able to read deleted persons, so that it can send tombstones
  let person: ApubPerson = Person::read_from_name(&mut context.pool(), name, domain, true)
    .await?
    .ok_or(LemmyErrorType::NotFound)?
    .into();

  person.http_response(&FEDERATION_CONTEXT, &context).await
}

pub(crate) async fn get_apub_person_outbox(
  info: Path<ActorPath>,
  context: Data<LemmyContext>,
) -> LemmyResult<HttpResponse> {
  let person = Person::read_from_name(&mut context.pool(), &info.name, None, false)
    .await?
    .ok_or(LemmyErrorType::NotFound)?;
  let outbox_id = generate_outbox_url(&person.ap_id)?.to_string();
  UrlCollection::new_empty_response(outbox_id)
}
