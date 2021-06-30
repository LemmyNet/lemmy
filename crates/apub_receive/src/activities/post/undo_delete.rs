use crate::activities::{
  post::{delete::DeletePost, send_websocket_message},
  LemmyActivity,
};
use activitystreams::activity::kind::UndoType;
use lemmy_api_common::blocking;
use lemmy_apub::{check_is_apub_id_valid, fetcher::objects::get_or_fetch_and_insert_post};
use lemmy_apub_lib::{verify_domains_match, ActivityHandler, PublicUrl};
use lemmy_db_queries::source::post::Post_;
use lemmy_db_schema::source::{person::Person, post::Post};
use lemmy_utils::LemmyError;
use lemmy_websocket::{LemmyContext, UserOperationCrud};
use url::Url;

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UndoDeletePost {
  to: PublicUrl,
  object: LemmyActivity<DeletePost>,
  cc: [Url; 1],
  #[serde(rename = "type")]
  kind: UndoType,
}

#[async_trait::async_trait(?Send)]
impl ActivityHandler for LemmyActivity<UndoDeletePost> {
  type Actor = Person;

  async fn verify(&self, context: &LemmyContext) -> Result<(), LemmyError> {
    verify_domains_match(&self.actor, self.id_unchecked())?;
    verify_domains_match(&self.actor, &self.inner.object.actor)?;
    check_is_apub_id_valid(&self.actor, false)?;
    self.inner.object.verify(context).await
  }

  async fn receive(
    &self,
    _actor: Self::Actor,
    context: &LemmyContext,
    request_counter: &mut i32,
  ) -> Result<(), LemmyError> {
    let post =
      get_or_fetch_and_insert_post(&self.inner.object.inner.object, context, request_counter)
        .await?;

    let deleted_post = blocking(context.pool(), move |conn| {
      Post::update_deleted(conn, post.id, false)
    })
    .await??;

    send_websocket_message(deleted_post.id, UserOperationCrud::EditPost, context).await
  }
}
