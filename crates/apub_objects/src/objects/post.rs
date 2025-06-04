use crate::{
  protocol::page::{
    Attachment,
    Hashtag,
    HashtagType::{self},
    Page,
    PageType,
  },
  utils::{
    functions::{
      check_apub_id_valid_with_strictness,
      generate_to,
      read_from_string_or_source_opt,
      verify_person_in_community,
      verify_visibility,
    },
    markdown_links::{markdown_rewrite_remote_links_opt, to_local_url},
    protocol::{AttributedTo, ImageObject, InCommunity, LanguageTag, Source},
  },
};
use activitypub_federation::{
  config::Data,
  protocol::{
    values::MediaTypeMarkdownOrHtml,
    verification::{verify_domains_match, verify_is_remote_object},
  },
  traits::Object,
};
use anyhow::anyhow;
use chrono::{DateTime, Utc};
use html2text::{from_read_with_decorator, render::TrivialDecorator};
use lemmy_api_utils::{
  context::LemmyContext,
  plugins::{plugin_hook_after, plugin_hook_before},
  request::generate_post_link_metadata,
  utils::{check_nsfw_allowed, get_url_blocklist, process_markdown_opt, slur_regex},
};
use lemmy_db_schema::{
  source::{
    community::Community,
    person::Person,
    post::{Post, PostInsertForm, PostUpdateForm},
  },
  traits::Crud,
};
use lemmy_db_views_community_moderator::CommunityModeratorView;
use lemmy_db_views_site::SiteView;
use lemmy_utils::{
  error::{LemmyError, LemmyResult},
  spawn_try_task,
  utils::{
    markdown::markdown_to_html,
    slurs::check_slurs_opt,
    validation::{is_url_blocked, is_valid_url},
  },
};
use std::ops::Deref;
use stringreader::StringReader;
use url::Url;

const MAX_TITLE_LENGTH: usize = 200;

#[derive(Clone, Debug, PartialEq)]
pub struct ApubPost(pub Post);

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

  async fn read_from_id(
    object_id: Url,
    context: &Data<Self::DataType>,
  ) -> LemmyResult<Option<Self>> {
    Ok(
      Post::read_from_apub_id(&mut context.pool(), object_id)
        .await?
        .map(Into::into),
    )
  }

  async fn delete(self, context: &Data<Self::DataType>) -> LemmyResult<()> {
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

  async fn into_json(self, context: &Data<Self::DataType>) -> LemmyResult<Page> {
    let creator_id = self.creator_id;
    let creator = Person::read(&mut context.pool(), creator_id).await?;
    let community_id = self.community_id;
    let community = Community::read(&mut context.pool(), community_id).await?;
    let language = Some(LanguageTag::new_single(self.language_id, &mut context.pool()).await?);

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
      attributed_to: AttributedTo::Lemmy(creator.ap_id.into()),
      to: generate_to(&community)?,
      cc: vec![],
      name: Some(self.name.clone()),
      content: self.body.as_ref().map(|b| markdown_to_html(b)),
      media_type: Some(MediaTypeMarkdownOrHtml::Html),
      source: self.body.clone().map(Source::new),
      attachment,
      image: self.thumbnail_url.clone().map(ImageObject::new),
      sensitive: Some(self.nsfw),
      language,
      published: Some(self.published),
      updated: self.updated,
      in_reply_to: None,
      tag: vec![hashtag],
    };
    Ok(page)
  }

  async fn verify(
    page: &Page,
    expected_domain: &Url,
    context: &Data<Self::DataType>,
  ) -> LemmyResult<()> {
    verify_domains_match(page.id.inner(), expected_domain)?;
    if let Err(e) = verify_is_remote_object(&page.id, context) {
      if let Ok(post) = page.id.dereference_local(context).await {
        post.set_not_pending(&mut context.pool()).await?;
      }
      return Err(e.into());
    }

    let community = page.community(context).await?;
    check_apub_id_valid_with_strictness(page.id.inner(), community.local, context).await?;
    verify_person_in_community(&page.creator()?, &community, context).await?;

    let slur_regex = slur_regex(context).await?;
    check_slurs_opt(&page.name, &slur_regex)?;

    verify_domains_match(page.creator()?.inner(), page.id.inner())?;
    verify_visibility(&page.to, &page.cc, &community)?;
    Ok(())
  }

  async fn from_json(page: Page, context: &Data<Self::DataType>) -> LemmyResult<ApubPost> {
    let local_site = SiteView::read_local(&mut context.pool())
      .await
      .ok()
      .map(|s| s.local_site);
    let creator = page.creator()?.dereference(context).await?;
    let community = page.community(context).await?;

    // Prevent posts from non-mod users in local, restricted community. If its a remote community
    // then its possible that the restricted setting was enabled recently, so existing user posts
    // should still be fetched.
    if community.local && community.posting_restricted_to_mods {
      CommunityModeratorView::check_is_community_moderator(
        &mut context.pool(),
        community.id,
        creator.id,
      )
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
            c.unwrap_or_default().lines().next().map(|s| {
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

    let first_attachment = page.attachment.first();
    let url = if let Some(attachment) = first_attachment.cloned() {
      Some(attachment.url())
    } else if page.kind == PageType::Video {
      // we cant display videos directly, so insert a link to external video page
      Some(page.id.inner().clone())
    } else {
      None
    };

    // Ensure that all posts in NSFW communities are marked as NSFW
    let nsfw = if community.nsfw {
      Some(true)
    } else {
      page.sensitive
    };

    // If NSFW is not allowed, reject NSFW posts and delete existing
    // posts that get updated to be NSFW
    let block_for_nsfw = check_nsfw_allowed(nsfw, local_site.as_ref());
    if let Err(e) = block_for_nsfw {
      // TODO: Remove locally generated thumbnail if one exists, depends on
      //       https://github.com/LemmyNet/lemmy/issues/5564 to be implemented to be able to
      //       safely do this.
      Post::delete_from_apub_id(&mut context.pool(), page.id.inner().clone()).await?;
      Err(e)?
    }

    let url_blocklist = get_url_blocklist(context).await?;

    let url = if let Some(url) = url {
      is_url_blocked(&url, &url_blocklist)?;
      is_valid_url(&url)?;
      if page.kind != PageType::Video {
        to_local_url(url.as_str(), context).await.or(Some(url))
      } else {
        Some(url)
      }
    } else {
      None
    };

    let alt_text = first_attachment.cloned().and_then(Attachment::alt_text);

    let slur_regex = slur_regex(context).await?;

    let body = read_from_string_or_source_opt(&page.content, &page.media_type, &page.source);
    let body = process_markdown_opt(&body, &slur_regex, &url_blocklist, context).await?;
    let body = markdown_rewrite_remote_links_opt(body, context).await;
    let language_id = Some(
      LanguageTag::to_language_id_single(page.language.unwrap_or_default(), &mut context.pool())
        .await?,
    );

    let mut form = PostInsertForm {
      url: url.map(Into::into),
      body,
      alt_text,
      published: page.published,
      updated: page.updated,
      deleted: Some(false),
      nsfw,
      ap_id: Some(page.id.clone().into()),
      local: Some(false),
      language_id,
      ..PostInsertForm::new(name, creator.id, community.id)
    };
    form = plugin_hook_before("before_receive_federated_post", form).await?;

    let timestamp = page.updated.or(page.published).unwrap_or_else(Utc::now);
    let post = Post::insert_apub(&mut context.pool(), timestamp, &form).await?;
    plugin_hook_after("after_receive_federated_post", &post)?;
    let post_ = post.clone();
    let context_ = context.reset_request_count();

    // Generates a post thumbnail in background task, because some sites can be very slow to
    // respond.
    spawn_try_task(
      async move { generate_post_link_metadata(post_, None, |_| None, context_).await },
    );

    Ok(post.into())
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::{
    objects::ApubPerson,
    utils::test::{file_to_json_object, parse_lemmy_community, parse_lemmy_person},
  };
  use lemmy_db_schema::source::site::Site;
  use pretty_assertions::assert_eq;
  use serial_test::serial;

  #[tokio::test]
  #[serial]
  async fn test_parse_lemmy_post() -> LemmyResult<()> {
    let context = LemmyContext::init_test_context().await;
    let (person, site) = parse_lemmy_person(&context).await?;
    let community = parse_lemmy_community(&context).await?;

    let json = file_to_json_object("../apub/assets/lemmy/objects/page.json")?;
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

    let json = file_to_json_object("../apub/assets/mastodon/objects/person.json")?;
    let person = ApubPerson::from_json(json, &context).await?;

    let json = file_to_json_object("../apub/assets/mastodon/objects/page.json")?;
    let post = ApubPost::from_json(json, &context).await?;

    assert_eq!(post.name, "Variable never resetting at refresh");

    Post::delete(&mut context.pool(), post.id).await?;
    Person::delete(&mut context.pool(), person.id).await?;
    Community::delete(&mut context.pool(), community.id).await?;
    Ok(())
  }
}
