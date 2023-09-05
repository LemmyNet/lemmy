use crate::{
    activity_lists::PersonInboxActivities,
    fetcher::user_or_community::UserOrCommunity,
    http::{create_apub_response, create_apub_tombstone_response},
    objects::person::ApubPerson,
    protocol::collections::empty_outbox::EmptyOutbox,
};
use activitypub_federation::{
    actix_web::inbox::receive_activity, config::Data, protocol::context::WithContext,
    traits::Object,
};
use actix_web::{web, web::Bytes, HttpRequest, HttpResponse};
use lemmy_api_common::{context::LemmyContext, utils::generate_outbox_url};
use lemmy_db_schema::{source::person::Person, traits::ApubActor};
use lemmy_utils::error::LemmyError;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct PersonQuery {
    user_name: String,
}

/// Return the ActivityPub json representation of a local person over HTTP.
#[tracing::instrument(skip_all)]
pub(crate) async fn get_apub_person_http(
    info: web::Path<PersonQuery>,
    context: Data<LemmyContext>,
) -> Result<HttpResponse, LemmyError> {
    let user_name = info.into_inner().user_name;
    // TODO: this needs to be able to read deleted persons, so that it can send tombstones
    let person: ApubPerson = Person::read_from_name(&mut context.pool(), &user_name, true)
        .await?
        .into();

    if !person.deleted {
        let apub = person.into_json(&context).await?;

        create_apub_response(&apub)
    } else {
        create_apub_tombstone_response(person.actor_id.clone())
    }
}

#[tracing::instrument(skip_all)]
pub async fn person_inbox(
    request: HttpRequest,
    body: Bytes,
    data: Data<LemmyContext>,
) -> Result<HttpResponse, LemmyError> {
    receive_activity::<WithContext<PersonInboxActivities>, UserOrCommunity, LemmyContext>(
        request, body, &data,
    )
    .await
}

#[tracing::instrument(skip_all)]
pub(crate) async fn get_apub_person_outbox(
    info: web::Path<PersonQuery>,
    context: Data<LemmyContext>,
) -> Result<HttpResponse, LemmyError> {
    let person = Person::read_from_name(&mut context.pool(), &info.user_name, false).await?;
    let outbox_id = generate_outbox_url(&person.actor_id)?.into();
    let outbox = EmptyOutbox::new(outbox_id)?;
    create_apub_response(&outbox)
}
