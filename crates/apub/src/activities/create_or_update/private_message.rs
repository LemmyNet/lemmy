use crate::{
  activities::{generate_activity_id, send_lemmy_activity, verify_person},
  objects::{person::ApubPerson, private_message::ApubPrivateMessage},
  protocol::activities::{
    create_or_update::chat_message::CreateOrUpdateChatMessage,
    CreateOrUpdateType,
  },
  ActorType,
  SendActivity,
};
use activitypub_federation::{
  core::object_id::ObjectId,
  data::Data,
  traits::{ActivityHandler, Actor, ApubObject},
  utils::verify_domains_match,
};
use lemmy_api_common::{
  context::LemmyContext,
  private_message::{CreatePrivateMessage, EditPrivateMessage, PrivateMessageResponse},
  websocket::UserOperationCrud,
};
use lemmy_db_schema::{
  newtypes::PersonId,
  source::{person::Person, private_message::PrivateMessage},
  traits::Crud,
};
use lemmy_utils::error::LemmyError;
use url::Url;

#[async_trait::async_trait(?Send)]
impl SendActivity for CreatePrivateMessage {
  type Response = PrivateMessageResponse;

  async fn send_activity(
    _request: &Self,
    response: &Self::Response,
    context: &LemmyContext,
  ) -> Result<(), LemmyError> {
    CreateOrUpdateChatMessage::send(
      &response.private_message_view.private_message,
      response.private_message_view.creator.id,
      CreateOrUpdateType::Create,
      context,
    )
    .await
  }
}
#[async_trait::async_trait(?Send)]
impl SendActivity for EditPrivateMessage {
  type Response = PrivateMessageResponse;

  async fn send_activity(
    _request: &Self,
    response: &Self::Response,
    context: &LemmyContext,
  ) -> Result<(), LemmyError> {
    CreateOrUpdateChatMessage::send(
      &response.private_message_view.private_message,
      response.private_message_view.creator.id,
      CreateOrUpdateType::Update,
      context,
    )
    .await
  }
}

impl CreateOrUpdateChatMessage {
  #[tracing::instrument(skip_all)]
  async fn send(
    private_message: &PrivateMessage,
    sender_id: PersonId,
    kind: CreateOrUpdateType,
    context: &LemmyContext,
  ) -> Result<(), LemmyError> {
    let recipient_id = private_message.recipient_id;
    let sender: ApubPerson = Person::read(context.pool(), sender_id).await?.into();
    let recipient: ApubPerson = Person::read(context.pool(), recipient_id).await?.into();

    let id = generate_activity_id(
      kind.clone(),
      &context.settings().get_protocol_and_hostname(),
    )?;
    let create_or_update = CreateOrUpdateChatMessage {
      id: id.clone(),
      actor: ObjectId::new(sender.actor_id()),
      to: [ObjectId::new(recipient.actor_id())],
      object: ApubPrivateMessage(private_message.clone())
        .into_apub(context)
        .await?,
      kind,
    };
    let inbox = vec![recipient.shared_inbox_or_inbox()];
    send_lemmy_activity(context, create_or_update, &sender, inbox, true).await
  }
}

#[async_trait::async_trait(?Send)]
impl ActivityHandler for CreateOrUpdateChatMessage {
  type DataType = LemmyContext;
  type Error = LemmyError;

  fn id(&self) -> &Url {
    &self.id
  }

  fn actor(&self) -> &Url {
    self.actor.inner()
  }

  #[tracing::instrument(skip_all)]
  async fn verify(
    &self,
    context: &Data<LemmyContext>,
    request_counter: &mut i32,
  ) -> Result<(), LemmyError> {
    verify_person(&self.actor, context, request_counter).await?;
    verify_domains_match(self.actor.inner(), self.object.id.inner())?;
    verify_domains_match(self.to[0].inner(), self.object.to[0].inner())?;
    ApubPrivateMessage::verify(&self.object, self.actor.inner(), context, request_counter).await?;
    Ok(())
  }

  #[tracing::instrument(skip_all)]
  async fn receive(
    self,
    context: &Data<LemmyContext>,
    request_counter: &mut i32,
  ) -> Result<(), LemmyError> {
    let private_message =
      ApubPrivateMessage::from_apub(self.object, context, request_counter).await?;

    let notif_type = match self.kind {
      CreateOrUpdateType::Create => UserOperationCrud::CreatePrivateMessage,
      CreateOrUpdateType::Update => UserOperationCrud::EditPrivateMessage,
    };
    context
      .send_pm_ws_message(&notif_type, private_message.id, None)
      .await?;

    Ok(())
  }
}
