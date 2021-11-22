use crate::{
  fetcher::post_or_comment::PostOrComment,
  mentions::Mention,
  objects::{comment::ApubComment, person::ApubPerson, post::ApubPost},
  protocol::{Source, Unparsed},
};
use activitystreams_kinds::object::NoteType;
use chrono::{DateTime, FixedOffset};
use lemmy_api_common::blocking;
use lemmy_apub_lib::{object_id::ObjectId, values::MediaTypeHtml};
use lemmy_db_schema::{newtypes::CommentId, source::post::Post, traits::Crud};
use lemmy_utils::LemmyError;
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
  pub(crate) to: Vec<Url>,
  #[serde(default)]
  pub(crate) cc: Vec<Url>,
  pub(crate) content: String,
  pub(crate) media_type: Option<MediaTypeHtml>,
  #[serde(default)]
  pub(crate) source: SourceCompat,
  pub(crate) in_reply_to: ObjectId<PostOrComment>,
  pub(crate) published: Option<DateTime<FixedOffset>>,
  pub(crate) updated: Option<DateTime<FixedOffset>>,
  #[serde(default)]
  pub(crate) tag: Vec<Mention>,
  #[serde(flatten)]
  pub(crate) unparsed: Unparsed,
}

/// Pleroma puts a raw string in the source, so we have to handle it here for deserialization to work
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
#[serde(untagged)]
pub(crate) enum SourceCompat {
  None,
  Lemmy(Source),
  Pleroma(String),
}

impl Default for SourceCompat {
  fn default() -> Self {
    SourceCompat::None
  }
}

impl Note {
  pub(crate) async fn get_parents(
    &self,
    context: &LemmyContext,
    request_counter: &mut i32,
  ) -> Result<(ApubPost, Option<CommentId>), LemmyError> {
    // Fetch parent comment chain in a box, otherwise it can cause a stack overflow.
    let parent = Box::pin(
      self
        .in_reply_to
        .dereference(context, request_counter)
        .await?,
    );
    match parent.deref() {
      PostOrComment::Post(p) => {
        // Workaround because I cant figure out how to get the post out of the box (and we dont
        // want to stackoverflow in a deep comment hierarchy).
        let post_id = p.id;
        let post = blocking(context.pool(), move |conn| Post::read(conn, post_id)).await??;
        Ok((post.into(), None))
      }
      PostOrComment::Comment(c) => {
        let post_id = c.post_id;
        let post = blocking(context.pool(), move |conn| Post::read(conn, post_id)).await??;
        Ok((post.into(), Some(c.id)))
      }
    }
  }
}
