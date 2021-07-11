use crate::activities::{
  comment::send_websocket_message as send_comment_message,
  post::send_websocket_message as send_post_message,
  post_or_comment::remove::RemovePostOrComment,
  verify_activity,
  verify_mod_action,
  verify_person_in_community,
};
use activitystreams::activity::kind::UndoType;
use lemmy_api_common::blocking;
use lemmy_apub::{fetcher::objects::get_or_fetch_and_insert_post_or_comment, PostOrComment};
use lemmy_apub_lib::{ActivityCommonFields, ActivityHandlerNew, PublicUrl};
use lemmy_db_queries::source::{comment::Comment_, post::Post_};
use lemmy_db_schema::source::{comment::Comment, post::Post};
use lemmy_utils::LemmyError;
use lemmy_websocket::{LemmyContext, UserOperationCrud};
use url::Url;

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UndoRemovePostOrComment {
  to: PublicUrl,
  object: RemovePostOrComment,
  cc: [Url; 1],
  #[serde(rename = "type")]
  kind: UndoType,
  #[serde(flatten)]
  common: ActivityCommonFields,
}

#[async_trait::async_trait(?Send)]
impl ActivityHandlerNew for UndoRemovePostOrComment {
  async fn verify(
    &self,
    context: &LemmyContext,
    request_counter: &mut i32,
  ) -> Result<(), LemmyError> {
    verify_activity(self.common())?;
    verify_person_in_community(&self.common().actor, &self.cc, context, request_counter).await?;
    verify_mod_action(&self.common.actor, self.cc[0].clone(), context).await?;
    self.object.verify(context, request_counter).await?;
    // dont check that actor and object.actor are identical, so that one mod can
    // undo the action of another
    Ok(())
  }

  async fn receive(
    &self,
    context: &LemmyContext,
    request_counter: &mut i32,
  ) -> Result<(), LemmyError> {
    match get_or_fetch_and_insert_post_or_comment(&self.object.object, context, request_counter)
      .await?
    {
      PostOrComment::Post(post) => {
        let removed_post = blocking(context.pool(), move |conn| {
          Post::update_removed(conn, post.id, false)
        })
        .await??;
        send_post_message(removed_post.id, UserOperationCrud::EditPost, context).await
      }
      PostOrComment::Comment(comment) => {
        let removed_comment = blocking(context.pool(), move |conn| {
          Comment::update_removed(conn, comment.id, false)
        })
        .await??;
        send_comment_message(
          removed_comment.id,
          vec![],
          UserOperationCrud::EditComment,
          context,
        )
        .await
      }
    }
  }

  fn common(&self) -> &ActivityCommonFields {
    &self.common
  }
}
