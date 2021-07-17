use crate::activities::{
  verify_activity,
  verify_person_in_community,
  voting::{like::LikePostOrComment, receive_undo_like_or_dislike},
};
use activitystreams::activity::kind::UndoType;
use lemmy_apub_lib::{verify_urls_match, ActivityCommonFields, ActivityHandler, PublicUrl};
use lemmy_utils::LemmyError;
use lemmy_websocket::LemmyContext;
use url::Url;

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UndoLikePostOrComment {
  to: PublicUrl,
  object: LikePostOrComment,
  cc: [Url; 1],
  #[serde(rename = "type")]
  kind: UndoType,
  #[serde(flatten)]
  common: ActivityCommonFields,
}

#[async_trait::async_trait(?Send)]
impl ActivityHandler for UndoLikePostOrComment {
  async fn verify(
    &self,
    context: &LemmyContext,
    request_counter: &mut i32,
  ) -> Result<(), LemmyError> {
    verify_activity(self.common())?;
    verify_person_in_community(&self.common.actor, &self.cc, context, request_counter).await?;
    verify_urls_match(&self.common.actor, &self.object.common().actor)?;
    self.object.verify(context, request_counter).await?;
    Ok(())
  }

  async fn receive(
    &self,
    context: &LemmyContext,
    request_counter: &mut i32,
  ) -> Result<(), LemmyError> {
    receive_undo_like_or_dislike(
      &self.common.actor,
      &self.object.object,
      context,
      request_counter,
    )
    .await
  }

  fn common(&self) -> &ActivityCommonFields {
    &self.common
  }
}
