use crate::activities::{
  comment::send_websocket_message as send_comment_message,
  post::send_websocket_message as send_post_message,
  verify_activity,
  verify_person_in_community,
};
use activitystreams::activity::kind::DeleteType;
use lemmy_api_common::blocking;
use lemmy_apub::{
  fetcher::objects::get_or_fetch_and_insert_post_or_comment,
  ActorType,
  PostOrComment,
};
use lemmy_apub_lib::{verify_urls_match, ActivityCommonFields, ActivityHandlerNew, PublicUrl};
use lemmy_db_queries::{
  source::{comment::Comment_, post::Post_},
  Crud,
};
use lemmy_db_schema::source::{comment::Comment, person::Person, post::Post};
use lemmy_utils::LemmyError;
use lemmy_websocket::{LemmyContext, UserOperationCrud};
use url::Url;

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DeletePostOrComment {
  to: PublicUrl,
  pub(in crate::activities::post_or_comment) object: Url,
  cc: [Url; 1],
  #[serde(rename = "type")]
  kind: DeleteType,
  #[serde(flatten)]
  common: ActivityCommonFields,
}

#[async_trait::async_trait(?Send)]
impl ActivityHandlerNew for DeletePostOrComment {
  async fn verify(
    &self,
    context: &LemmyContext,
    request_counter: &mut i32,
  ) -> Result<(), LemmyError> {
    verify_activity(self.common())?;
    verify_person_in_community(&self.common().actor, &self.cc, context, request_counter).await?;
    let object_creator =
      get_post_or_comment_actor_id(&self.object, context, request_counter).await?;
    verify_urls_match(&self.common.actor, &object_creator)?;
    Ok(())
  }

  async fn receive(
    &self,
    context: &LemmyContext,
    request_counter: &mut i32,
  ) -> Result<(), LemmyError> {
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
