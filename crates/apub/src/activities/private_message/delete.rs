use crate::{
  activities::{generate_activity_id, send_lemmy_activity, verify_activity, verify_person},
  objects::{person::ApubPerson, private_message::ApubPrivateMessage},
  protocol::activities::private_message::delete::DeletePrivateMessage,
};
use activitystreams_kinds::activity::DeleteType;
use lemmy_api_common::blocking;
use lemmy_apub_lib::{
  data::Data,
  object_id::ObjectId,
  traits::{ActivityHandler, ActorType},
  verify::verify_domains_match,
};
use lemmy_db_schema::{
  source::{person::Person, private_message::PrivateMessage},
  traits::Crud,
};
use lemmy_utils::LemmyError;
use lemmy_websocket::{send::send_pm_ws_message, LemmyContext, UserOperationCrud};

impl DeletePrivateMessage {
  pub(in crate::activities::private_message) fn new(
    actor: &ApubPerson,
    pm: &PrivateMessage,
    context: &LemmyContext,
  ) -> Result<DeletePrivateMessage, LemmyError> {
    Ok(DeletePrivateMessage {
      actor: ObjectId::new(actor.actor_id()),
      to: [ObjectId::new(actor.actor_id())],
      object: ObjectId::new(pm.ap_id.clone()),
      kind: DeleteType::Delete,
      id: generate_activity_id(
        DeleteType::Delete,
        &context.settings().get_protocol_and_hostname(),
      )?,
      unparsed: Default::default(),
    })
  }

  #[tracing::instrument(skip_all)]
  pub async fn send(
    actor: &ApubPerson,
    pm: &ApubPrivateMessage,
    context: &LemmyContext,
  ) -> Result<(), LemmyError> {
    let delete = DeletePrivateMessage::new(actor, pm, context)?;
    let delete_id = delete.id.clone();

    let recipient_id = pm.recipient_id;
    let recipient: ApubPerson =
      blocking(context.pool(), move |conn| Person::read(conn, recipient_id))
        .await??
        .into();
    let inbox = vec![recipient.shared_inbox_or_inbox_url()];
    send_lemmy_activity(context, &delete, &delete_id, actor, inbox, true).await
  }
}

#[async_trait::async_trait(?Send)]
impl ActivityHandler for DeletePrivateMessage {
  type DataType = LemmyContext;

  #[tracing::instrument(skip_all)]
  async fn verify(
    &self,
    context: &Data<LemmyContext>,
    request_counter: &mut i32,
  ) -> Result<(), LemmyError> {
    verify_activity(&self.id, self.actor.inner(), &context.settings())?;
    verify_person(&self.actor, context, request_counter).await?;
    verify_domains_match(self.actor.inner(), self.object.inner())?;
    Ok(())
  }

  #[tracing::instrument(skip_all)]
  async fn receive(
    self,
    context: &Data<LemmyContext>,
    _request_counter: &mut i32,
  ) -> Result<(), LemmyError> {
    let private_message = self.object.dereference_local(context).await?;
    let deleted_private_message = blocking(context.pool(), move |conn| {
      PrivateMessage::update_deleted(conn, private_message.id, true)
    })
    .await??;

    send_pm_ws_message(
      deleted_private_message.id,
      UserOperationCrud::DeletePrivateMessage,
      None,
      context,
    )
    .await?;

    Ok(())
  }
}
