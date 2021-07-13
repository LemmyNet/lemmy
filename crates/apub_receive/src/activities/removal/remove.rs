use crate::activities::{
  comment::send_websocket_message as send_comment_message,
  community::send_websocket_message as send_community_message,
  post::send_websocket_message as send_post_message,
  verify_activity,
  verify_mod_action,
  verify_person_in_community,
};
use activitystreams::activity::kind::RemoveType;
use anyhow::anyhow;
use lemmy_api_common::blocking;
use lemmy_apub::{
  fetcher::{
    community::get_or_fetch_and_upsert_community,
    objects::get_or_fetch_and_insert_post_or_comment,
  },
  PostOrComment,
};
use lemmy_apub_lib::{ActivityCommonFields, ActivityHandler, PublicUrl};
use lemmy_db_queries::source::{comment::Comment_, community::Community_, post::Post_};
use lemmy_db_schema::source::{comment::Comment, community::Community, post::Post};
use lemmy_utils::LemmyError;
use lemmy_websocket::{LemmyContext, UserOperationCrud};
use url::Url;

// TODO: we can probably deduplicate a bunch of code between this and DeletePostCommentOrCommunity
#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RemovePostCommentOrCommunity {
  to: PublicUrl,
  pub(in crate::activities::removal) object: Url,
  cc: [Url; 1],
  #[serde(rename = "type")]
  kind: RemoveType,
  #[serde(flatten)]
  common: ActivityCommonFields,
}

#[async_trait::async_trait(?Send)]
impl ActivityHandler for RemovePostCommentOrCommunity {
  async fn verify(
    &self,
    context: &LemmyContext,
    request_counter: &mut i32,
  ) -> Result<(), LemmyError> {
    verify_activity(self.common())?;
    let object_community =
      get_or_fetch_and_upsert_community(&self.object, context, request_counter).await;
    // removing a community
    if object_community.is_ok() {
      verify_mod_action(&self.common.actor, self.object.clone(), context).await?;
    }
    // removing a post or comment
    else {
      verify_person_in_community(&self.common.actor, &self.cc, context, request_counter).await?;
      verify_mod_action(&self.common.actor, self.cc[0].clone(), context).await?;
    }
    Ok(())
  }

  async fn receive(
    &self,
    context: &LemmyContext,
    request_counter: &mut i32,
  ) -> Result<(), LemmyError> {
    let object_community =
      get_or_fetch_and_upsert_community(&self.object, context, request_counter).await;
    // removing a community
    if let Ok(community) = object_community {
      if community.local {
        return Err(anyhow!("Only local admin can remove community").into());
      }
      let deleted_community = blocking(context.pool(), move |conn| {
        Community::update_removed(conn, community.id, true)
      })
      .await??;

      send_community_message(
        deleted_community.id,
        UserOperationCrud::RemoveCommunity,
        context,
      )
      .await
    }
    // removing a post or comment
    else {
      match get_or_fetch_and_insert_post_or_comment(&self.object, context, request_counter).await? {
        PostOrComment::Post(post) => {
          let removed_post = blocking(context.pool(), move |conn| {
            Post::update_removed(conn, post.id, true)
          })
          .await??;
          send_post_message(removed_post.id, UserOperationCrud::EditPost, context).await
        }
        PostOrComment::Comment(comment) => {
          let removed_comment = blocking(context.pool(), move |conn| {
            Comment::update_removed(conn, comment.id, true)
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
  }

  fn common(&self) -> &ActivityCommonFields {
    &self.common
  }
}
