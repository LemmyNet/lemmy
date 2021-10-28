use crate::{
  activities::{
    community::announce::{AnnouncableActivities, AnnounceActivity},
    following::accept::AcceptFollowCommunity,
    private_message::{
      create_or_update::CreateOrUpdatePrivateMessage,
      delete::DeletePrivateMessage,
      undo_delete::UndoDeletePrivateMessage,
    },
  },
  collections::user_outbox::UserOutbox,
  context::WithContext,
  http::{
    create_apub_response,
    create_apub_tombstone_response,
    payload_to_string,
    receive_activity,
  },
  objects::person::ApubPerson,
};
use actix_web::{body::Body, web, web::Payload, HttpRequest, HttpResponse};
use lemmy_api_common::blocking;
use lemmy_apub_lib::traits::{ActivityFields, ActivityHandler, ApubObject};
use lemmy_db_schema::source::person::Person;
use lemmy_utils::LemmyError;
use lemmy_websocket::LemmyContext;
use log::info;
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct PersonQuery {
  user_name: String,
}

/// Return the ActivityPub json representation of a local person over HTTP.
pub(crate) async fn get_apub_person_http(
  info: web::Path<PersonQuery>,
  context: web::Data<LemmyContext>,
) -> Result<HttpResponse<Body>, LemmyError> {
  let user_name = info.into_inner().user_name;
  // TODO: this needs to be able to read deleted persons, so that it can send tombstones
  let person: ApubPerson = blocking(context.pool(), move |conn| {
    Person::find_by_name(conn, &user_name)
  })
  .await??
  .into();

  if !person.deleted {
    let apub = person.to_apub(&context).await?;

    Ok(create_apub_response(&apub))
  } else {
    Ok(create_apub_tombstone_response(&person.to_tombstone()?))
  }
}

#[derive(Clone, Debug, Deserialize, Serialize, ActivityHandler, ActivityFields)]
#[serde(untagged)]
#[activity_handler(LemmyContext)]
pub enum PersonInboxActivities {
  AcceptFollowCommunity(AcceptFollowCommunity),
  /// Some activities can also be sent from user to user, eg a comment with mentions
  AnnouncableActivities(AnnouncableActivities),
  CreateOrUpdatePrivateMessage(CreateOrUpdatePrivateMessage),
  DeletePrivateMessage(DeletePrivateMessage),
  UndoDeletePrivateMessage(UndoDeletePrivateMessage),
  AnnounceActivity(Box<AnnounceActivity>),
}

pub async fn person_inbox(
  request: HttpRequest,
  payload: Payload,
  _path: web::Path<String>,
  context: web::Data<LemmyContext>,
) -> Result<HttpResponse, LemmyError> {
  let unparsed = payload_to_string(payload).await?;
  info!("Received person inbox activity {}", unparsed);
  let activity = serde_json::from_str::<WithContext<PersonInboxActivities>>(&unparsed)?;
  receive_person_inbox(activity.inner(), request, &context).await
}

pub(in crate::http) async fn receive_person_inbox(
  activity: PersonInboxActivities,
  request: HttpRequest,
  context: &LemmyContext,
) -> Result<HttpResponse, LemmyError> {
  receive_activity(request, activity, context).await
}

pub(crate) async fn get_apub_person_outbox(
  info: web::Path<PersonQuery>,
  context: web::Data<LemmyContext>,
) -> Result<HttpResponse<Body>, LemmyError> {
  let person = blocking(context.pool(), move |conn| {
    Person::find_by_name(conn, &info.user_name)
  })
  .await??;
  let outbox = UserOutbox::new(person).await?;
  Ok(create_apub_response(&outbox))
}
