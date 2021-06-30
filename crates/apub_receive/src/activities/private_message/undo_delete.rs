use crate::activities::{
  private_message::{delete::DeletePrivateMessage, send_websocket_message},
  LemmyActivity,
};
use activitystreams::activity::kind::UndoType;
use lemmy_api_common::blocking;
use lemmy_apub::check_is_apub_id_valid;
use lemmy_apub_lib::{verify_domains_match, ActivityHandler};
use lemmy_db_queries::{source::private_message::PrivateMessage_, ApubObject};
use lemmy_db_schema::source::{person::Person, private_message::PrivateMessage};
use lemmy_utils::LemmyError;
use lemmy_websocket::{LemmyContext, UserOperationCrud};
use url::Url;

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UndoDeletePrivateMessage {
  to: Url,
  object: LemmyActivity<DeletePrivateMessage>,
  #[serde(rename = "type")]
  kind: UndoType,
}

#[async_trait::async_trait(?Send)]
impl ActivityHandler for LemmyActivity<UndoDeletePrivateMessage> {
  type Actor = Person;

  async fn verify(&self, context: &LemmyContext) -> Result<(), LemmyError> {
    verify_domains_match(&self.actor, self.id_unchecked())?;
    verify_domains_match(&self.actor, &self.inner.object.id_unchecked())?;
    check_is_apub_id_valid(&self.actor, false)?;
    self.inner.object.verify(context).await
  }

  async fn receive(
    &self,
    _actor: Self::Actor,
    context: &LemmyContext,
    _request_counter: &mut i32,
  ) -> Result<(), LemmyError> {
    let ap_id = self.inner.object.inner.object.clone();
    let private_message = blocking(context.pool(), move |conn| {
      PrivateMessage::read_from_apub_id(conn, &ap_id.into())
    })
    .await??;

    let deleted_private_message = blocking(context.pool(), move |conn| {
      PrivateMessage::update_deleted(conn, private_message.id, false)
    })
    .await??;

    send_websocket_message(
      deleted_private_message.id,
      UserOperationCrud::EditPrivateMessage,
      context,
    )
    .await?;

    Ok(())
  }
}
