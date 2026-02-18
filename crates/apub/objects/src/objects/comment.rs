use crate::{
  protocol::note::Note,
  utils::{
    functions::{
      append_attachments_to_comment,
      check_apub_id_valid_with_strictness,
      context_url,
      generate_to,
      read_from_string_or_source,
      verify_person_in_community,
      verify_visibility,
    },
    markdown_links::markdown_rewrite_remote_links,
    mentions::{collect_non_local_mentions, get_comment_parent_creator},
    protocol::{InCommunity, LanguageTag, Source},
  },
};
use activitypub_federation::{
  config::Data,
  kinds::object::NoteType,
  protocol::{
    values::MediaTypeMarkdownOrHtml,
    verification::{verify_domains_match, verify_is_remote_object},
  },
  traits::Object,
};
use chrono::{DateTime, Utc};
use lemmy_api_utils::{
  context::LemmyContext,
  plugins::{plugin_hook_after, plugin_hook_before},
  utils::{
    check_comment_depth,
    check_is_mod_or_admin,
    get_url_blocklist,
    process_markdown,
    slur_regex,
  },
};
use lemmy_db_schema::source::{
  comment::{Comment, CommentInsertForm, CommentUpdateForm},
  community::Community,
  person::Person,
  post::Post,
};
use lemmy_diesel_utils::traits::Crud;
use lemmy_utils::{
  error::{LemmyError, LemmyResult, UntranslatedError},
  utils::markdown::markdown_to_html,
};
use std::ops::Deref;
use url::Url;

#[derive(Clone, Debug)]
pub struct ApubComment(pub Comment);

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

  fn id(&self) -> &Url {
    self.ap_id.inner()
  }

  async fn read_from_id(
    object_id: Url,
    context: &Data<Self::DataType>,
  ) -> LemmyResult<Option<Self>> {
    Ok(
      Comment::read_from_apub_id(&mut context.pool(), object_id.into())
        .await?
        .map(Into::into),
    )
  }

  async fn delete(&self, context: &Data<Self::DataType>) -> LemmyResult<()> {
    if !self.deleted {
      let form = CommentUpdateForm {
        deleted: Some(true),
        ..Default::default()
      };
      Comment::update(&mut context.pool(), self.id, &form).await?;
    }
    Ok(())
  }

  fn is_deleted(&self) -> bool {
    self.removed || self.deleted
  }

  async fn into_json(self, context: &Data<Self::DataType>) -> LemmyResult<Note> {
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
      post.ap_id.clone().into()
    };
    let language = Some(LanguageTag::new_single(self.language_id, &mut context.pool()).await?);
    // Make this call optional in case the account was deleted.
    let parent_creator = get_comment_parent_creator(&mut context.pool(), &self)
      .await
      .ok();
    let maa = collect_non_local_mentions(Some(&self.content), parent_creator, context).await?;

    let note = Note {
      r#type: NoteType::Note,
      id: self.ap_id.clone().into(),
      attributed_to: creator.ap_id.into(),
      to: generate_to(&community)?,
      cc: maa.ccs,
      content: markdown_to_html(&self.content),
      media_type: Some(MediaTypeMarkdownOrHtml::Html),
      source: Some(Source::new(self.content.clone())),
      in_reply_to,
      published: Some(self.published_at),
      updated: self.updated_at,
      tag: maa.mentions,
      distinguished: Some(self.distinguished),
      language,
      audience: Some(community.ap_id.into()),
      attachment: vec![],
      context: Some(context_url(&self.ap_id)),
    };

    Ok(note)
  }

  /// Recursively fetches all parent comments. This can lead to a stack overflow so we need to
  /// Box::pin all large futures on the heap.
  async fn verify(
    note: &Note,
    expected_domain: &Url,
    context: &Data<LemmyContext>,
  ) -> LemmyResult<()> {
    verify_domains_match(note.id.inner(), expected_domain)?;
    verify_domains_match(note.attributed_to.inner(), note.id.inner())?;
    let community = Box::pin(note.community(context)).await?;
    verify_visibility(&note.to, &note.cc, &community)?;

    Box::pin(check_apub_id_valid_with_strictness(
      note.id.inner(),
      community.local,
      context,
    ))
    .await?;
    if let Err(e) = verify_is_remote_object(&note.id, context) {
      if let Ok(comment) = note.id.dereference_local(context).await {
        comment.set_not_pending(&mut context.pool()).await?;
      }
      return Err(e.into());
    }
    Box::pin(verify_person_in_community(
      &note.attributed_to,
      &community,
      context,
    ))
    .await?;

    let (post, parent_comment) = Box::pin(note.get_parents(context)).await?;
    let creator = Box::pin(note.attributed_to.dereference(context)).await?;

    let is_mod_or_admin = check_is_mod_or_admin(&mut context.pool(), creator.id, community.id)
      .await
      .is_ok();
    let locked = post.locked || parent_comment.is_some_and(|c| c.locked);
    if locked && !is_mod_or_admin {
      Err(UntranslatedError::PostIsLocked)?
    } else {
      Ok(())
    }
  }

  /// Converts a `Note` to `Comment`.
  ///
  /// If the parent community, post and comment(s) are not known locally, these are also fetched.
  async fn from_json(note: Note, context: &Data<LemmyContext>) -> LemmyResult<ApubComment> {
    let creator = note.attributed_to.dereference(context).await?;
    let (post, parent_comment) = note.get_parents(context).await?;
    if let Some(c) = &parent_comment {
      check_comment_depth(c)?;
    }

    let content = read_from_string_or_source(&note.content, &note.media_type, &note.source);

    let slur_regex = slur_regex(context).await?;
    let url_blocklist = get_url_blocklist(context).await?;
    let content = append_attachments_to_comment(content, &note.attachment, context).await?;
    let content = process_markdown(&content, &slur_regex, &url_blocklist, context).await?;
    let content = markdown_rewrite_remote_links(content, context).await;
    let language_id = Some(
      LanguageTag::to_language_id_single(note.language.unwrap_or_default(), &mut context.pool())
        .await?,
    );

    let mut form = CommentInsertForm {
      creator_id: creator.id,
      post_id: post.id,
      content,
      removed: None,
      published_at: note.published,
      updated_at: note.updated,
      deleted: Some(false),
      ap_id: Some(note.id.into()),
      distinguished: note.distinguished,
      local: Some(false),
      language_id,
      federation_pending: Some(false),
      locked: None,
    };
    form = plugin_hook_before("federated_comment_before_receive", form).await?;
    let parent_comment_path = parent_comment.map(|t| t.0.path);
    let timestamp: DateTime<Utc> = note.updated.or(note.published).unwrap_or_else(Utc::now);
    let comment = Comment::insert_apub(
      &mut context.pool(),
      Some(timestamp),
      &form,
      parent_comment_path.as_ref(),
    )
    .await?;
    plugin_hook_after("federated_comment_after_receive", &comment);
    Ok(comment.into())
  }
}

#[cfg(test)]
pub(crate) mod tests {
  use super::*;
  use crate::{
    objects::{community::ApubCommunity, instance::ApubSite, person::ApubPerson, post::ApubPost},
    utils::test::{file_to_json_object, parse_lemmy_community, parse_lemmy_person},
  };
  use assert_json_diff::assert_json_include;
  use html2md::parse_html;
  use lemmy_db_schema::{source::instance::Instance, test_data::TestData};
  use pretty_assertions::assert_eq;
  use serial_test::serial;

  async fn prepare_comment_test(
    url: &Url,
    context: &Data<LemmyContext>,
  ) -> LemmyResult<(ApubPerson, ApubCommunity, ApubPost, ApubSite)> {
    // use separate counter so this doesn't affect tests
    let context2 = context.clone();
    let (person, site) = parse_lemmy_person(&context2).await?;
    let community = parse_lemmy_community(&context2).await?;
    let post_json = file_to_json_object("../apub/assets/lemmy/objects/page.json")?;
    ApubPost::verify(&post_json, url, &context2).await?;
    let post = ApubPost::from_json(post_json, &context2).await?;
    Ok((person, community, post, site))
  }

  #[tokio::test]
  #[serial]
  pub(crate) async fn test_parse_lemmy_comment() -> LemmyResult<()> {
    let context = LemmyContext::init_test_context().await;
    let test_data = TestData::create(&mut context.pool()).await?;
    let url = Url::parse("https://enterprise.lemmy.ml/comment/38741")?;
    prepare_comment_test(&url, &context).await?;

    let json: Note = file_to_json_object("../apub/assets/lemmy/objects/comment.json")?;
    ApubComment::verify(&json, &url, &context).await?;
    let comment = ApubComment::from_json(json.clone(), &context).await?;

    assert_eq!(comment.ap_id, url.into());
    assert_eq!(comment.content.len(), 14);
    assert!(!comment.local);
    assert_eq!(context.request_count(), 0);

    let to_apub = comment.into_json(&context).await?;
    assert_json_include!(actual: json, expected: to_apub);

    test_data.delete(&mut context.pool()).await?;
    Instance::delete_all(&mut context.pool()).await?;
    Ok(())
  }

  #[tokio::test]
  #[serial]
  async fn test_parse_pleroma_comment() -> LemmyResult<()> {
    let context = LemmyContext::init_test_context().await;
    let test_data = TestData::create(&mut context.pool()).await?;
    let url = Url::parse("https://enterprise.lemmy.ml/comment/38741")?;
    prepare_comment_test(&url, &context).await?;

    let pleroma_url =
      Url::parse("https://queer.hacktivis.me/objects/8d4973f4-53de-49cd-8c27-df160e16a9c2")?;
    let person_json = file_to_json_object("../apub/assets/pleroma/objects/person.json")?;
    ApubPerson::verify(&person_json, &pleroma_url, &context).await?;
    ApubPerson::from_json(person_json, &context).await?;
    let json = file_to_json_object("../apub/assets/pleroma/objects/note.json")?;
    ApubComment::verify(&json, &pleroma_url, &context).await?;
    let comment = ApubComment::from_json(json, &context).await?;

    assert_eq!(comment.ap_id, pleroma_url.into());
    assert_eq!(comment.content.len(), 10);
    assert!(!comment.local);
    assert_eq!(context.request_count(), 1);

    test_data.delete(&mut context.pool()).await?;
    Instance::delete_all(&mut context.pool()).await?;
    Ok(())
  }

  #[tokio::test]
  #[serial]
  async fn test_html_to_markdown_sanitize() {
    let parsed = parse_html("<script></script><b>hello</b>");
    assert_eq!(parsed, "**hello**");
  }
}
