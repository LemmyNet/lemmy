use crate::{
  activities_new::private_message::{delete::DeletePrivateMessage, send_websocket_message},
  inbox::new_inbox_routing::Activity,
};
use activitystreams::activity::kind::UndoType;
use lemmy_api_common::blocking;
use lemmy_apub_lib::{verify_domains_match, ReceiveActivity};
use lemmy_db_queries::{source::private_message::PrivateMessage_, ApubObject};
use lemmy_db_schema::source::private_message::PrivateMessage;
use lemmy_utils::LemmyError;
use lemmy_websocket::LemmyContext;
use url::Url;

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UndoDeletePrivateMessage {
  actor: Url,
  to: Url,
  object: Activity<DeletePrivateMessage>,
  #[serde(rename = "type")]
  kind: UndoType,
}

#[async_trait::async_trait(?Send)]
impl ReceiveActivity for Activity<UndoDeletePrivateMessage> {
  async fn receive(
    &self,
    context: &LemmyContext,
    _request_counter: &mut i32,
  ) -> Result<(), LemmyError> {
    verify_domains_match(&self.inner.actor, self.id_unchecked())?;
    verify_domains_match(&self.inner.actor, &self.inner.object.id_unchecked())?;
    verify_domains_match(&self.inner.actor, &self.inner.object.inner.actor)?;

    let ap_id = self.inner.object.inner.object.clone();
    let private_message = blocking(context.pool(), move |conn| {
      PrivateMessage::read_from_apub_id(conn, &ap_id.into())
    })
    .await??;

    let deleted_private_message = blocking(context.pool(), move |conn| {
      PrivateMessage::update_deleted(conn, private_message.id, false)
    })
    .await??;

    send_websocket_message(deleted_private_message.id, context).await?;

    Ok(())
  }
}
