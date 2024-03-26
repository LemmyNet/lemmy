use crate::{
  activities::{verify_is_public, verify_person_in_community},
  check_apub_id_valid_with_strictness,
  local_site_data_cached,
  objects::{read_from_string_or_source_opt, verify_is_remote_object},
  protocol::{
    objects::{
      page::{Attachment, AttributedTo, Hashtag, HashtagType, Page, PageType},
      LanguageTag,
    },
    ImageObject,
    InCommunity,
    Source,
  },
};
use activitypub_federation::{
  config::Data,
  kinds::public,
  protocol::{values::MediaTypeMarkdownOrHtml, verification::verify_domains_match},
  traits::Object,
};
use anyhow::anyhow;
use chrono::{DateTime, Utc};
use html2text::{from_read_with_decorator, render::text_renderer::TrivialDecorator};
use lemmy_api_common::{
  context::LemmyContext,
  request::generate_post_link_metadata,
  utils::{
    get_url_blocklist,
    local_site_opt_to_slur_regex,
    process_markdown_opt,
    proxy_image_link_opt_apub,
  },
};
use lemmy_db_schema::{
  source::{
    community::Community,
    local_site::LocalSite,
    moderator::{ModLockPost, ModLockPostForm},
    person::Person,
    post::{Post, PostInsertForm, PostUpdateForm},
  },
  traits::Crud,
};
use lemmy_db_views_actor::structs::CommunityModeratorView;
use lemmy_utils::{
  error::LemmyError,
  utils::{markdown::markdown_to_html, slurs::check_slurs_opt, validation::check_url_scheme},
};
use std::ops::Deref;
use stringreader::StringReader;
use url::Url;

const MAX_TITLE_LENGTH: usize = 200;

#[derive(Clone, Debug, PartialEq)]
pub struct ApubPost(pub(crate) Post);

impl Deref for ApubPost {
  type Target = Post;
  fn deref(&self) -> &Self::Target {
    &self.0
  }
}

impl From<Post> for ApubPost {
  fn from(p: Post) -> Self {
    ApubPost(p)
  }
}

#[async_trait::async_trait]
impl Object for ApubPost {
  type DataType = LemmyContext;
  type Kind = Page;
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
      Post::read_from_apub_id(&mut context.pool(), object_id)
        .await?
        .map(Into::into),
    )
  }

  #[tracing::instrument(skip_all)]
  async fn delete(self, context: &Data<Self::DataType>) -> Result<(), LemmyError> {
    if !self.deleted {
      let form = PostUpdateForm {
        deleted: Some(true),
        ..Default::default()
      };
      Post::update(&mut context.pool(), self.id, &form).await?;
    }
    Ok(())
  }

  // Turn a Lemmy post into an ActivityPub page that can be sent out over the network.
  #[tracing::instrument(skip_all)]
  async fn into_json(self, context: &Data<Self::DataType>) -> Result<Page, LemmyError> {
    let creator_id = self.creator_id;
    let creator = Person::read(&mut context.pool(), creator_id).await?;
    let community_id = self.community_id;
    let community = Community::read(&mut context.pool(), community_id).await?;
    let language = LanguageTag::new_single(self.language_id, &mut context.pool()).await?;

    let attachment = self
      .url
      .clone()
      .map(|url| {
        Attachment::new(
          url.into(),
          self.url_content_type.clone(),
          self.alt_text.clone(),
        )
      })
      .into_iter()
      .collect();
    let hashtag = Hashtag {
      href: self.ap_id.clone().into(),
      name: format!("#{}", &community.name),
      kind: HashtagType::Hashtag,
    };

    let page = Page {
      kind: PageType::Page,
      id: self.ap_id.clone().into(),
      attributed_to: AttributedTo::Lemmy(creator.actor_id.into()),
      to: vec![community.actor_id.clone().into(), public()],
      cc: vec![],
      name: Some(self.name.clone()),
      content: self.body.as_ref().map(|b| markdown_to_html(b)),
      media_type: Some(MediaTypeMarkdownOrHtml::Html),
      source: self.body.clone().map(Source::new),
      attachment,
      image: self.thumbnail_url.clone().map(ImageObject::new),
      comments_enabled: Some(!self.locked),
      sensitive: Some(self.nsfw),
      language,
      published: Some(self.published),
      updated: self.updated,
      audience: Some(community.actor_id.into()),
      in_reply_to: None,
      tag: vec![hashtag],
    };
    Ok(page)
  }

  #[tracing::instrument(skip_all)]
  async fn verify(
    page: &Page,
    expected_domain: &Url,
    context: &Data<Self::DataType>,
  ) -> Result<(), LemmyError> {
    // We can't verify the domain in case of mod action, because the mod may be on a different
    // instance from the post author.
    if !page.is_mod_action(context).await? {
      verify_domains_match(page.id.inner(), expected_domain)?;
      verify_is_remote_object(page.id.inner(), context.settings())?;
    };

    let community = page.community(context).await?;
    check_apub_id_valid_with_strictness(page.id.inner(), community.local, context).await?;
    verify_person_in_community(&page.creator()?, &community, context).await?;

    let local_site_data = local_site_data_cached(&mut context.pool()).await?;
    let slur_regex = &local_site_opt_to_slur_regex(&local_site_data.local_site);
    check_slurs_opt(&page.name, slur_regex)?;

    verify_domains_match(page.creator()?.inner(), page.id.inner())?;
    verify_is_public(&page.to, &page.cc)?;
    Ok(())
  }

  #[tracing::instrument(skip_all)]
  async fn from_json(page: Page, context: &Data<Self::DataType>) -> Result<ApubPost, LemmyError> {
    let creator = page.creator()?.dereference(context).await?;
    let community = page.community(context).await?;
    if community.posting_restricted_to_mods {
      CommunityModeratorView::is_community_moderator(&mut context.pool(), community.id, creator.id)
        .await?;
    }
    let mut name = page
      .name
      .clone()
      .or_else(|| {
        // Posts coming from Mastodon or similar platforms don't have a title. Instead we take the
        // first line of the content and convert it from HTML to plaintext. We also remove mentions
        // of the community name.
        page
          .content
          .as_deref()
          .map(StringReader::new)
          .map(|c| from_read_with_decorator(c, MAX_TITLE_LENGTH, TrivialDecorator::new()))
          .and_then(|c| {
            c.lines().next().map(|s| {
              s.replace(&format!("@{}", community.name), "")
                .trim()
                .to_string()
            })
          })
      })
      .ok_or_else(|| anyhow!("Object must have name or content"))?;
    if name.chars().count() > MAX_TITLE_LENGTH {
      name = name.chars().take(MAX_TITLE_LENGTH).collect();
    }

    // read existing, local post if any (for generating mod log)
    let old_post = page.id.dereference_local(context).await;

    let first_attachment = page.attachment.first();
    let local_site = LocalSite::read(&mut context.pool()).await.ok();

    let form = if !page.is_mod_action(context).await? {
      let url = if let Some(attachment) = first_attachment.cloned() {
        Some(attachment.url())
      } else if page.kind == PageType::Video {
        // we cant display videos directly, so insert a link to external video page
        Some(page.id.inner().clone())
      } else {
        None
      };
      check_url_scheme(&url)?;

      let alt_text = first_attachment.cloned().and_then(Attachment::alt_text);

      let url = proxy_image_link_opt_apub(url, context).await?;

      let slur_regex = &local_site_opt_to_slur_regex(&local_site);
      let url_blocklist = get_url_blocklist(context).await?;

      let body = read_from_string_or_source_opt(&page.content, &page.media_type, &page.source);
      let body = process_markdown_opt(&body, slur_regex, &url_blocklist, context).await?;
      let language_id =
        LanguageTag::to_language_id_single(page.language, &mut context.pool()).await?;

      PostInsertForm::builder()
        .name(name)
        .url(url.map(Into::into))
        .body(body)
        .alt_text(alt_text)
        .creator_id(creator.id)
        .community_id(community.id)
        .locked(page.comments_enabled.map(|e| !e))
        .published(page.published.map(Into::into))
        .updated(page.updated.map(Into::into))
        .deleted(Some(false))
        .nsfw(page.sensitive)
        .ap_id(Some(page.id.clone().into()))
        .local(Some(false))
        .language_id(language_id)
        .build()
    } else {
      // if is mod action, only update locked/stickied fields, nothing else
      PostInsertForm::builder()
        .name(name)
        .creator_id(creator.id)
        .community_id(community.id)
        .ap_id(Some(page.id.clone().into()))
        .locked(page.comments_enabled.map(|e| !e))
        .updated(page.updated.map(Into::into))
        .build()
    };

    let post = Post::create(&mut context.pool(), &form).await?;

    generate_post_link_metadata(
      post.clone(),
      page.image.map(|i| i.url),
      |_| None,
      local_site,
      context.reset_request_count(),
    );

    // write mod log entry for lock
    if Page::is_locked_changed(&old_post, &page.comments_enabled) {
      let form = ModLockPostForm {
        mod_person_id: creator.id,
        post_id: post.id,
        locked: Some(post.locked),
      };
      ModLockPost::create(&mut context.pool(), &form).await?;
    }

    Ok(post.into())
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::{
    objects::{
      community::tests::parse_lemmy_community,
      person::{tests::parse_lemmy_person, ApubPerson},
    },
    protocol::tests::file_to_json_object,
  };
  use lemmy_db_schema::source::site::Site;
  use lemmy_utils::error::LemmyResult;
  use pretty_assertions::assert_eq;
  use serial_test::serial;

  #[tokio::test]
  #[serial]
  async fn test_parse_lemmy_post() -> LemmyResult<()> {
    let context = LemmyContext::init_test_context().await;
    let (person, site) = parse_lemmy_person(&context).await?;
    let community = parse_lemmy_community(&context).await?;

    let json = file_to_json_object("assets/lemmy/objects/page.json")?;
    let url = Url::parse("https://enterprise.lemmy.ml/post/55143")?;
    ApubPost::verify(&json, &url, &context).await?;
    let post = ApubPost::from_json(json, &context).await?;

    assert_eq!(post.ap_id, url.into());
    assert_eq!(post.name, "Post title");
    assert!(post.body.is_some());
    assert_eq!(post.body.as_ref().map(std::string::String::len), Some(45));
    assert!(!post.locked);
    assert!(!post.featured_community);
    assert_eq!(context.request_count(), 0);

    Post::delete(&mut context.pool(), post.id).await?;
    Person::delete(&mut context.pool(), person.id).await?;
    Community::delete(&mut context.pool(), community.id).await?;
    Site::delete(&mut context.pool(), site.id).await?;
    Ok(())
  }

  #[tokio::test]
  #[serial]
  async fn test_convert_mastodon_post_title() -> LemmyResult<()> {
    let context = LemmyContext::init_test_context().await;
    let community = parse_lemmy_community(&context).await?;

    let json = file_to_json_object("assets/mastodon/objects/person.json")?;
    let person = ApubPerson::from_json(json, &context).await?;

    let json = file_to_json_object("assets/mastodon/objects/page.json")?;
    let post = ApubPost::from_json(json, &context).await?;

    assert_eq!(post.name, "Variable never resetting at refresh");

    Post::delete(&mut context.pool(), post.id).await?;
    Person::delete(&mut context.pool(), person.id).await?;
    Community::delete(&mut context.pool(), community.id).await?;
    Ok(())
  }
}
