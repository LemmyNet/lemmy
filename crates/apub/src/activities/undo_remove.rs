use crate::{
  activities::{
    community::remove_mod::RemoveMod,
    deletion::{undo_delete::UndoDelete, verify_delete_activity},
    verify_activity,
  },
  fetcher::object_id::ObjectId,
};
use activitystreams::{
  activity::kind::UndoType,
  base::AnyBase,
  primitives::OneOrMany,
  unparsed::Unparsed,
};
use lemmy_apub_lib::{values::PublicUrl, ActivityFields, ActivityHandler, Data};
use lemmy_db_schema::source::{community::Community, person::Person};
use lemmy_utils::LemmyError;
use lemmy_websocket::LemmyContext;
use serde::{Deserialize, Serialize};
use url::Url;

#[derive(Clone, Debug, Deserialize, Serialize, ActivityFields)]
#[serde(rename_all = "camelCase")]
pub struct UndoRemovePostCommentOrCommunity {
  actor: ObjectId<Person>,
  to: [PublicUrl; 1],
  // Note, there is no such thing as Undo/Remove/Mod, so we ignore that
  object: RemoveMod,
  cc: [ObjectId<Community>; 1],
  #[serde(rename = "type")]
  kind: UndoType,
  id: Url,
  #[serde(rename = "@context")]
  context: OneOrMany<AnyBase>,
  #[serde(flatten)]
  unparsed: Unparsed,
}

#[async_trait::async_trait(?Send)]
impl ActivityHandler for UndoRemovePostCommentOrCommunity {
  type DataType = LemmyContext;
  async fn verify(
    &self,
    context: &Data<LemmyContext>,
    request_counter: &mut i32,
  ) -> Result<(), LemmyError> {
    verify_activity(self, &context.settings())?;
    self.object.verify(context, request_counter).await?;

    verify_delete_activity(
      self.object.object.inner(),
      self,
      &self.cc[0],
      true,
      context,
      request_counter,
    )
    .await?;
    Ok(())
  }

  async fn receive(
    self,
    context: &Data<LemmyContext>,
    _request_counter: &mut i32,
  ) -> Result<(), LemmyError> {
    UndoDelete::receive_undo_remove_action(self.object.object.inner(), context).await
  }
}
