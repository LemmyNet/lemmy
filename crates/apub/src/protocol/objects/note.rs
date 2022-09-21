use crate::{
  fetcher::post_or_comment::PostOrComment,
  local_instance,
  mentions::MentionOrValue,
  objects::{comment::ApubComment, person::ApubPerson, post::ApubPost},
  protocol::{objects::LanguageTag, Source},
};
use activitypub_federation::{
  core::object_id::ObjectId,
  deser::{
    helpers::{deserialize_one_or_many, deserialize_skip_error},
    values::MediaTypeMarkdownOrHtml,
  },
};
use activitystreams_kinds::object::NoteType;
use chrono::{DateTime, FixedOffset};
use lemmy_api_common::utils::blocking;
use lemmy_db_schema::{source::post::Post, traits::Crud};
use lemmy_utils::error::LemmyError;
use lemmy_websocket::LemmyContext;
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
  pub(crate) published: Option<DateTime<FixedOffset>>,
  pub(crate) updated: Option<DateTime<FixedOffset>>,
  #[serde(default)]
  pub(crate) tag: Vec<MentionOrValue>,
  // lemmy extension
  pub(crate) distinguished: Option<bool>,
  pub(crate) language: Option<LanguageTag>,
}

impl Note {
  pub(crate) async fn get_parents(
    &self,
    context: &LemmyContext,
    request_counter: &mut i32,
  ) -> Result<(ApubPost, Option<ApubComment>), LemmyError> {
    // Fetch parent comment chain in a box, otherwise it can cause a stack overflow.
    let parent = Box::pin(
      self
        .in_reply_to
        .dereference(context, local_instance(context), request_counter)
        .await?,
    );
    match parent.deref() {
      PostOrComment::Post(p) => {
        let post = p.deref().to_owned();
        Ok((post, None))
      }
      PostOrComment::Comment(c) => {
        let post_id = c.post_id;
        let post = blocking(context.pool(), move |conn| Post::read(conn, post_id)).await??;
        let comment = c.deref().to_owned();
        Ok((post.into(), Some(comment)))
      }
    }
  }
}
