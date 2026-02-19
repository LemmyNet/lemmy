use crate::{
  objects::{
    PostOrComment,
    comment::ApubComment,
    community::ApubCommunity,
    person::ApubPerson,
    post::ApubPost,
  },
  protocol::{page::Attachment, tags::ApubTag},
  utils::protocol::{InCommunity, LanguageTag, Source},
};
use activitypub_federation::{
  config::Data,
  fetch::object_id::ObjectId,
  kinds::object::NoteType,
  protocol::{
    helpers::{deserialize_one_or_many, deserialize_skip_error},
    values::MediaTypeMarkdownOrHtml,
  },
};
use chrono::{DateTime, Utc};
use lemmy_api_utils::context::LemmyContext;
use lemmy_db_schema::source::{community::Community, post::Post};
use lemmy_diesel_utils::traits::Crud;
use lemmy_utils::{
  MAX_COMMENT_DEPTH_LIMIT,
  error::{LemmyErrorType, LemmyResult},
};
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
use url::Url;

#[skip_serializing_none]
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Note {
  pub(crate) r#type: NoteType,
  pub id: ObjectId<ApubComment>,
  pub attributed_to: ObjectId<ApubPerson>,
  #[serde(deserialize_with = "deserialize_one_or_many")]
  pub(crate) to: Vec<Url>,
  #[serde(deserialize_with = "deserialize_one_or_many", default)]
  pub cc: Vec<Url>,
  pub(crate) content: String,
  pub(crate) in_reply_to: ObjectId<PostOrComment>,

  pub(crate) media_type: Option<MediaTypeMarkdownOrHtml>,
  #[serde(deserialize_with = "deserialize_skip_error", default)]
  pub(crate) source: Option<Source>,
  pub(crate) published: Option<DateTime<Utc>>,
  pub(crate) updated: Option<DateTime<Utc>>,
  #[serde(default)]
  pub tag: Vec<ApubTag>,
  // lemmy extension
  pub distinguished: Option<bool>,
  pub(crate) language: Option<LanguageTag>,
  pub(crate) audience: Option<ObjectId<ApubCommunity>>,
  #[serde(default)]
  pub(crate) attachment: Vec<Attachment>,
  pub(crate) context: Option<String>,
}

impl Note {
  pub async fn get_parents(
    &self,
    context: &Data<LemmyContext>,
  ) -> LemmyResult<(ApubPost, Option<ApubComment>)> {
    // We use recursion here to fetch the entire comment chain up to the top-level parent. This is
    // necessary because we need to know the post and parent comment in order to insert a new
    // comment. However it can also lead to too much resource consumption when fetching many
    // comments recursively. To avoid this we check the request count against max comment depth.
    //
    // A separate task is spawned for the recursive call. Otherwise, when the async executor polls
    // the task this is in, the poll function's call stack would grow with the level of recursion,
    // so a stack overflow would be possible.
    //
    // The stack overflow prevention relies on the total laziness that the async keyword provides
    // (https://rust-lang.github.io/rfcs/2394-async_await.html#async-functions). This means you need
    // to be careful if you want to change `Note::get_parents` and `CreateOrUpdateNote::verify` from
    // `async fn foo(...) -> T` to `fn foo(...) -> impl Future<Output = T>`. Between each level of
    // recursion, there must be the beginning of at least one `async` block or `async fn`,
    // otherwise there might be multiple levels of recursion before the first poll.
    if context.request_count() > MAX_COMMENT_DEPTH_LIMIT.try_into()? {
      return Err(LemmyErrorType::MaxCommentDepthReached.into());
    }
    let parent = tokio::spawn({
      let in_reply_to = self.in_reply_to.clone();
      let context = context.clone();
      // This is wrapped in an async block only to satisfy the borrow checker. This wrapping is not
      // needed for the stack overflow prevention.
      async move { in_reply_to.dereference(&context).await }
    })
    .await??;
    match parent {
      PostOrComment::Left(p) => Ok((p.clone(), None)),
      PostOrComment::Right(c) => {
        let post_id = c.post_id;
        let post = Post::read(&mut context.pool(), post_id).await?;
        Ok((post.into(), Some(c.clone())))
      }
    }
  }
}

impl InCommunity for Note {
  async fn community(&self, context: &Data<LemmyContext>) -> LemmyResult<ApubCommunity> {
    if let Some(audience) = &self.audience {
      return audience.dereference(context).await;
    }
    let (post, _) = self.get_parents(context).await?;
    let community = Community::read(&mut context.pool(), post.community_id).await?;
    Ok(community.into())
  }
}
