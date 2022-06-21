use crate::{local_instance, objects::person::ApubPerson};
use activitypub_federation::core::object_id::ObjectId;
use lemmy_api_common::utils::blocking;
use lemmy_db_schema::{
  newtypes::LocalUserId,
  source::{comment::Comment, post::Post},
  traits::Crud,
};
use lemmy_utils::{error::LemmyError, utils::scrape_text_for_mentions};
use lemmy_websocket::{send::send_local_notifs, LemmyContext};

pub mod comment;
pub mod post;
pub mod private_message;

#[tracing::instrument(skip_all)]
async fn get_comment_notif_recipients(
  actor: &ObjectId<ApubPerson>,
  comment: &Comment,
  do_send_email: bool,
  context: &LemmyContext,
  request_counter: &mut i32,
) -> Result<Vec<LocalUserId>, LemmyError> {
  let post_id = comment.post_id;
  let post = blocking(context.pool(), move |conn| Post::read(conn, post_id)).await??;
  let actor = actor
    .dereference(context, local_instance(context), request_counter)
    .await?;

  // Note:
  // Although mentions could be gotten from the post tags (they are included there), or the ccs,
  // Its much easier to scrape them from the comment body, since the API has to do that
  // anyway.
  // TODO: for compatibility with other projects, it would be much better to read this from cc or tags
  let mentions = scrape_text_for_mentions(&comment.content);
  send_local_notifs(mentions, comment, &*actor, &post, do_send_email, context).await
}
