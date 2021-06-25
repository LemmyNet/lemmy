use crate::{activities_new::post::send_websocket_message, inbox::new_inbox_routing::Activity};
use activitystreams::activity::kind::CreateType;
use lemmy_apub::{
  fetcher::person::get_or_fetch_and_upsert_person,
  objects::FromApub,
  ActorType,
  PageExt,
};
use lemmy_apub_lib::{PublicUrl, ReceiveActivity};
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
  #[serde(rename = "type")]
  kind: CreateType,
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
