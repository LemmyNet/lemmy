use crate::{
  activities::{generate_activity_id, verify_activity, verify_person},
  activity_queue::send_activity_new,
  extensions::context::lemmy_context,
  ActorType,
};
use activitystreams::{
  activity::kind::DeleteType,
  base::AnyBase,
  primitives::OneOrMany,
  unparsed::Unparsed,
};
use lemmy_api_common::blocking;
use lemmy_apub_lib::{verify_domains_match, ActivityFields, ActivityHandler};
use lemmy_db_queries::{source::private_message::PrivateMessage_, ApubObject, Crud};
use lemmy_db_schema::source::{person::Person, private_message::PrivateMessage};
use lemmy_utils::LemmyError;
use lemmy_websocket::{send::send_pm_ws_message, LemmyContext, UserOperationCrud};
use serde::{Deserialize, Serialize};
use url::Url;

#[derive(Clone, Debug, Deserialize, Serialize, ActivityFields)]
#[serde(rename_all = "camelCase")]
pub struct DeletePrivateMessage {
  actor: Url,
  to: Url,
  pub(in crate::activities::private_message) object: Url,
  #[serde(rename = "type")]
  kind: DeleteType,
  id: Url,
  #[serde(rename = "@context")]
  context: OneOrMany<AnyBase>,
  #[serde(flatten)]
  unparsed: Unparsed,
}

impl DeletePrivateMessage {
  pub(in crate::activities::private_message) fn new(
    actor: &Person,
    pm: &PrivateMessage,
  ) -> Result<DeletePrivateMessage, LemmyError> {
    Ok(DeletePrivateMessage {
      actor: actor.actor_id(),
      to: actor.actor_id(),
      object: pm.ap_id.clone().into(),
      kind: DeleteType::Delete,
      id: generate_activity_id(DeleteType::Delete)?,
      context: lemmy_context(),
      unparsed: Default::default(),
    })
  }
  pub async fn send(
    actor: &Person,
    pm: &PrivateMessage,
    context: &LemmyContext,
  ) -> Result<(), LemmyError> {
    let delete = DeletePrivateMessage::new(actor, pm)?;
    let delete_id = delete.id.clone();

    let recipient_id = pm.recipient_id;
    let recipient =
      blocking(context.pool(), move |conn| Person::read(conn, recipient_id)).await??;
    let inbox = vec![recipient.get_shared_inbox_or_inbox_url()];
    send_activity_new(context, &delete, &delete_id, actor, inbox, true).await
  }
}

#[async_trait::async_trait(?Send)]
impl ActivityHandler for DeletePrivateMessage {
  async fn verify(
    &self,
    context: &LemmyContext,
    request_counter: &mut i32,
  ) -> Result<(), LemmyError> {
    verify_activity(self)?;
    verify_person(&self.actor, context, request_counter).await?;
    verify_domains_match(&self.actor, &self.object)?;
    Ok(())
  }

  async fn receive(
    self,
    context: &LemmyContext,
    _request_counter: &mut i32,
  ) -> Result<(), LemmyError> {
    let ap_id = self.object.clone();
    let private_message = blocking(context.pool(), move |conn| {
      PrivateMessage::read_from_apub_id(conn, &ap_id.into())
    })
    .await??;
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
