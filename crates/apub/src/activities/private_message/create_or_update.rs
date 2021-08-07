use crate::{
  activities::{
    generate_activity_id,
    private_message::send_websocket_message,
    verify_activity,
    verify_person,
    CreateOrUpdateType,
  },
  activity_queue::send_activity_new,
  extensions::context::lemmy_context,
  objects::{private_message::Note, FromApub, ToApub},
  ActorType,
};
use lemmy_api_common::blocking;
use lemmy_apub_lib::{verify_domains_match, ActivityCommonFields, ActivityHandler};
use lemmy_db_queries::Crud;
use lemmy_db_schema::source::{person::Person, private_message::PrivateMessage};
use lemmy_utils::LemmyError;
use lemmy_websocket::{LemmyContext, UserOperationCrud};
use url::Url;

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateOrUpdatePrivateMessage {
  to: Url,
  object: Note,
  #[serde(rename = "type")]
  kind: CreateOrUpdateType,
  #[serde(flatten)]
  common: ActivityCommonFields,
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
      to: recipient.actor_id(),
      object: private_message.to_apub(context.pool()).await?,
      kind,
      common: ActivityCommonFields {
        context: lemmy_context(),
        id: id.clone(),
        actor: actor.actor_id(),
        unparsed: Default::default(),
      },
    };
    send_activity_new(
      context,
      &create_or_update,
      &id,
      actor,
      vec![recipient.get_shared_inbox_or_inbox_url()],
      true,
    )
    .await
  }
}
#[async_trait::async_trait(?Send)]
impl ActivityHandler for CreateOrUpdatePrivateMessage {
  async fn verify(
    &self,
    context: &LemmyContext,
    request_counter: &mut i32,
  ) -> Result<(), LemmyError> {
    verify_activity(self.common())?;
    verify_person(&self.common.actor, context, request_counter).await?;
    verify_domains_match(&self.common.actor, self.object.id_unchecked())?;
    self.object.verify(context, request_counter).await?;
    Ok(())
  }

  async fn receive(
    self,
    context: &LemmyContext,
    request_counter: &mut i32,
  ) -> Result<(), LemmyError> {
    let private_message =
      PrivateMessage::from_apub(&self.object, context, &self.common.actor, request_counter).await?;

    let notif_type = match self.kind {
      CreateOrUpdateType::Create => UserOperationCrud::CreatePrivateMessage,
      CreateOrUpdateType::Update => UserOperationCrud::EditPrivateMessage,
    };
    send_websocket_message(private_message.id, notif_type, context).await?;

    Ok(())
  }

  fn common(&self) -> &ActivityCommonFields {
    &self.common
  }
}
