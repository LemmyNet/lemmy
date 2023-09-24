use crate::{
  activities::{verify_is_public, verify_person_in_community},
  check_apub_id_valid_with_strictness,
  mentions::collect_non_local_mentions,
  objects::{read_from_string_or_source, verify_is_remote_object},
  protocol::{
    objects::{note::Note, LanguageTag},
    InCommunity,
    Source,
  },
};
use activitypub_federation::{
  config::Data,
  kinds::{object::NoteType, public},
  protocol::{values::MediaTypeMarkdownOrHtml, verification::verify_domains_match},
  traits::Object,
};
use chrono::{DateTime, Utc};
use lemmy_api_common::{
  context::LemmyContext,
  utils::{local_site_opt_to_slur_regex, sanitize_html_federation},
};
use lemmy_db_schema::{
  source::{
    comment::{Comment, CommentInsertForm, CommentUpdateForm},
    community::Community,
    local_site::LocalSite,
    person::Person,
    post::Post,
  },
  traits::Crud,
};
use lemmy_utils::{
  error::{LemmyError, LemmyErrorType},
  utils::{markdown::markdown_to_html, slurs::remove_slurs, time::convert_datetime},
};
use std::ops::Deref;
use url::Url;

#[derive(Clone, Debug)]
pub struct ApubComment(pub(crate) Comment);

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

#[async_trait::async_trait]
impl Object for ApubComment {
  type DataType = LemmyContext;
  type Kind = Note;
  type Error = LemmyError;

  fn last_refreshed_at(&self) -> Option<DateTime<Utc>> {
    None
  }

  #[tracing::instrument(skip_all)]
  async fn read_from_id(
    object_id: Url,
    context: &Data<Self::DataType>,
  ) -> Result<Option<Self>, LemmyError> {
    Ok(
      Comment::read_from_apub_id(&mut context.pool(), object_id)
        .await?
        .map(Into::into),
    )
  }

  #[tracing::instrument(skip_all)]
  async fn delete(self, context: &Data<Self::DataType>) -> Result<(), LemmyError> {
    if !self.deleted {
      let form = CommentUpdateForm {
        deleted: Some(true),
        ..Default::default()
      };
      Comment::update(&mut context.pool(), self.id, &form).await?;
    }
    Ok(())
  }

  #[tracing::instrument(skip_all)]
  async fn into_json(self, context: &Data<Self::DataType>) -> Result<Note, LemmyError> {
    let creator_id = self.creator_id;
    let creator = Person::read(&mut context.pool(), creator_id).await?;

    let post_id = self.post_id;
    let post = Post::read(&mut context.pool(), post_id).await?;
    let community_id = post.community_id;
    let community = Community::read(&mut context.pool(), community_id).await?;

    let in_reply_to = if let Some(comment_id) = self.parent_comment_id() {
      let parent_comment = Comment::read(&mut context.pool(), comment_id).await?;
      parent_comment.ap_id.into()
    } else {
      post.ap_id.into()
    };
    let language = LanguageTag::new_single(self.language_id, &mut context.pool()).await?;
    let maa = collect_non_local_mentions(&self, community.actor_id.clone().into(), context).await?;

    let note = Note {
      r#type: NoteType::Note,
      id: self.ap_id.clone().into(),
      attributed_to: creator.actor_id.into(),
      to: vec![public()],
      cc: maa.ccs,
      content: markdown_to_html(&self.content),
      media_type: Some(MediaTypeMarkdownOrHtml::Html),
      source: Some(Source::new(self.content.clone())),
      in_reply_to,
      published: Some(convert_datetime(self.published)),
      updated: self.updated.map(convert_datetime),
      tag: maa.tags,
      distinguished: Some(self.distinguished),
      language,
      audience: Some(community.actor_id.into()),
    };

    Ok(note)
  }

  #[tracing::instrument(skip_all)]
  async fn verify(
    note: &Note,
    expected_domain: &Url,
    context: &Data<LemmyContext>,
  ) -> Result<(), LemmyError> {
    verify_domains_match(note.id.inner(), expected_domain)?;
    verify_domains_match(note.attributed_to.inner(), note.id.inner())?;
    verify_is_public(&note.to, &note.cc)?;
    let community = note.community(context).await?;

    check_apub_id_valid_with_strictness(note.id.inner(), community.local, context).await?;
    verify_is_remote_object(note.id.inner(), context.settings())?;
    verify_person_in_community(&note.attributed_to, &community, context).await?;
    let (post, _) = note.get_parents(context).await?;
    if post.locked {
      Err(LemmyErrorType::PostIsLocked)?
    } else {
      Ok(())
    }
  }

  /// Converts a `Note` to `Comment`.
  ///
  /// If the parent community, post and comment(s) are not known locally, these are also fetched.
  #[tracing::instrument(skip_all)]
  async fn from_json(note: Note, context: &Data<LemmyContext>) -> Result<ApubComment, LemmyError> {
    let creator = note.attributed_to.dereference(context).await?;
    let (post, parent_comment) = note.get_parents(context).await?;

    let content = read_from_string_or_source(&note.content, &note.media_type, &note.source);

    let local_site = LocalSite::read(&mut context.pool()).await.ok();
    let slur_regex = &local_site_opt_to_slur_regex(&local_site);
    let content = remove_slurs(&content, slur_regex);
    let content = sanitize_html_federation(&content);
    let language_id =
      LanguageTag::to_language_id_single(note.language, &mut context.pool()).await?;

    let form = CommentInsertForm {
      creator_id: creator.id,
      post_id: post.id,
      content,
      removed: None,
      published: note.published.map(Into::into),
      updated: note.updated.map(Into::into),
      deleted: Some(false),
      ap_id: Some(note.id.into()),
      distinguished: note.distinguished,
      local: Some(false),
      language_id,
    };
    let parent_comment_path = parent_comment.map(|t| t.0.path);
    let comment = Comment::create(&mut context.pool(), &form, parent_comment_path.as_ref()).await?;
    Ok(comment.into())
  }
}

#[cfg(test)]
pub(crate) mod tests {
  #![allow(clippy::unwrap_used)]
  #![allow(clippy::indexing_slicing)]

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
    context: &Data<LemmyContext>,
  ) -> (ApubPerson, ApubCommunity, ApubPost, ApubSite) {
    // use separate counter so this doesnt affect tests
    let context2 = context.reset_request_count();
    let (person, site) = parse_lemmy_person(&context2).await;
    let community = parse_lemmy_community(&context2).await;
    let post_json = file_to_json_object("assets/lemmy/objects/page.json").unwrap();
    ApubPost::verify(&post_json, url, &context2).await.unwrap();
    let post = ApubPost::from_json(post_json, &context2).await.unwrap();
    (person, community, post, site)
  }

  async fn cleanup(data: (ApubPerson, ApubCommunity, ApubPost, ApubSite), context: &LemmyContext) {
    Post::delete(&mut context.pool(), data.2.id).await.unwrap();
    Community::delete(&mut context.pool(), data.1.id)
      .await
      .unwrap();
    Person::delete(&mut context.pool(), data.0.id)
      .await
      .unwrap();
    Site::delete(&mut context.pool(), data.3.id).await.unwrap();
    LocalSite::delete(&mut context.pool()).await.unwrap();
  }

  #[tokio::test]
  #[serial]
  pub(crate) async fn test_parse_lemmy_comment() {
    let context = init_context().await;
    let url = Url::parse("https://enterprise.lemmy.ml/comment/38741").unwrap();
    let data = prepare_comment_test(&url, &context).await;

    let json: Note = file_to_json_object("assets/lemmy/objects/note.json").unwrap();
    ApubComment::verify(&json, &url, &context).await.unwrap();
    let comment = ApubComment::from_json(json.clone(), &context)
      .await
      .unwrap();

    assert_eq!(comment.ap_id, url.into());
    assert_eq!(comment.content.len(), 14);
    assert!(!comment.local);
    assert_eq!(context.request_count(), 0);

    let comment_id = comment.id;
    let to_apub = comment.into_json(&context).await.unwrap();
    assert_json_include!(actual: json, expected: to_apub);

    Comment::delete(&mut context.pool(), comment_id)
      .await
      .unwrap();
    cleanup(data, &context).await;
  }

  #[tokio::test]
  #[serial]
  async fn test_parse_pleroma_comment() {
    let context = init_context().await;
    let url = Url::parse("https://enterprise.lemmy.ml/comment/38741").unwrap();
    let data = prepare_comment_test(&url, &context).await;

    let pleroma_url =
      Url::parse("https://queer.hacktivis.me/objects/8d4973f4-53de-49cd-8c27-df160e16a9c2")
        .unwrap();
    let person_json = file_to_json_object("assets/pleroma/objects/person.json").unwrap();
    ApubPerson::verify(&person_json, &pleroma_url, &context)
      .await
      .unwrap();
    ApubPerson::from_json(person_json, &context).await.unwrap();
    let json = file_to_json_object("assets/pleroma/objects/note.json").unwrap();
    ApubComment::verify(&json, &pleroma_url, &context)
      .await
      .unwrap();
    let comment = ApubComment::from_json(json, &context).await.unwrap();

    assert_eq!(comment.ap_id, pleroma_url.into());
    assert_eq!(comment.content.len(), 64);
    assert!(!comment.local);
    assert_eq!(context.request_count(), 1);

    Comment::delete(&mut context.pool(), comment.id)
      .await
      .unwrap();
    cleanup(data, &context).await;
  }

  #[tokio::test]
  #[serial]
  async fn test_html_to_markdown_sanitize() {
    let parsed = parse_html("<script></script><b>hello</b>");
    assert_eq!(parsed, "**hello**");
  }
}
