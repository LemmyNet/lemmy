use crate::activities::{
  comment::send_websocket_message as send_comment_message,
  community::send_websocket_message as send_community_message,
  post::send_websocket_message as send_post_message,
  verify_activity,
  verify_mod_action,
  verify_person_in_community,
};
use activitystreams::activity::kind::DeleteType;
use lemmy_api_common::blocking;
use lemmy_apub::{
  fetcher::{
    community::get_or_fetch_and_upsert_community,
    objects::get_or_fetch_and_insert_post_or_comment,
    person::get_or_fetch_and_upsert_person,
  },
  ActorType,
  CommunityType,
  PostOrComment,
};
use lemmy_apub_lib::{verify_urls_match, ActivityCommonFields, ActivityHandler, PublicUrl};
use lemmy_db_queries::{
  source::{comment::Comment_, community::Community_, post::Post_},
  Crud,
};
use lemmy_db_schema::source::{comment::Comment, community::Community, person::Person, post::Post};
use lemmy_utils::LemmyError;
use lemmy_websocket::{LemmyContext, UserOperationCrud};
use url::Url;

/// This is very confusing, because there are four distinct cases to handle:
/// - user deletes their post
/// - user deletes their comment
/// - remote community mod deletes local community
/// - remote community deletes itself (triggered by a mod)
///
/// TODO: we should probably change how community deletions work to simplify this. Probably by
/// wrapping it in an announce just like other activities, instead of having the community send it.
#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DeletePostCommentOrCommunity {
  to: PublicUrl,
  pub(in crate::activities::deletion) object: Url,
  cc: [Url; 1],
  #[serde(rename = "type")]
  kind: DeleteType,
  #[serde(flatten)]
  common: ActivityCommonFields,
}

#[async_trait::async_trait(?Send)]
impl ActivityHandler for DeletePostCommentOrCommunity {
  async fn verify(
    &self,
    context: &LemmyContext,
    request_counter: &mut i32,
  ) -> Result<(), LemmyError> {
    verify_activity(self.common())?;
    let object_community =
      get_or_fetch_and_upsert_community(&self.object, context, request_counter).await;
    // deleting a community (set counter 0 to only fetch from local db)
    if object_community.is_ok() {
      verify_mod_action(&self.common.actor, self.object.clone(), context).await?;
    }
    // deleting a post or comment
    else {
      verify_person_in_community(&self.common().actor, &self.cc, context, request_counter).await?;
      let object_creator =
        get_post_or_comment_actor_id(&self.object, context, request_counter).await?;
      verify_urls_match(&self.common.actor, &object_creator)?;
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
    // deleting a community
    if let Ok(community) = object_community {
      if community.local {
        // repeat these checks just to be sure
        verify_person_in_community(&self.common().actor, &self.cc, context, request_counter)
          .await?;
        verify_mod_action(&self.common.actor, self.object.clone(), context).await?;
        let mod_ =
          get_or_fetch_and_upsert_person(&self.common.actor, context, request_counter).await?;
        community.send_delete(mod_, context).await?;
      }
      let deleted_community = blocking(context.pool(), move |conn| {
        Community::update_deleted(conn, community.id, true)
      })
      .await??;

      send_community_message(
        deleted_community.id,
        UserOperationCrud::DeleteCommunity,
        context,
      )
      .await
    }
    // deleting a post or comment
    else {
      match get_or_fetch_and_insert_post_or_comment(&self.object, context, request_counter).await? {
        PostOrComment::Post(post) => {
          let deleted_post = blocking(context.pool(), move |conn| {
            Post::update_deleted(conn, post.id, true)
          })
          .await??;
          send_post_message(deleted_post.id, UserOperationCrud::EditPost, context).await
        }
        PostOrComment::Comment(comment) => {
          let deleted_comment = blocking(context.pool(), move |conn| {
            Comment::update_deleted(conn, comment.id, true)
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

async fn get_post_or_comment_actor_id(
  object: &Url,
  context: &LemmyContext,
  request_counter: &mut i32,
) -> Result<Url, LemmyError> {
  let actor_id =
    match get_or_fetch_and_insert_post_or_comment(object, context, request_counter).await? {
      PostOrComment::Post(post) => {
        let creator_id = post.creator_id;
        blocking(context.pool(), move |conn| Person::read(conn, creator_id))
          .await??
          .actor_id()
      }
      PostOrComment::Comment(comment) => {
        let creator_id = comment.creator_id;
        blocking(context.pool(), move |conn| Person::read(conn, creator_id))
          .await??
          .actor_id()
      }
    };
  Ok(actor_id)
}
