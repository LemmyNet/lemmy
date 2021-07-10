use crate::activities::{
  post_or_comment::voting::receive_like_or_dislike,
  verify_activity,
  verify_person_in_community,
};
use activitystreams::activity::kind::DislikeType;
use lemmy_apub_lib::{ActivityCommonFields, ActivityHandlerNew, PublicUrl};
use lemmy_utils::LemmyError;
use lemmy_websocket::LemmyContext;
use url::Url;

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DislikePostOrComment {
  to: PublicUrl,
  pub(in crate::activities) object: Url,
  cc: [Url; 1],
  #[serde(rename = "type")]
  kind: DislikeType,
  #[serde(flatten)]
  common: ActivityCommonFields,
}

#[async_trait::async_trait(?Send)]
impl ActivityHandlerNew for DislikePostOrComment {
  async fn verify(
    &self,
    context: &LemmyContext,
    request_counter: &mut i32,
  ) -> Result<(), LemmyError> {
    verify_activity(self.common())?;
    verify_person_in_community(&self.common.actor, &self.cc, context, request_counter).await?;
    Ok(())
  }

  async fn receive(
    &self,
    context: &LemmyContext,
    request_counter: &mut i32,
  ) -> Result<(), LemmyError> {
    receive_like_or_dislike(
      -1,
      &self.common.actor,
      &self.object,
      context,
      request_counter,
    )
    .await
  }

  fn common(&self) -> &ActivityCommonFields {
    &self.common
  }
}
