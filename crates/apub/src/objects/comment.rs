use crate::{
  activities::{verify_is_public, verify_person_in_community},
  check_is_apub_id_valid,
  mentions::collect_non_local_mentions,
  objects::{check_is_local_object, read_from_string_or_source},
  protocol::{
    objects::{note::Note, tombstone::Tombstone},
    Source,
  },
  PostOrComment,
};
use activitystreams_kinds::{object::NoteType, public};
use chrono::NaiveDateTime;
use lemmy_api_common::blocking;
use lemmy_apub_lib::{
  object_id::ObjectId,
  traits::ApubObject,
  values::MediaTypeHtml,
  verify::verify_domains_match,
};
use lemmy_db_schema::{
  source::{
    comment::{Comment, CommentForm},
    community::Community,
    person::Person,
    post::Post,
  },
  traits::Crud,
};
use lemmy_utils::{
  utils::{convert_datetime, markdown_to_html, remove_slurs},
  LemmyError,
};
use lemmy_websocket::LemmyContext;
use std::ops::Deref;
use url::Url;

#[derive(Clone, Debug)]
pub struct ApubComment(Comment);

impl Deref for ApubComment {
  type Target = Comment;
  fn deref(&self) -> &Self::Target {
    &self.0
  }
}

impl From<Comment> for ApubComment {
  fn from(c: Comment) -> Self {
    ApubComment(c)
  }
}

#[async_trait::async_trait(?Send)]
impl ApubObject for ApubComment {
  type DataType = LemmyContext;
  type ApubType = Note;
  type DbType = Comment;
  type TombstoneType = Tombstone;

  fn last_refreshed_at(&self) -> Option<NaiveDateTime> {
    None
  }

  #[tracing::instrument(skip_all)]
  async fn read_from_apub_id(
    object_id: Url,
    context: &LemmyContext,
  ) -> Result<Option<Self>, LemmyError> {
    Ok(
      blocking(context.pool(), move |conn| {
        Comment::read_from_apub_id(conn, object_id)
      })
      .await??
      .map(Into::into),
    )
  }

  #[tracing::instrument(skip_all)]
  async fn delete(self, context: &LemmyContext) -> Result<(), LemmyError> {
    if !self.deleted {
      blocking(context.pool(), move |conn| {
        Comment::update_deleted(conn, self.id, true)
      })
      .await??;
    }
    Ok(())
  }

  #[tracing::instrument(skip_all)]
  async fn into_apub(self, context: &LemmyContext) -> Result<Note, LemmyError> {
    let creator_id = self.creator_id;
    let creator = blocking(context.pool(), move |conn| Person::read(conn, creator_id)).await??;

    let post_id = self.post_id;
    let post = blocking(context.pool(), move |conn| Post::read(conn, post_id)).await??;
    let community_id = post.community_id;
    let community = blocking(context.pool(), move |conn| {
      Community::read(conn, community_id)
    })
    .await??;

    let in_reply_to = if let Some(comment_id) = self.parent_id {
      let parent_comment =
        blocking(context.pool(), move |conn| Comment::read(conn, comment_id)).await??;
      ObjectId::<PostOrComment>::new(parent_comment.ap_id)
    } else {
      ObjectId::<PostOrComment>::new(post.ap_id)
    };
    let maa =
      collect_non_local_mentions(&self, ObjectId::new(community.actor_id), context, &mut 0).await?;

    let note = Note {
      r#type: NoteType::Note,
      id: ObjectId::new(self.ap_id.clone()),
      attributed_to: ObjectId::new(creator.actor_id),
      to: vec![public()],
      cc: maa.ccs,
      content: markdown_to_html(&self.content),
      media_type: Some(MediaTypeHtml::Html),
      source: Some(Source::new(self.content.clone())),
      in_reply_to,
      published: Some(convert_datetime(self.published)),
      updated: self.updated.map(convert_datetime),
      tag: maa.tags,
    };

    Ok(note)
  }

  fn to_tombstone(&self) -> Result<Tombstone, LemmyError> {
    Ok(Tombstone::new(self.ap_id.clone().into()))
  }

  #[tracing::instrument(skip_all)]
  async fn verify(
    note: &Note,
    expected_domain: &Url,
    context: &LemmyContext,
    request_counter: &mut i32,
  ) -> Result<(), LemmyError> {
    verify_domains_match(note.id.inner(), expected_domain)?;
    verify_domains_match(note.attributed_to.inner(), note.id.inner())?;
    verify_is_public(&note.to, &note.cc)?;
    let (post, _) = note.get_parents(context, request_counter).await?;
    let community_id = post.community_id;
    let community = blocking(context.pool(), move |conn| {
      Community::read(conn, community_id)
    })
    .await??;
    check_is_apub_id_valid(note.id.inner(), community.local, &context.settings())?;
    check_is_local_object(note.id.inner())?;
    verify_person_in_community(
      &note.attributed_to,
      &community.into(),
      context,
      request_counter,
    )
    .await?;
    if post.locked {
      return Err(LemmyError::from_message("Post is locked"));
    }
    Ok(())
  }

  /// Converts a `Note` to `Comment`.
  ///
  /// If the parent community, post and comment(s) are not known locally, these are also fetched.
  #[tracing::instrument(skip_all)]
  async fn from_apub(
    note: Note,
    context: &LemmyContext,
    request_counter: &mut i32,
  ) -> Result<ApubComment, LemmyError> {
    let creator = note
      .attributed_to
      .dereference(context, context.client(), request_counter)
      .await?;
    let (post, parent_comment_id) = note.get_parents(context, request_counter).await?;

    let content = read_from_string_or_source(&note.content, &note.source);
    let content_slurs_removed = remove_slurs(&content, &context.settings().slur_regex());

    let form = CommentForm {
      creator_id: creator.id,
      post_id: post.id,
      parent_id: parent_comment_id,
      content: content_slurs_removed,
      removed: None,
      read: None,
      published: note.published.map(|u| u.naive_local()),
      updated: note.updated.map(|u| u.naive_local()),
      deleted: None,
      ap_id: Some(note.id.into()),
      local: Some(false),
    };
    let comment = blocking(context.pool(), move |conn| Comment::upsert(conn, &form)).await??;
    Ok(comment.into())
  }
}

#[cfg(test)]
pub(crate) mod tests {
  use super::*;
  use crate::{
    objects::{
      community::{tests::parse_lemmy_community, ApubCommunity},
      instance::ApubSite,
      person::{tests::parse_lemmy_person, ApubPerson},
      post::ApubPost,
      tests::init_context,
    },
    protocol::tests::file_to_json_object,
  };
  use assert_json_diff::assert_json_include;
  use html2md::parse_html;
  use lemmy_db_schema::source::site::Site;
  use serial_test::serial;

  async fn prepare_comment_test(
    url: &Url,
    context: &LemmyContext,
  ) -> (ApubPerson, ApubCommunity, ApubPost, ApubSite) {
    let (person, site) = parse_lemmy_person(context).await;
    let community = parse_lemmy_community(context).await;
    let post_json = file_to_json_object("assets/lemmy/objects/page.json").unwrap();
    ApubPost::verify(&post_json, url, context, &mut 0)
      .await
      .unwrap();
    let post = ApubPost::from_apub(post_json, context, &mut 0)
      .await
      .unwrap();
    (person, community, post, site)
  }

  fn cleanup(data: (ApubPerson, ApubCommunity, ApubPost, ApubSite), context: &LemmyContext) {
    Post::delete(&*context.pool().get().unwrap(), data.2.id).unwrap();
    Community::delete(&*context.pool().get().unwrap(), data.1.id).unwrap();
    Person::delete(&*context.pool().get().unwrap(), data.0.id).unwrap();
    Site::delete(&*context.pool().get().unwrap(), data.3.id).unwrap();
  }

  #[actix_rt::test]
  #[serial]
  pub(crate) async fn test_parse_lemmy_comment() {
    let context = init_context();
    let url = Url::parse("https://enterprise.lemmy.ml/comment/38741").unwrap();
    let data = prepare_comment_test(&url, &context).await;

    let json: Note = file_to_json_object("assets/lemmy/objects/note.json").unwrap();
    let mut request_counter = 0;
    ApubComment::verify(&json, &url, &context, &mut request_counter)
      .await
      .unwrap();
    let comment = ApubComment::from_apub(json.clone(), &context, &mut request_counter)
      .await
      .unwrap();

    assert_eq!(comment.ap_id, url.into());
    assert_eq!(comment.content.len(), 14);
    assert!(!comment.local);
    assert_eq!(request_counter, 0);

    let comment_id = comment.id;
    let to_apub = comment.into_apub(&context).await.unwrap();
    assert_json_include!(actual: json, expected: to_apub);

    Comment::delete(&*context.pool().get().unwrap(), comment_id).unwrap();
    cleanup(data, &context);
  }

  #[actix_rt::test]
  #[serial]
  async fn test_parse_pleroma_comment() {
    let context = init_context();
    let url = Url::parse("https://enterprise.lemmy.ml/comment/38741").unwrap();
    let data = prepare_comment_test(&url, &context).await;

    let pleroma_url =
      Url::parse("https://queer.hacktivis.me/objects/8d4973f4-53de-49cd-8c27-df160e16a9c2")
        .unwrap();
    let person_json = file_to_json_object("assets/pleroma/objects/person.json").unwrap();
    ApubPerson::verify(&person_json, &pleroma_url, &context, &mut 0)
      .await
      .unwrap();
    ApubPerson::from_apub(person_json, &context, &mut 0)
      .await
      .unwrap();
    let json = file_to_json_object("assets/pleroma/objects/note.json").unwrap();
    let mut request_counter = 0;
    ApubComment::verify(&json, &pleroma_url, &context, &mut request_counter)
      .await
      .unwrap();
    let comment = ApubComment::from_apub(json, &context, &mut request_counter)
      .await
      .unwrap();

    assert_eq!(comment.ap_id, pleroma_url.into());
    assert_eq!(comment.content.len(), 64);
    assert!(!comment.local);
    assert_eq!(request_counter, 0);

    Comment::delete(&*context.pool().get().unwrap(), comment.id).unwrap();
    cleanup(data, &context);
  }

  #[actix_rt::test]
  #[serial]
  async fn test_html_to_markdown_sanitize() {
    let parsed = parse_html("<script></script><b>hello</b>");
    assert_eq!(parsed, "**hello**");
  }
}
