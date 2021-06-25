use crate::{
  activities_new::private_message::send_websocket_message,
  inbox::new_inbox_routing::Activity,
};
use activitystreams::{activity::kind::UpdateType, base::BaseExt};
use lemmy_apub::{check_is_apub_id_valid, objects::FromApub, NoteExt};
use lemmy_apub_lib::{verify_domains_match, ReceiveActivity, VerifyActivity};
use lemmy_db_schema::source::private_message::PrivateMessage;
use lemmy_utils::LemmyError;
use lemmy_websocket::{LemmyContext, UserOperationCrud};
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
impl VerifyActivity for Activity<UpdatePrivateMessage> {
  async fn verify(&self, _context: &LemmyContext) -> Result<(), LemmyError> {
    verify_domains_match(self.id_unchecked(), &self.inner.actor)?;
    self.inner.object.id(self.inner.actor.as_str())?;
    check_is_apub_id_valid(&self.inner.actor, false)
  }
}

#[async_trait::async_trait(?Send)]
impl ReceiveActivity for Activity<UpdatePrivateMessage> {
  async fn receive(
    &self,
    context: &LemmyContext,
    request_counter: &mut i32,
  ) -> Result<(), LemmyError> {
    let private_message = PrivateMessage::from_apub(
      &self.inner.object,
      context,
      self.inner.actor.clone(),
      request_counter,
      false,
    )
    .await?;

    send_websocket_message(
      private_message.id,
      UserOperationCrud::EditPrivateMessage,
      context,
    )
    .await?;

    Ok(())
  }
}
