use crate::{
  activities_new::private_message::send_websocket_message,
  inbox::new_inbox_routing::Activity,
};
use activitystreams::activity::kind::UpdateType;
use lemmy_apub::{objects::FromApub, NoteExt};
use lemmy_apub_lib::{verify_domains_match, ReceiveActivity};
use lemmy_db_schema::source::private_message::PrivateMessage;
use lemmy_utils::LemmyError;
use lemmy_websocket::LemmyContext;
use url::Url;

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdatePrivateMessage {
  actor: Url,
  to: Url,
  object: NoteExt,
  #[serde(rename = "type")]
  kind: UpdateType,
}

#[async_trait::async_trait(?Send)]
impl ReceiveActivity for Activity<UpdatePrivateMessage> {
  async fn receive(
    &self,
    context: &LemmyContext,
    request_counter: &mut i32,
  ) -> Result<(), LemmyError> {
    verify_domains_match(&self.inner.actor, self.id_unchecked())?;

    let private_message = PrivateMessage::from_apub(
      &self.inner.object,
      context,
      self.inner.actor.clone(),
      request_counter,
      false,
    )
    .await?;

    send_websocket_message(private_message.id, context).await?;

    Ok(())
  }
}
