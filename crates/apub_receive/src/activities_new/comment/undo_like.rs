use crate::{
  activities_new::{
    comment::{like::LikeComment, undo_like_or_dislike_comment},
    post::like::LikePost,
  },
  inbox::new_inbox_routing::Activity,
};
use activitystreams::activity::kind::UndoType;
use lemmy_apub::check_is_apub_id_valid;
use lemmy_apub_lib::{verify_domains_match, PublicUrl, ReceiveActivity, VerifyActivity};
use lemmy_utils::LemmyError;
use lemmy_websocket::LemmyContext;
use url::Url;

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UndoLikeComment {
  actor: Url,
  to: PublicUrl,
  object: Activity<LikeComment>,
  cc: [Url; 1],
  #[serde(rename = "type")]
  kind: UndoType,
}

#[async_trait::async_trait(?Send)]
impl VerifyActivity for Activity<UndoLikeComment> {
  async fn verify(&self, context: &LemmyContext) -> Result<(), LemmyError> {
    verify_domains_match(&self.inner.actor, self.id_unchecked())?;
    verify_domains_match(&self.inner.actor, &self.inner.object.inner.object)?;
    check_is_apub_id_valid(&self.inner.actor, false)?;
    self.inner.object.verify(context).await
  }
}

#[async_trait::async_trait(?Send)]
impl ReceiveActivity for Activity<UndoLikeComment> {
  async fn receive(
    &self,
    context: &LemmyContext,
    request_counter: &mut i32,
  ) -> Result<(), LemmyError> {
    undo_like_or_dislike_comment(
      &self.inner.actor,
      &self.inner.object.inner.object,
      context,
      request_counter,
    )
    .await
  }
}
