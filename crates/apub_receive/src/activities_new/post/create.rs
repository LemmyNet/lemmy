use crate::{activities_new::post::send_websocket_message, inbox::new_inbox_routing::Activity};
use activitystreams::{activity::kind::CreateType, base::BaseExt};
use lemmy_apub::{
  check_is_apub_id_valid,
  fetcher::person::get_or_fetch_and_upsert_person,
  objects::FromApub,
  ActorType,
  PageExt,
};
use lemmy_apub_lib::{verify_domains_match, PublicUrl, ReceiveActivity, VerifyActivity};
use lemmy_db_schema::source::post::Post;
use lemmy_utils::LemmyError;
use lemmy_websocket::{LemmyContext, UserOperationCrud};
use url::Url;

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CreatePost {
  actor: Url,
  to: PublicUrl,
  object: PageExt,
  cc: Vec<Url>,
  #[serde(rename = "type")]
  kind: CreateType,
}

#[async_trait::async_trait(?Send)]
impl VerifyActivity for Activity<CreatePost> {
  async fn verify(&self, _context: &LemmyContext) -> Result<(), LemmyError> {
    verify_domains_match(self.id_unchecked(), &self.inner.actor)?;
    self.inner.object.id(self.inner.actor.as_str())?;
    check_is_apub_id_valid(&self.inner.actor, false)
  }
}

#[async_trait::async_trait(?Send)]
impl ReceiveActivity for Activity<CreatePost> {
  async fn receive(
    &self,
    context: &LemmyContext,
    request_counter: &mut i32,
  ) -> Result<(), LemmyError> {
    let person =
      get_or_fetch_and_upsert_person(&self.inner.actor, context, request_counter).await?;

    let post = Post::from_apub(
      &self.inner.object,
      context,
      person.actor_id(),
      request_counter,
      false,
    )
    .await?;

    send_websocket_message(post.id, UserOperationCrud::CreatePost, context).await
  }
}
