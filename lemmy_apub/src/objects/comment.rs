use crate::{
  extensions::context::lemmy_context,
  fetcher::{
    get_or_fetch_and_insert_comment,
    get_or_fetch_and_insert_post,
    get_or_fetch_and_upsert_user,
  },
  objects::{
    check_object_domain,
    check_object_for_community_or_site_ban,
    create_tombstone,
    get_object_from_apub,
    get_source_markdown_value,
    set_content_and_source,
    FromApub,
    FromApubToForm,
    ToApub,
  },
  NoteExt,
};
use activitystreams::{
  object::{kind::NoteType, ApObject, Note, Tombstone},
  prelude::*,
};
use anyhow::{anyhow, Context};
use lemmy_db::{
  comment::{Comment, CommentForm},
  community::Community,
  post::Post,
  user::User_,
  Crud,
  DbPool,
};
use lemmy_structs::blocking;
use lemmy_utils::{
  location_info,
  utils::{convert_datetime, remove_slurs},
  LemmyError,
};
use lemmy_websocket::LemmyContext;
use url::Url;

#[async_trait::async_trait(?Send)]
impl ToApub for Comment {
  type ApubType = NoteExt;

  async fn to_apub(&self, pool: &DbPool) -> Result<NoteExt, LemmyError> {
    let mut comment = ApObject::new(Note::new());

    let creator_id = self.creator_id;
    let creator = blocking(pool, move |conn| User_::read(conn, creator_id)).await??;

    let post_id = self.post_id;
    let post = blocking(pool, move |conn| Post::read(conn, post_id)).await??;

    let community_id = post.community_id;
    let community = blocking(pool, move |conn| Community::read(conn, community_id)).await??;

    // Add a vector containing some important info to the "in_reply_to" field
    // [post_ap_id, Option(parent_comment_ap_id)]
    let mut in_reply_to_vec = vec![post.ap_id];

    if let Some(parent_id) = self.parent_id {
      let parent_comment = blocking(pool, move |conn| Comment::read(conn, parent_id)).await??;

      in_reply_to_vec.push(parent_comment.ap_id);
    }

    comment
      // Not needed when the Post is embedded in a collection (like for community outbox)
      .set_many_contexts(lemmy_context()?)
      .set_id(Url::parse(&self.ap_id)?)
      .set_published(convert_datetime(self.published))
      .set_to(community.actor_id)
      .set_many_in_reply_tos(in_reply_to_vec)
      .set_attributed_to(creator.actor_id);

    set_content_and_source(&mut comment, &self.content)?;

    if let Some(u) = self.updated {
      comment.set_updated(convert_datetime(u));
    }

    Ok(comment)
  }

  fn to_tombstone(&self) -> Result<Tombstone, LemmyError> {
    create_tombstone(self.deleted, &self.ap_id, self.updated, NoteType::Note)
  }
}

#[async_trait::async_trait(?Send)]
impl FromApub for Comment {
  type ApubType = NoteExt;

  /// Converts a `Note` to `Comment`.
  ///
  /// If the parent community, post and comment(s) are not known locally, these are also fetched.
  async fn from_apub(
    note: &NoteExt,
    context: &LemmyContext,
    expected_domain: Url,
    request_counter: &mut i32,
  ) -> Result<Comment, LemmyError> {
    check_object_for_community_or_site_ban(note, context, request_counter).await?;

    let comment: Comment =
      get_object_from_apub(note, context, expected_domain, request_counter).await?;

    let post_id = comment.post_id;
    let post = blocking(context.pool(), move |conn| Post::read(conn, post_id)).await??;
    if post.locked {
      // This is not very efficient because a comment gets inserted just to be deleted right
      // afterwards, but it seems to be the easiest way to implement it.
      blocking(context.pool(), move |conn| {
        Comment::delete(conn, comment.id)
      })
      .await??;
      return Err(anyhow!("Post is locked").into());
    } else {
      Ok(comment)
    }
  }
}

#[async_trait::async_trait(?Send)]
impl FromApubToForm<NoteExt> for CommentForm {
  async fn from_apub(
    note: &NoteExt,
    context: &LemmyContext,
    expected_domain: Url,
    request_counter: &mut i32,
  ) -> Result<CommentForm, LemmyError> {
    let creator_actor_id = &note
      .attributed_to()
      .context(location_info!())?
      .as_single_xsd_any_uri()
      .context(location_info!())?;

    let creator = get_or_fetch_and_upsert_user(creator_actor_id, context, request_counter).await?;

    let mut in_reply_tos = note
      .in_reply_to()
      .as_ref()
      .context(location_info!())?
      .as_many()
      .context(location_info!())?
      .iter()
      .map(|i| i.as_xsd_any_uri().context(""));
    let post_ap_id = in_reply_tos.next().context(location_info!())??;

    // This post, or the parent comment might not yet exist on this server yet, fetch them.
    let post = get_or_fetch_and_insert_post(&post_ap_id, context, request_counter).await?;

    // The 2nd item, if it exists, is the parent comment apub_id
    // For deeply nested comments, FromApub automatically gets called recursively
    let parent_id: Option<i32> = match in_reply_tos.next() {
      Some(parent_comment_uri) => {
        let parent_comment_ap_id = &parent_comment_uri?;
        let parent_comment =
          get_or_fetch_and_insert_comment(&parent_comment_ap_id, context, request_counter).await?;

        Some(parent_comment.id)
      }
      None => None,
    };

    let content = get_source_markdown_value(note)?.context(location_info!())?;
    let content_slurs_removed = remove_slurs(&content);

    Ok(CommentForm {
      creator_id: creator.id,
      post_id: post.id,
      parent_id,
      content: content_slurs_removed,
      removed: None,
      read: None,
      published: note.published().map(|u| u.to_owned().naive_local()),
      updated: note.updated().map(|u| u.to_owned().naive_local()),
      deleted: None,
      ap_id: Some(check_object_domain(note, expected_domain)?),
      local: false,
    })
  }
}
