use crate::{activities_new::comment::like_or_dislike_comment, inbox::new_inbox_routing::Activity};
use activitystreams::activity::kind::LikeType;
use lemmy_apub::check_is_apub_id_valid;
use lemmy_apub_lib::{verify_domains_match, PublicUrl, ReceiveActivity, VerifyActivity};
use lemmy_utils::LemmyError;
use lemmy_websocket::LemmyContext;
use url::Url;

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LikeComment {
  actor: Url,
  to: PublicUrl,
  object: Url,
  cc: [Url; 1],
  #[serde(rename = "type")]
  kind: LikeType,
}

#[async_trait::async_trait(?Send)]
impl VerifyActivity for Activity<LikeComment> {
  async fn verify(&self, _context: &LemmyContext) -> Result<(), LemmyError> {
    verify_domains_match(&self.inner.actor, self.id_unchecked())?;
    check_is_apub_id_valid(&self.inner.actor, false)
  }
}

#[async_trait::async_trait(?Send)]
impl ReceiveActivity for Activity<LikeComment> {
  async fn receive(
    &self,
    context: &LemmyContext,
    request_counter: &mut i32,
  ) -> Result<(), LemmyError> {
    like_or_dislike_comment(
      -1,
      &self.inner.actor,
      &self.inner.object,
      context,
      request_counter,
    )
    .await
  }
}
