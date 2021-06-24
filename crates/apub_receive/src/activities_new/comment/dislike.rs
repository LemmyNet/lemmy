use crate::{activities_new::comment::send_websocket_message, inbox::new_inbox_routing::Activity};
use activitystreams::activity::kind::DislikeType;
use lemmy_api_common::blocking;
use lemmy_apub::fetcher::{
  objects::get_or_fetch_and_insert_comment,
  person::get_or_fetch_and_upsert_person,
};
use lemmy_apub_lib::{PublicUrl, ReceiveActivity};
use lemmy_db_queries::Likeable;
use lemmy_db_schema::source::comment::{CommentLike, CommentLikeForm};
use lemmy_utils::LemmyError;
use lemmy_websocket::{LemmyContext, UserOperation};
use url::Url;

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DislikeComment {
  actor: Url,
  to: PublicUrl,
  object: Url,
  cc: Vec<Url>,
  #[serde(rename = "type")]
  kind: DislikeType,
}

#[async_trait::async_trait(?Send)]
impl ReceiveActivity for Activity<DislikeComment> {
  async fn receive(
    &self,
    context: &LemmyContext,
    request_counter: &mut i32,
  ) -> Result<(), LemmyError> {
    let person =
      get_or_fetch_and_upsert_person(&self.inner.actor, context, request_counter).await?;
    let comment =
      get_or_fetch_and_insert_comment(&self.inner.object, context, request_counter).await?;

    let comment_id = comment.id;
    let like_form = CommentLikeForm {
      comment_id,
      post_id: comment.post_id,
      person_id: person.id,
      score: -1,
    };
    let person_id = person.id;
    blocking(context.pool(), move |conn| {
      CommentLike::remove(conn, person_id, comment_id)?;
      CommentLike::like(conn, &like_form)
    })
    .await??;

    // TODO get those recipient actor ids from somewhere
    send_websocket_message(
      comment_id,
      vec![],
      UserOperation::CreateCommentLike,
      context,
    )
    .await
  }
}
