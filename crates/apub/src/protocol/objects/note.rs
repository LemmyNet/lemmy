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
use lemmy_utils::error::LemmyResult;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
use std::ops::Deref;
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
    // Fetch parent comment chain in a box, otherwise it can cause a stack overflow.
    let parent = Box::pin(self.in_reply_to.dereference(context).await?);
    match parent.deref() {
      PostOrComment::Post(p) => Ok((p.clone(), None)),
      PostOrComment::Comment(c) => {
        let post_id = c.post_id;
        let post = Post::read(&mut context.pool(), post_id).await?;
        Ok((post.into(), Some(c.clone())))
      }
    }
  }
}

#[async_trait::async_trait]
impl InCommunity for Note {
  async fn community(&self, context: &Data<LemmyContext>) -> LemmyResult<ApubCommunity> {
    let (post, _) = self.get_parents(context).await?;
    let community = Community::read(&mut context.pool(), post.community_id).await?;
    if let Some(audience) = &self.audience {
      verify_community_matches(audience, community.actor_id.clone())?;
    }
    Ok(community.into())
  }
}
