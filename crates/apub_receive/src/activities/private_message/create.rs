use crate::{
  activities::private_message::send_websocket_message,
  inbox::new_inbox_routing::Activity,
};
use activitystreams::{activity::kind::CreateType, base::BaseExt};
use lemmy_apub::{check_is_apub_id_valid, objects::FromApub, ActorType, NoteExt};
use lemmy_apub_lib::{verify_domains_match, ActivityHandler};
use lemmy_db_schema::source::{person::Person, private_message::PrivateMessage};
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
}

#[async_trait::async_trait(?Send)]
impl ActivityHandler for Activity<CreatePrivateMessage> {
  type Actor = Person;

  async fn verify(&self, _context: &LemmyContext) -> Result<(), LemmyError> {
    verify_domains_match(self.id_unchecked(), &self.actor)?;
    self.inner.object.id(self.actor.as_str())?;
    check_is_apub_id_valid(&self.actor, false)
  }

  async fn receive(
    &self,
    actor: Self::Actor,
    context: &LemmyContext,
    request_counter: &mut i32,
  ) -> Result<(), LemmyError> {
    let private_message = PrivateMessage::from_apub(
      &self.inner.object,
      context,
      actor.actor_id(),
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
}
