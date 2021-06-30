use crate::activities::{
  comment::{get_notif_recipients, send_websocket_message},
  LemmyActivity,
};
use activitystreams::{activity::kind::CreateType, base::BaseExt};
use lemmy_apub::{check_is_apub_id_valid, objects::FromApub, ActorType, NoteExt};
use lemmy_apub_lib::{verify_domains_match, ActivityHandler, PublicUrl};
use lemmy_db_schema::source::{comment::Comment, person::Person};
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
}

#[async_trait::async_trait(?Send)]
impl ActivityHandler for LemmyActivity<CreateComment> {
  type Actor = Person;

  async fn verify(&self, _context: &LemmyContext) -> Result<(), LemmyError> {
    verify_domains_match(&self.actor, self.id_unchecked())?;
    self.inner.object.id(self.actor.as_str())?;
    check_is_apub_id_valid(&self.actor, false)
  }

  async fn receive(
    &self,
    actor: Self::Actor,
    context: &LemmyContext,
    request_counter: &mut i32,
  ) -> Result<(), LemmyError> {
    let comment = Comment::from_apub(
      &self.inner.object,
      context,
      actor.actor_id(),
      request_counter,
      false,
    )
    .await?;
    let recipients = get_notif_recipients(&self.actor, &comment, context, request_counter).await?;
    send_websocket_message(
      comment.id,
      recipients,
      UserOperationCrud::CreateComment,
      context,
    )
    .await
  }
}
