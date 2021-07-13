use crate::activities::{
  comment::send_websocket_message as send_comment_message,
  community::send_websocket_message as send_community_message,
  deletion::delete::DeletePostCommentOrCommunity,
  post::send_websocket_message as send_post_message,
  verify_activity,
  verify_mod_action,
  verify_person_in_community,
};
use activitystreams::activity::kind::UndoType;
use lemmy_api_common::blocking;
use lemmy_apub::{
  fetcher::{
    community::get_or_fetch_and_upsert_community,
    objects::get_or_fetch_and_insert_post_or_comment,
    person::get_or_fetch_and_upsert_person,
  },
  CommunityType,
  PostOrComment,
};
use lemmy_apub_lib::{verify_urls_match, ActivityCommonFields, ActivityHandler, PublicUrl};
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
    let object_community =
      get_or_fetch_and_upsert_community(&self.object.object, context, request_counter).await;
    // restoring a community
    if object_community.is_ok() {
      verify_mod_action(&self.common.actor, self.object.object.clone(), context).await?;
    }
    // restoring a post or comment
    else {
      verify_person_in_community(&self.common().actor, &self.cc, context, request_counter).await?;
      verify_urls_match(&self.common.actor, &self.object.common().actor)?;
    }
    Ok(())
  }

  async fn receive(
    &self,
    context: &LemmyContext,
    request_counter: &mut i32,
  ) -> Result<(), LemmyError> {
    let object_community =
      get_or_fetch_and_upsert_community(&self.object.object, context, request_counter).await;
    // restoring a community
    if let Ok(community) = object_community {
      if community.local {
        // repeat these checks just to be sure
        verify_person_in_community(&self.common().actor, &self.cc, context, request_counter)
          .await?;
        verify_mod_action(&self.common.actor, self.object.object.clone(), context).await?;
        let mod_ =
          get_or_fetch_and_upsert_person(&self.common.actor, context, request_counter).await?;
        community.send_undo_delete(mod_, context).await?;
      }
      let deleted_community = blocking(context.pool(), move |conn| {
        Community::update_deleted(conn, community.id, false)
      })
      .await??;

      send_community_message(
        deleted_community.id,
        UserOperationCrud::EditCommunity,
        context,
      )
      .await
    }
    // restoring a post or comment
    else {
      match get_or_fetch_and_insert_post_or_comment(&self.object.object, context, request_counter)
        .await?
      {
        PostOrComment::Post(post) => {
          let deleted_post = blocking(context.pool(), move |conn| {
            Post::update_deleted(conn, post.id, false)
          })
          .await??;
          send_post_message(deleted_post.id, UserOperationCrud::EditPost, context).await
        }
        PostOrComment::Comment(comment) => {
          let deleted_comment = blocking(context.pool(), move |conn| {
            Comment::update_deleted(conn, comment.id, false)
          })
          .await??;
          send_comment_message(
            deleted_comment.id,
            vec![],
            UserOperationCrud::EditComment,
            context,
          )
          .await
        }
      }
    }
  }

  fn common(&self) -> &ActivityCommonFields {
    &self.common
  }
}
