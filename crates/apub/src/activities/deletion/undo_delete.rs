use crate::{
  activities::{
    comment::send_websocket_message as send_comment_message,
    community::send_websocket_message as send_community_message,
    deletion::{delete::DeletePostCommentOrCommunity, verify_delete_activity, DeletableObjects},
    post::send_websocket_message as send_post_message,
    verify_activity,
  },
  fetcher::person::get_or_fetch_and_upsert_person,
  CommunityType,
};
use activitystreams::activity::kind::UndoType;
use lemmy_api_common::blocking;
use lemmy_apub_lib::{values::PublicUrl, ActivityCommonFields, ActivityHandler};
use lemmy_db_queries::source::{comment::Comment_, community::Community_, post::Post_};
use lemmy_db_schema::source::{comment::Comment, community::Community, post::Post};
use lemmy_utils::LemmyError;
use lemmy_websocket::{LemmyContext, UserOperationCrud};
use url::Url;

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UndoDeletePostCommentOrCommunity {
  to: PublicUrl,
  object: DeletePostCommentOrCommunity,
  cc: [Url; 1],
  #[serde(rename = "type")]
  kind: UndoType,
  #[serde(flatten)]
  common: ActivityCommonFields,
}

#[async_trait::async_trait(?Send)]
impl ActivityHandler for UndoDeletePostCommentOrCommunity {
  async fn verify(
    &self,
    context: &LemmyContext,
    request_counter: &mut i32,
  ) -> Result<(), LemmyError> {
    verify_activity(self.common())?;
    self.object.verify(context, request_counter).await?;
    verify_delete_activity(
      &self.object.object,
      &self.cc[0],
      &self.common,
      context,
      request_counter,
    )
    .await?;
    Ok(())
  }

  async fn receive(
    self,
    context: &LemmyContext,
    request_counter: &mut i32,
  ) -> Result<(), LemmyError> {
    use UserOperationCrud::*;
    let object = DeletableObjects::read_from_db(&self.object.object, context).await?;
    match object {
      DeletableObjects::Community(community) => {
        if community.local {
          let mod_ =
            get_or_fetch_and_upsert_person(&self.common.actor, context, request_counter).await?;
          community.send_undo_delete(mod_, context).await?;
        }

        let deleted_community = blocking(context.pool(), move |conn| {
          Community::update_deleted(conn, community.id, false)
        })
        .await??;
        send_community_message(deleted_community.id, EditCommunity, context).await
      }
      DeletableObjects::Post(post) => {
        let deleted_post = blocking(context.pool(), move |conn| {
          Post::update_deleted(conn, post.id, false)
        })
        .await??;
        send_post_message(deleted_post.id, EditPost, context).await
      }
      DeletableObjects::Comment(comment) => {
        let deleted_comment = blocking(context.pool(), move |conn| {
          Comment::update_deleted(conn, comment.id, false)
        })
        .await??;
        send_comment_message(deleted_comment.id, vec![], EditComment, context).await
      }
    }
  }

  fn common(&self) -> &ActivityCommonFields {
    &self.common
  }
}
