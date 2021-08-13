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
use activitystreams::activity::kind::{DeleteType, UndoType};
use lemmy_api_common::blocking;
use lemmy_apub_lib::{
  verify_domains_match,
  verify_urls_match,
  ActivityCommonFields,
  ActivityHandler,
};
use lemmy_db_queries::{source::private_message::PrivateMessage_, ApubObject, Crud};
use lemmy_db_schema::source::{person::Person, private_message::PrivateMessage};
use lemmy_utils::LemmyError;
use lemmy_websocket::{send::send_pm_ws_message, LemmyContext, UserOperationCrud};
use url::Url;

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UndoDeletePrivateMessage {
  to: Url,
  object: DeletePrivateMessage,
  #[serde(rename = "type")]
  kind: UndoType,
  #[serde(flatten)]
  common: ActivityCommonFields,
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

    let object = DeletePrivateMessage {
      to: recipient.actor_id(),
      object: pm.ap_id.clone().into(),
      kind: DeleteType::Delete,
      common: ActivityCommonFields {
        context: lemmy_context(),
        id: generate_activity_id(DeleteType::Delete)?,
        actor: actor.actor_id(),
        unparsed: Default::default(),
      },
    };

    let id = generate_activity_id(UndoType::Undo)?;
    let undo = UndoDeletePrivateMessage {
      to: recipient.actor_id(),
      object,
      kind: UndoType::Undo,
      common: ActivityCommonFields {
        context: lemmy_context(),
        id: id.clone(),
        actor: actor.actor_id(),
        unparsed: Default::default(),
      },
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
    verify_activity(self.common())?;
    verify_person(&self.common.actor, context, request_counter).await?;
    verify_urls_match(&self.common.actor, &self.object.common.actor)?;
    verify_domains_match(&self.common.actor, &self.object.object)?;
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

  fn common(&self) -> &ActivityCommonFields {
    &self.common
  }
}
