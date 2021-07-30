use crate::{
  activities::{
    comment::{get_notif_recipients, send_websocket_message},
    extract_community,
    verify_activity,
    verify_person_in_community,
  },
  objects::FromApub,
  ActorType,
  NoteExt,
};
use activitystreams::{activity::kind::CreateType, base::BaseExt};
use lemmy_apub_lib::{
  values::PublicUrl,
  verify_domains_match_opt,
  ActivityCommonFields,
  ActivityHandler,
};
use lemmy_db_schema::source::comment::Comment;
use lemmy_utils::LemmyError;
use lemmy_websocket::{LemmyContext, UserOperationCrud};
use url::Url;

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateComment {
  to: PublicUrl,
  object: NoteExt,
  cc: Vec<Url>,
  #[serde(rename = "type")]
  kind: CreateType,
  #[serde(flatten)]
  common: ActivityCommonFields,
}

#[async_trait::async_trait(?Send)]
impl ActivityHandler for CreateComment {
  async fn verify(
    &self,
    context: &LemmyContext,
    request_counter: &mut i32,
  ) -> Result<(), LemmyError> {
    let community = extract_community(&self.cc, context, request_counter).await?;

    verify_activity(self.common())?;
    verify_person_in_community(
      &self.common.actor,
      &community.actor_id(),
      context,
      request_counter,
    )
    .await?;
    verify_domains_match_opt(&self.common.actor, self.object.id_unchecked())?;
    // TODO: should add a check that the correct community is in cc (probably needs changes to
    //       comment deserialization)
    Ok(())
  }

  async fn receive(
    &self,
    context: &LemmyContext,
    request_counter: &mut i32,
  ) -> Result<(), LemmyError> {
    let comment = Comment::from_apub(
      &self.object,
      context,
      self.common.actor.clone(),
      request_counter,
      false,
    )
    .await?;
    let recipients =
      get_notif_recipients(&self.common.actor, &comment, context, request_counter).await?;
    send_websocket_message(
      comment.id,
      recipients,
      UserOperationCrud::CreateComment,
      context,
    )
    .await
  }

  fn common(&self) -> &ActivityCommonFields {
    &self.common
  }
}
