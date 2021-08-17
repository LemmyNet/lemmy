use crate::activities::{
  deletion::{undo_delete::UndoDelete, verify_delete_activity},
  removal::remove::RemoveMod,
  verify_activity,
};
use activitystreams::activity::kind::UndoType;
use lemmy_apub_lib::{values::PublicUrl, ActivityCommonFields, ActivityHandler};
use lemmy_utils::LemmyError;
use lemmy_websocket::LemmyContext;
use url::Url;

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UndoRemovePostCommentOrCommunity {
  to: PublicUrl,
  // Note, there is no such thing as Undo/Remove/Mod, so we ignore that
  object: RemoveMod,
  cc: [Url; 1],
  #[serde(rename = "type")]
  kind: UndoType,
  #[serde(flatten)]
  common: ActivityCommonFields,
}

#[async_trait::async_trait(?Send)]
impl ActivityHandler for UndoRemovePostCommentOrCommunity {
  async fn verify(
    &self,
    context: &LemmyContext,
    request_counter: &mut i32,
  ) -> Result<(), LemmyError> {
    verify_activity(self.common())?;
    self.object.verify(context, request_counter).await?;

    verify_delete_activity(
      &self.object.object,
      &self.cc[0],
      self.common(),
      true,
      context,
      request_counter,
    )
    .await?;
    Ok(())
  }

  async fn receive(
    self,
    context: &LemmyContext,
    _request_counter: &mut i32,
  ) -> Result<(), LemmyError> {
    UndoDelete::receive_undo_remove_action(&self.object.object, context).await
  }

  fn common(&self) -> &ActivityCommonFields {
    &self.common
  }
}
