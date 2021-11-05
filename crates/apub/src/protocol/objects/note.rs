use crate::{
  activities::{verify_is_public, verify_person_in_community},
  fetcher::post_or_comment::PostOrComment,
  objects::{comment::ApubComment, community::ApubCommunity, person::ApubPerson, post::ApubPost},
  protocol::Source,
};
use activitystreams::{object::kind::NoteType, unparsed::Unparsed};
use anyhow::anyhow;
use chrono::{DateTime, FixedOffset};
use lemmy_api_common::blocking;
use lemmy_apub_lib::{object_id::ObjectId, values::MediaTypeHtml, verify::verify_domains_match};
use lemmy_db_schema::{
  newtypes::CommentId,
  source::{community::Community, post::Post},
  traits::Crud,
};
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
  pub(crate) content: String,
  pub(crate) media_type: Option<MediaTypeHtml>,
  pub(crate) source: SourceCompat,
  pub(crate) in_reply_to: ObjectId<PostOrComment>,
  pub(crate) published: Option<DateTime<FixedOffset>>,
  pub(crate) updated: Option<DateTime<FixedOffset>>,
  #[serde(flatten)]
  pub(crate) unparsed: Unparsed,
}

/// Pleroma puts a raw string in the source, so we have to handle it here for deserialization to work
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
#[serde(untagged)]
pub(crate) enum SourceCompat {
  Lemmy(Source),
  Pleroma(String),
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

  pub(crate) async fn verify(
    &self,
    context: &LemmyContext,
    request_counter: &mut i32,
  ) -> Result<(), LemmyError> {
    let (post, _parent_comment_id) = self.get_parents(context, request_counter).await?;
    let community_id = post.community_id;
    let community: ApubCommunity = blocking(context.pool(), move |conn| {
      Community::read(conn, community_id)
    })
    .await??
    .into();

    if post.locked {
      return Err(anyhow!("Post is locked").into());
    }
    verify_domains_match(self.attributed_to.inner(), self.id.inner())?;
    verify_person_in_community(&self.attributed_to, &community, context, request_counter).await?;
    verify_is_public(&self.to)?;
    Ok(())
  }
}
