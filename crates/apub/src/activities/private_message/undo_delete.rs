use crate::{
  activities::{
    generate_activity_id,
    private_message::delete::DeletePrivateMessage,
    verify_activity,
    verify_person,
  },
  activity_queue::send_activity_new,
  extensions::context::lemmy_context,
  ActorType,
};
use activitystreams::{
  activity::kind::UndoType,
  base::AnyBase,
  primitives::OneOrMany,
  unparsed::Unparsed,
};
use lemmy_api_common::blocking;
use lemmy_apub_lib::{verify_domains_match, verify_urls_match, ActivityFields, ActivityHandler};
use lemmy_db_queries::{source::private_message::PrivateMessage_, ApubObject, Crud};
use lemmy_db_schema::source::{person::Person, private_message::PrivateMessage};
use lemmy_utils::LemmyError;
use lemmy_websocket::{send::send_pm_ws_message, LemmyContext, UserOperationCrud};
use serde::{Deserialize, Serialize};
use url::Url;

#[derive(Clone, Debug, Deserialize, Serialize, ActivityFields)]
#[serde(rename_all = "camelCase")]
pub struct UndoDeletePrivateMessage {
  actor: Url,
  to: Url,
  object: DeletePrivateMessage,
  #[serde(rename = "type")]
  kind: UndoType,
  id: Url,
  #[serde(rename = "@context")]
  context: OneOrMany<AnyBase>,
  #[serde(flatten)]
  unparsed: Unparsed,
}

impl UndoDeletePrivateMessage {
  pub async fn send(
    actor: &Person,
    pm: &PrivateMessage,
    context: &LemmyContext,
  ) -> Result<(), LemmyError> {
    let recipient_id = pm.recipient_id;
    let recipient =
      blocking(context.pool(), move |conn| Person::read(conn, recipient_id)).await??;

    let object = DeletePrivateMessage::new(actor, pm)?;
    let id = generate_activity_id(UndoType::Undo)?;
    let undo = UndoDeletePrivateMessage {
      actor: actor.actor_id(),
      to: recipient.actor_id(),
      object,
      kind: UndoType::Undo,
      id: id.clone(),
      context: lemmy_context(),
      unparsed: Default::default(),
    };
    let inbox = vec![recipient.get_shared_inbox_or_inbox_url()];
    send_activity_new(context, &undo, &id, actor, inbox, true).await
  }
}

#[async_trait::async_trait(?Send)]
impl ActivityHandler for UndoDeletePrivateMessage {
  async fn verify(
    &self,
    context: &LemmyContext,
    request_counter: &mut i32,
  ) -> Result<(), LemmyError> {
    verify_activity(self)?;
    verify_person(&self.actor, context, request_counter).await?;
    verify_urls_match(&self.actor, self.object.actor())?;
    verify_domains_match(&self.actor, &self.object.object)?;
    self.object.verify(context, request_counter).await?;
    Ok(())
  }

  async fn receive(
    self,
    context: &LemmyContext,
    _request_counter: &mut i32,
  ) -> Result<(), LemmyError> {
    let ap_id = self.object.object.clone();
    let private_message = blocking(context.pool(), move |conn| {
      PrivateMessage::read_from_apub_id(conn, &ap_id.into())
    })
    .await??;

    let deleted_private_message = blocking(context.pool(), move |conn| {
      PrivateMessage::update_deleted(conn, private_message.id, false)
    })
    .await??;

    send_pm_ws_message(
      deleted_private_message.id,
      UserOperationCrud::EditPrivateMessage,
      None,
      context,
    )
    .await?;

    Ok(())
  }
}
