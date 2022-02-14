use crate::{
  activities::{generate_activity_id, send_lemmy_activity, verify_activity, verify_person},
  objects::{person::ApubPerson, private_message::ApubPrivateMessage},
  protocol::activities::{
    create_or_update::private_message::CreateOrUpdatePrivateMessage,
    CreateOrUpdateType,
  },
};
use lemmy_api_common::blocking;
use lemmy_apub_lib::{
  data::Data,
  object_id::ObjectId,
  traits::{ActivityHandler, ActorType, ApubObject},
  verify::verify_domains_match,
};
use lemmy_db_schema::{source::person::Person, traits::Crud};
use lemmy_utils::LemmyError;
use lemmy_websocket::{send::send_pm_ws_message, LemmyContext, UserOperationCrud};

impl CreateOrUpdatePrivateMessage {
  #[tracing::instrument(skip_all)]
  pub async fn send(
    private_message: ApubPrivateMessage,
    actor: &ApubPerson,
    kind: CreateOrUpdateType,
    context: &LemmyContext,
  ) -> Result<(), LemmyError> {
    let recipient_id = private_message.recipient_id;
    let recipient: ApubPerson =
      blocking(context.pool(), move |conn| Person::read(conn, recipient_id))
        .await??
        .into();

    let id = generate_activity_id(
      kind.clone(),
      &context.settings().get_protocol_and_hostname(),
    )?;
    let create_or_update = CreateOrUpdatePrivateMessage {
      id: id.clone(),
      actor: ObjectId::new(actor.actor_id()),
      to: [ObjectId::new(recipient.actor_id())],
      object: private_message.into_apub(context).await?,
      kind,
      unparsed: Default::default(),
    };
    let inbox = vec![recipient.shared_inbox_or_inbox_url()];
    send_lemmy_activity(context, &create_or_update, &id, actor, inbox, true).await
  }
}

#[async_trait::async_trait(?Send)]
impl ActivityHandler for CreateOrUpdatePrivateMessage {
  type DataType = LemmyContext;

  #[tracing::instrument(skip_all)]
  async fn verify(
    &self,
    context: &Data<LemmyContext>,
    request_counter: &mut i32,
  ) -> Result<(), LemmyError> {
    verify_activity(&self.id, self.actor.inner(), &context.settings())?;
    verify_person(&self.actor, context, request_counter).await?;
    verify_domains_match(self.actor.inner(), self.object.id.inner())?;
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
    send_pm_ws_message(private_message.id, notif_type, None, context).await?;

    Ok(())
  }
}
