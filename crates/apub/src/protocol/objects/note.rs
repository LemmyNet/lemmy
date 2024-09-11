use crate::{
  activities::verify_community_matches,
  fetcher::post_or_comment::PostOrComment,
  mentions::MentionOrValue,
  objects::{comment::ApubComment, community::ApubCommunity, person::ApubPerson, post::ApubPost},
  protocol::{objects::LanguageTag, InCommunity, Source},
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
use lemmy_api_common::context::LemmyContext;
use lemmy_db_schema::{
  source::{community::Community, post::Post},
  traits::Crud,
};
use lemmy_utils::{error::{LemmyError, LemmyResult}, LemmyErrorType, MAX_COMMENT_DEPTH_LIMIT};
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
use url::Url;

#[skip_serializing_none]
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Note {
  pub(crate) r#type: NoteType,
  pub(crate) id: ObjectId<ApubComment>,
  pub(crate) attributed_to: ObjectId<ApubPerson>,
  #[serde(deserialize_with = "deserialize_one_or_many")]
  pub(crate) to: Vec<Url>,
  #[serde(deserialize_with = "deserialize_one_or_many", default)]
  pub(crate) cc: Vec<Url>,
  pub(crate) content: String,
  pub(crate) in_reply_to: ObjectId<PostOrComment>,

  pub(crate) media_type: Option<MediaTypeMarkdownOrHtml>,
  #[serde(deserialize_with = "deserialize_skip_error", default)]
  pub(crate) source: Option<Source>,
  pub(crate) published: Option<DateTime<Utc>>,
  pub(crate) updated: Option<DateTime<Utc>>,
  #[serde(default)]
  pub(crate) tag: Vec<MentionOrValue>,
  // lemmy extension
  pub(crate) distinguished: Option<bool>,
  pub(crate) language: Option<LanguageTag>,
  pub(crate) audience: Option<ObjectId<ApubCommunity>>,
}

impl Note {
  pub(crate) async fn get_parents(
    &self,
    context: &Data<LemmyContext>,
  ) -> LemmyResult<(ApubPost, Option<ApubComment>)> {
    // We use recursion here to fetch the entire comment chain up to the top-level parent. This is
    // necessary because we need to know the post and parent comment in order to insert a new
    // comment. However it can also lead to stack overflow when fetching many comments recursively.
    // To avoid this we check the request count against max comment depth, which based on testing
    // can be handled without risking stack overflow. This is not a perfect solution, because in
    // some cases we have to fetch user profiles too, and reach the limit after only 25 comments
    // or so.
    // A cleaner solution would be converting the recursion into a loop, but that is tricky.
    // Use a lower request limit here. Otherwise we can run into stack overflow due to recursion.
    if context.request_count() > MAX_COMMENT_DEPTH_LIMIT as u32 {
      Err(LemmyErrorType::MaxCommentDepthReached)?;
    }
    let parent = self.in_reply_to.dereference(context).await?;
    match parent {
      PostOrComment::Post(p) => Ok((p.clone(), None)),
      PostOrComment::Comment(c) => {
        let post_id = c.post_id;
        let post = Box::pin(Post::read(&mut context.pool(), post_id))
          .await?
          .ok_or(LemmyErrorType::CouldntFindPost)?;
        Ok((post.into(), Some(c.clone())))
      }
    }
  }
}

#[async_trait::async_trait]
impl InCommunity for Note {
  async fn community(&self, context: &Data<LemmyContext>) -> LemmyResult<ApubCommunity> {
    let (post, _) = self.get_parents(context).await?;
    let community = Community::read(&mut context.pool(), post.community_id)
      .await?
      .ok_or(LemmyErrorType::CouldntFindCommunity)?;
    if let Some(audience) = &self.audience {
      verify_community_matches(audience, community.actor_id.clone())?;
    }
    Ok(community.into())
  }
}
