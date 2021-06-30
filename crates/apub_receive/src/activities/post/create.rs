use crate::activities::{post::send_websocket_message, LemmyActivity};
use activitystreams::{activity::kind::CreateType, base::BaseExt};
use lemmy_apub::{check_is_apub_id_valid, objects::FromApub, ActorType, PageExt};
use lemmy_apub_lib::{verify_domains_match, ActivityHandler, PublicUrl};
use lemmy_db_schema::source::{person::Person, post::Post};
use lemmy_utils::LemmyError;
use lemmy_websocket::{LemmyContext, UserOperationCrud};
use url::Url;

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CreatePost {
  to: PublicUrl,
  object: PageExt,
  cc: Vec<Url>,
  #[serde(rename = "type")]
  kind: CreateType,
}

#[async_trait::async_trait(?Send)]
impl ActivityHandler for LemmyActivity<CreatePost> {
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
    let post = Post::from_apub(
      &self.inner.object,
      context,
      actor.actor_id(),
      request_counter,
      false,
    )
    .await?;

    send_websocket_message(post.id, UserOperationCrud::CreatePost, context).await
  }
}
