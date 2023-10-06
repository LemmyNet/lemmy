use crate::{
  activities::{verify_is_public, verify_person_in_community},
  check_apub_id_valid_with_strictness,
  local_site_data_cached,
  objects::{read_from_string_or_source_opt, verify_is_remote_object},
  protocol::{
    objects::{
      page::{Attachment, AttributedTo, Page, PageType},
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
use html2md::parse_html;
use lemmy_api_common::{
  context::LemmyContext,
  request::fetch_site_data,
  utils::{is_mod_or_admin, local_site_opt_to_sensitive, local_site_opt_to_slur_regex},
};
use lemmy_db_schema::{
  self,
  source::{
    community::Community,
    local_site::LocalSite,
    moderator::{ModLockPost, ModLockPostForm},
    person::Person,
    post::{Post, PostInsertForm, PostUpdateForm},
  },
  traits::Crud,
};
use lemmy_utils::{
  error::LemmyError,
  utils::{
    markdown::markdown_to_html,
    slurs::{check_slurs_opt, remove_slurs},
    time::convert_datetime,
    validation::check_url_scheme,
  },
};
use std::ops::Deref;
use url::Url;

const MAX_TITLE_LENGTH: usize = 200;

#[derive(Clone, Debug)]
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
      attachment: self.url.clone().map(Attachment::new).into_iter().collect(),
      image: self.thumbnail_url.clone().map(ImageObject::new),
      comments_enabled: Some(!self.locked),
      sensitive: Some(self.nsfw),
      language,
      published: Some(convert_datetime(self.published)),
      updated: self.updated.map(convert_datetime),
      audience: Some(community.actor_id.into()),
      in_reply_to: None,
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
      is_mod_or_admin(&mut context.pool(), creator.id, community.id).await?;
    }
    let mut name = page
      .name
      .clone()
      .or_else(|| {
        page
          .content
          .clone()
          .as_ref()
          .and_then(|c| parse_html(c).lines().next().map(ToString::to_string))
      })
      .ok_or_else(|| anyhow!("Object must have name or content"))?;
    if name.chars().count() > MAX_TITLE_LENGTH {
      name = name.chars().take(MAX_TITLE_LENGTH).collect();
    }

    // read existing, local post if any (for generating mod log)
    let old_post = page.id.dereference_local(context).await;

    let form = if !page.is_mod_action(context).await? {
      let first_attachment = page.attachment.into_iter().map(Attachment::url).next();
      let url = if first_attachment.is_some() {
        first_attachment
      } else if page.kind == PageType::Video {
        // we cant display videos directly, so insert a link to external video page
        Some(page.id.inner().clone())
      } else {
        None
      };
      check_url_scheme(&url)?;

      let local_site = LocalSite::read(&mut context.pool()).await.ok();
      let allow_sensitive = local_site_opt_to_sensitive(&local_site);
      let page_is_sensitive = page.sensitive.unwrap_or(false);
      let include_image = allow_sensitive || !page_is_sensitive;

      // Only fetch metadata if the post has a url and was not seen previously. We dont want to
      // waste resources by fetching metadata for the same post multiple times.
      // Additionally, only fetch image if content is not sensitive or is allowed on local site.
      let (metadata_res, thumbnail) = match &url {
        Some(url) if old_post.is_err() => {
          fetch_site_data(
            context.client(),
            context.settings(),
            Some(url),
            include_image,
          )
          .await
        }
        _ => (None, None),
      };
      // If no image was included with metadata, use post image instead when available.
      let thumbnail_url = thumbnail.or_else(|| page.image.map(|i| i.url.into()));

      let (embed_title, embed_description, embed_video_url) = metadata_res
        .map(|u| (u.title, u.description, u.embed_video_url))
        .unwrap_or_default();
      let slur_regex = &local_site_opt_to_slur_regex(&local_site);

      let body = read_from_string_or_source_opt(&page.content, &page.media_type, &page.source)
        .map(|s| remove_slurs(&s, slur_regex));
      let language_id =
        LanguageTag::to_language_id_single(page.language, &mut context.pool()).await?;

      PostInsertForm {
        name,
        url: url.map(Into::into),
        body,
        creator_id: creator.id,
        community_id: community.id,
        removed: None,
        locked: page.comments_enabled.map(|e| !e),
        published: page.published.map(Into::into),
        updated: page.updated.map(Into::into),
        deleted: Some(false),
        nsfw: page.sensitive,
        embed_title,
        embed_description,
        embed_video_url,
        thumbnail_url,
        ap_id: Some(page.id.clone().into()),
        local: Some(false),
        language_id,
        featured_community: None,
        featured_local: None,
      }
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
  #![allow(clippy::unwrap_used)]
  #![allow(clippy::indexing_slicing)]

  use super::*;
  use crate::{
    objects::{
      community::tests::parse_lemmy_community,
      person::tests::parse_lemmy_person,
      post::ApubPost,
      tests::init_context,
    },
    protocol::tests::file_to_json_object,
  };
  use lemmy_db_schema::source::site::Site;
  use serial_test::serial;

  #[tokio::test]
  #[serial]
  async fn test_parse_lemmy_post() {
    let context = init_context().await;
    let (person, site) = parse_lemmy_person(&context).await;
    let community = parse_lemmy_community(&context).await;

    let json = file_to_json_object("assets/lemmy/objects/page.json").unwrap();
    let url = Url::parse("https://enterprise.lemmy.ml/post/55143").unwrap();
    ApubPost::verify(&json, &url, &context).await.unwrap();
    let post = ApubPost::from_json(json, &context).await.unwrap();

    assert_eq!(post.ap_id, url.into());
    assert_eq!(post.name, "Post title");
    assert!(post.body.is_some());
    assert_eq!(post.body.as_ref().unwrap().len(), 45);
    assert!(!post.locked);
    assert!(!post.featured_community);
    assert_eq!(context.request_count(), 0);

    Post::delete(&mut context.pool(), post.id).await.unwrap();
    Person::delete(&mut context.pool(), person.id)
      .await
      .unwrap();
    Community::delete(&mut context.pool(), community.id)
      .await
      .unwrap();
    Site::delete(&mut context.pool(), site.id).await.unwrap();
  }
}
