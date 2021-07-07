use crate::activities::private_message::send_websocket_message;
use activitystreams::{activity::kind::CreateType, base::BaseExt};
use lemmy_apub::{check_is_apub_id_valid, objects::FromApub, NoteExt};
use lemmy_apub_lib::{verify_domains_match, ActivityCommonFields, ActivityHandlerNew};
use lemmy_db_schema::source::private_message::PrivateMessage;
use lemmy_utils::LemmyError;
use lemmy_websocket::{LemmyContext, UserOperationCrud};
use url::Url;

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CreatePrivateMessage {
  to: Url,
  object: NoteExt,
  #[serde(rename = "type")]
  kind: CreateType,
  #[serde(flatten)]
  common: ActivityCommonFields,
}

#[async_trait::async_trait(?Send)]
impl ActivityHandlerNew for CreatePrivateMessage {
  async fn verify(&self, _context: &LemmyContext, _: &mut i32) -> Result<(), LemmyError> {
    verify_domains_match(self.common.id_unchecked(), &self.common.actor)?;
    self.object.id(self.common.actor.as_str())?;
    check_is_apub_id_valid(&self.common.actor, false)
  }

  async fn receive(
    &self,
    context: &LemmyContext,
    request_counter: &mut i32,
  ) -> Result<(), LemmyError> {
    let private_message = PrivateMessage::from_apub(
      &self.object,
      context,
      self.common.actor.clone(),
      request_counter,
      false,
    )
    .await?;

    send_websocket_message(
      private_message.id,
      UserOperationCrud::CreatePrivateMessage,
      context,
    )
    .await?;

    Ok(())
  }

  fn common(&self) -> &ActivityCommonFields {
    &self.common
  }
}
