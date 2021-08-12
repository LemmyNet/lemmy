use crate::{
  activities::verify_person_in_community,
  extensions::context::lemmy_context,
  fetcher::objects::{
    get_or_fetch_and_insert_comment,
    get_or_fetch_and_insert_post,
    get_or_fetch_and_insert_post_or_comment,
  },
  migrations::CommentInReplyToMigration,
  objects::{create_tombstone, get_or_fetch_and_upsert_person, FromApub, Source, ToApub},
  ActorType,
  PostOrComment,
};
use activitystreams::{
  base::AnyBase,
  object::{kind::NoteType, Tombstone},
  primitives::OneOrMany,
  unparsed::Unparsed,
};
use anyhow::{anyhow, Context};
use chrono::{DateTime, FixedOffset};
use lemmy_api_common::blocking;
use lemmy_apub_lib::{
  values::{MediaTypeHtml, MediaTypeMarkdown, PublicUrl},
  verify_domains_match,
};
use lemmy_db_queries::{ApubObject, Crud, DbPool};
use lemmy_db_schema::{
  source::{
    comment::{Comment, CommentForm},
    community::Community,
    person::Person,
    post::Post,
  },
  CommentId,
};
use lemmy_utils::{
  location_info,
  utils::{convert_datetime, remove_slurs},
  LemmyError,
};
use lemmy_websocket::LemmyContext;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
use std::ops::Deref;
use url::Url;

#[skip_serializing_none]
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Note {
  #[serde(rename = "@context")]
  context: OneOrMany<AnyBase>,
  r#type: NoteType,
  id: Url,
  pub(crate) attributed_to: Url,
  /// Indicates that the object is publicly readable. Unlike [`Post.to`], this one doesn't contain
  /// the community ID, as it would be incompatible with Pleroma (and we can get the community from
  /// the post in [`in_reply_to`]).
  to: PublicUrl,
  content: String,
  media_type: MediaTypeHtml,
  source: Source,
  in_reply_to: CommentInReplyToMigration,
  published: DateTime<FixedOffset>,
  updated: Option<DateTime<FixedOffset>>,
  #[serde(flatten)]
  unparsed: Unparsed,
}

impl Note {
  pub(crate) fn id_unchecked(&self) -> &Url {
    &self.id
  }
  pub(crate) fn id(&self, expected_domain: &Url) -> Result<&Url, LemmyError> {
    verify_domains_match(&self.id, expected_domain)?;
    Ok(&self.id)
  }

  async fn get_parents(
    &self,
    context: &LemmyContext,
    request_counter: &mut i32,
  ) -> Result<(Post, Option<CommentId>), LemmyError> {
    match &self.in_reply_to {
      CommentInReplyToMigration::Old(in_reply_to) => {
        // This post, or the parent comment might not yet exist on this server yet, fetch them.
        let post_id = in_reply_to.get(0).context(location_info!())?;
        let post = Box::pin(get_or_fetch_and_insert_post(
          post_id,
          context,
          request_counter,
        ))
        .await?;

        // The 2nd item, if it exists, is the parent comment apub_id
        // Nested comments will automatically get fetched recursively
        let parent_id: Option<CommentId> = match in_reply_to.get(1) {
          Some(parent_comment_uri) => {
            let parent_comment = Box::pin(get_or_fetch_and_insert_comment(
              parent_comment_uri,
              context,
              request_counter,
            ))
            .await?;

            Some(parent_comment.id)
          }
          None => None,
        };

        Ok((post, parent_id))
      }
      CommentInReplyToMigration::New(in_reply_to) => {
        let parent = Box::pin(
          get_or_fetch_and_insert_post_or_comment(in_reply_to, context, request_counter).await?,
        );
        match parent.deref() {
          PostOrComment::Post(p) => {
            // Workaround because I cant figure ut how to get the post out of the box (and we dont
            // want to stackoverflow in a deep comment hierarchy).
            let post_id = p.id;
            let post = blocking(context.pool(), move |conn| Post::read(conn, post_id)).await??;
            Ok((post, None))
          }
          PostOrComment::Comment(c) => {
            let post_id = c.post_id;
            let post = blocking(context.pool(), move |conn| Post::read(conn, post_id)).await??;
            Ok((post, Some(c.id)))
          }
        }
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
    let community = blocking(context.pool(), move |conn| {
      Community::read(conn, community_id)
    })
    .await??;

    if post.locked {
      return Err(anyhow!("Post is locked").into());
    }
    verify_domains_match(&self.attributed_to, &self.id)?;
    verify_person_in_community(
      &self.attributed_to,
      &community.actor_id(),
      context,
      request_counter,
    )
    .await?;
    Ok(())
  }
}

#[async_trait::async_trait(?Send)]
impl ToApub for Comment {
  type ApubType = Note;

  async fn to_apub(&self, pool: &DbPool) -> Result<Note, LemmyError> {
    let creator_id = self.creator_id;
    let creator = blocking(pool, move |conn| Person::read(conn, creator_id)).await??;

    let post_id = self.post_id;
    let post = blocking(pool, move |conn| Post::read(conn, post_id)).await??;

    // Add a vector containing some important info to the "in_reply_to" field
    // [post_ap_id, Option(parent_comment_ap_id)]
    let mut in_reply_to_vec = vec![post.ap_id.into_inner()];

    if let Some(parent_id) = self.parent_id {
      let parent_comment = blocking(pool, move |conn| Comment::read(conn, parent_id)).await??;

      in_reply_to_vec.push(parent_comment.ap_id.into_inner());
    }

    let note = Note {
      context: lemmy_context(),
      r#type: NoteType::Note,
      id: self.ap_id.to_owned().into_inner(),
      attributed_to: creator.actor_id.into_inner(),
      to: PublicUrl::Public,
      content: self.content.clone(),
      media_type: MediaTypeHtml::Html,
      source: Source {
        content: self.content.clone(),
        media_type: MediaTypeMarkdown::Markdown,
      },
      in_reply_to: CommentInReplyToMigration::Old(in_reply_to_vec),
      published: convert_datetime(self.published),
      updated: self.updated.map(convert_datetime),
      unparsed: Default::default(),
    };

    Ok(note)
  }

  fn to_tombstone(&self) -> Result<Tombstone, LemmyError> {
    create_tombstone(
      self.deleted,
      self.ap_id.to_owned().into(),
      self.updated,
      NoteType::Note,
    )
  }
}

#[async_trait::async_trait(?Send)]
impl FromApub for Comment {
  type ApubType = Note;

  /// Converts a `Note` to `Comment`.
  ///
  /// If the parent community, post and comment(s) are not known locally, these are also fetched.
  async fn from_apub(
    note: &Note,
    context: &LemmyContext,
    expected_domain: &Url,
    request_counter: &mut i32,
  ) -> Result<Comment, LemmyError> {
    let ap_id = Some(note.id(expected_domain)?.clone().into());
    let creator =
      get_or_fetch_and_upsert_person(&note.attributed_to, context, request_counter).await?;
    let (post, parent_comment_id) = note.get_parents(context, request_counter).await?;

    let content = &note.source.content;
    let content_slurs_removed = remove_slurs(content);

    let form = CommentForm {
      creator_id: creator.id,
      post_id: post.id,
      parent_id: parent_comment_id,
      content: content_slurs_removed,
      removed: None,
      read: None,
      published: Some(note.published.naive_local()),
      updated: note.updated.map(|u| u.to_owned().naive_local()),
      deleted: None,
      ap_id,
      local: Some(false),
    };
    Ok(blocking(context.pool(), move |conn| Comment::upsert(conn, &form)).await??)
  }
}
