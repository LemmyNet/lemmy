use crate::{
  activities::{generate_activity_id, verify_activity, verify_person},
  activity_queue::send_activity_new,
  extensions::context::lemmy_context,
  ActorType,
};
use activitystreams::activity::kind::DeleteType;
use lemmy_api_common::blocking;
use lemmy_apub_lib::{verify_domains_match, ActivityCommonFields, ActivityHandler};
use lemmy_db_queries::{source::private_message::PrivateMessage_, ApubObject, Crud};
use lemmy_db_schema::source::{person::Person, private_message::PrivateMessage};
use lemmy_utils::LemmyError;
use lemmy_websocket::{send::send_pm_ws_message, LemmyContext, UserOperationCrud};
use url::Url;

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DeletePrivateMessage {
  pub(in crate::activities::private_message) to: Url,
  pub(in crate::activities::private_message) object: Url,
  #[serde(rename = "type")]
  pub(in crate::activities::private_message) kind: DeleteType,
  #[serde(flatten)]
  pub(in crate::activities::private_message) common: ActivityCommonFields,
}

impl DeletePrivateMessage {
  pub async fn send(
    actor: &Person,
    pm: &PrivateMessage,
    context: &LemmyContext,
  ) -> Result<(), LemmyError> {
    let recipient_id = pm.recipient_id;
    let recipient =
      blocking(context.pool(), move |conn| Person::read(conn, recipient_id)).await??;

    let id = generate_activity_id(DeleteType::Delete)?;
    let delete = DeletePrivateMessage {
      to: actor.actor_id(),
      object: pm.ap_id.clone().into(),
      kind: DeleteType::Delete,
      common: ActivityCommonFields {
        context: lemmy_context(),
        id: id.clone(),
        actor: actor.actor_id(),
        unparsed: Default::default(),
      },
    };
    let inbox = vec![recipient.get_shared_inbox_or_inbox_url()];
    send_activity_new(context, &delete, &id, actor, inbox, true).await
  }
}

#[async_trait::async_trait(?Send)]
impl ActivityHandler for DeletePrivateMessage {
  async fn verify(
    &self,
    context: &LemmyContext,
    request_counter: &mut i32,
  ) -> Result<(), LemmyError> {
    verify_activity(self.common())?;
    verify_person(&self.common.actor, context, request_counter).await?;
    verify_domains_match(&self.common.actor, &self.object)?;
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

  fn common(&self) -> &ActivityCommonFields {
    &self.common
  }
}
