use crate::activities::{
  comment::{remove::RemoveComment, send_websocket_message},
  verify_mod_action,
  LemmyActivity,
};
use activitystreams::activity::kind::UndoType;
use lemmy_api_common::blocking;
use lemmy_apub::{check_is_apub_id_valid, fetcher::objects::get_or_fetch_and_insert_comment};
use lemmy_apub_lib::{verify_domains_match, ActivityHandler, PublicUrl};
use lemmy_db_queries::source::comment::Comment_;
use lemmy_db_schema::source::{comment::Comment, person::Person};
use lemmy_utils::LemmyError;
use lemmy_websocket::{LemmyContext, UserOperationCrud};
use url::Url;

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UndoRemoveComment {
  to: PublicUrl,
  object: LemmyActivity<RemoveComment>,
  cc: [Url; 1],
  #[serde(rename = "type")]
  kind: UndoType,
}

#[async_trait::async_trait(?Send)]
impl ActivityHandler for LemmyActivity<UndoRemoveComment> {
  type Actor = Person;

  async fn verify(&self, context: &LemmyContext) -> Result<(), LemmyError> {
    verify_domains_match(&self.actor, self.id_unchecked())?;
    check_is_apub_id_valid(&self.actor, false)?;
    verify_mod_action(self.actor.clone(), self.inner.cc[0].clone(), context).await?;
    self.inner.object.verify(context).await
  }

  async fn receive(
    &self,
    _actor: Self::Actor,
    context: &LemmyContext,
    request_counter: &mut i32,
  ) -> Result<(), LemmyError> {
    let comment =
      get_or_fetch_and_insert_comment(&self.inner.object.inner.object, context, request_counter)
        .await?;

    let removed_comment = blocking(context.pool(), move |conn| {
      Comment::update_removed(conn, comment.id, false)
    })
    .await??;

    send_websocket_message(
      removed_comment.id,
      vec![],
      UserOperationCrud::EditComment,
      context,
    )
    .await
  }
}
