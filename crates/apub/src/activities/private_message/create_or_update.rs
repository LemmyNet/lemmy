use crate::{
  activities::{generate_activity_id, verify_activity, verify_person, CreateOrUpdateType},
  activity_queue::send_activity_new,
  extensions::context::lemmy_context,
  objects::{private_message::Note, FromApub, ToApub},
  ActorType,
};
use activitystreams::{base::AnyBase, primitives::OneOrMany, unparsed::Unparsed};
use lemmy_api_common::blocking;
use lemmy_apub_lib::{verify_domains_match, ActivityFields, ActivityHandler};
use lemmy_db_queries::Crud;
use lemmy_db_schema::source::{person::Person, private_message::PrivateMessage};
use lemmy_utils::LemmyError;
use lemmy_websocket::{send::send_pm_ws_message, LemmyContext, UserOperationCrud};
use serde::{Deserialize, Serialize};
use url::Url;

#[derive(Clone, Debug, Deserialize, Serialize, ActivityFields)]
#[serde(rename_all = "camelCase")]
pub struct CreateOrUpdatePrivateMessage {
  #[serde(rename = "@context")]
  pub context: OneOrMany<AnyBase>,
  id: Url,
  actor: Url,
  to: Url,
  cc: [Url; 0],
  object: Note,
  #[serde(rename = "type")]
  kind: CreateOrUpdateType,
  #[serde(flatten)]
  pub unparsed: Unparsed,
}

impl CreateOrUpdatePrivateMessage {
  pub async fn send(
    private_message: &PrivateMessage,
    actor: &Person,
    kind: CreateOrUpdateType,
    context: &LemmyContext,
  ) -> Result<(), LemmyError> {
    let recipient_id = private_message.recipient_id;
    let recipient =
      blocking(context.pool(), move |conn| Person::read(conn, recipient_id)).await??;

    let id = generate_activity_id(kind.clone())?;
    let create_or_update = CreateOrUpdatePrivateMessage {
      context: lemmy_context(),
      id: id.clone(),
      actor: actor.actor_id(),
      to: recipient.actor_id(),
      cc: [],
      object: private_message.to_apub(context.pool()).await?,
      kind,
      unparsed: Default::default(),
    };
    let inbox = vec![recipient.get_shared_inbox_or_inbox_url()];
    send_activity_new(context, &create_or_update, &id, actor, inbox, true).await
  }
}
#[async_trait::async_trait(?Send)]
impl ActivityHandler for CreateOrUpdatePrivateMessage {
  async fn verify(
    &self,
    context: &LemmyContext,
    request_counter: &mut i32,
  ) -> Result<(), LemmyError> {
    verify_activity(self)?;
    verify_person(&self.actor, context, request_counter).await?;
    verify_domains_match(&self.actor, self.object.id_unchecked())?;
    self.object.verify(context, request_counter).await?;
    Ok(())
  }

  async fn receive(
    self,
    context: &LemmyContext,
    request_counter: &mut i32,
  ) -> Result<(), LemmyError> {
    let private_message =
      PrivateMessage::from_apub(&self.object, context, &self.actor, request_counter).await?;

    let notif_type = match self.kind {
      CreateOrUpdateType::Create => UserOperationCrud::CreatePrivateMessage,
      CreateOrUpdateType::Update => UserOperationCrud::EditPrivateMessage,
    };
    send_pm_ws_message(private_message.id, notif_type, None, context).await?;

    Ok(())
  }
}
