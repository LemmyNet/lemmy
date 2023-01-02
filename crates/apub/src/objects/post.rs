use crate::{
  activities::{verify_is_public, verify_person_in_community},
  check_apub_id_valid_with_strictness,
  fetch_local_site_data,
  local_instance,
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
  core::object_id::ObjectId,
  deser::values::MediaTypeMarkdownOrHtml,
  traits::ApubObject,
  utils::verify_domains_match,
};
use activitystreams_kinds::public;
use chrono::NaiveDateTime;
use lemmy_api_common::{
  context::LemmyContext,
  request::fetch_site_data,
  utils::local_site_opt_to_slur_regex,
};
use lemmy_db_schema::{
  self,
  source::{
    community::Community,
    local_site::LocalSite,
    moderator::{ModFeaturePost, ModFeaturePostForm, ModLockPost, ModLockPostForm},
    person::Person,
    post::{Post, PostInsertForm, PostUpdateForm},
  },
  traits::Crud,
};
use lemmy_utils::{
  error::LemmyError,
  utils::{check_slurs, convert_datetime, markdown_to_html, remove_slurs},
};
use std::ops::Deref;
use url::Url;

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

#[async_trait::async_trait(?Send)]
impl ApubObject for ApubPost {
  type DataType = LemmyContext;
  type ApubType = Page;
  type DbType = Post;
  type Error = LemmyError;

  fn last_refreshed_at(&self) -> Option<NaiveDateTime> {
    None
  }

  #[tracing::instrument(skip_all)]
  async fn read_from_apub_id(
    object_id: Url,
    context: &LemmyContext,
  ) -> Result<Option<Self>, LemmyError> {
    Ok(
      Post::read_from_apub_id(context.pool(), object_id)
        .await?
        .map(Into::into),
    )
  }

  #[tracing::instrument(skip_all)]
  async fn delete(self, context: &LemmyContext) -> Result<(), LemmyError> {
    if !self.deleted {
      let form = PostUpdateForm::builder().deleted(Some(true)).build();
      Post::update(context.pool(), self.id, &form).await?;
    }
    Ok(())
  }

  // Turn a Lemmy post into an ActivityPub page that can be sent out over the network.
  #[tracing::instrument(skip_all)]
  async fn into_apub(self, context: &LemmyContext) -> Result<Page, LemmyError> {
    let creator_id = self.creator_id;
    let creator = Person::read(context.pool(), creator_id).await?;
    let community_id = self.community_id;
    let community = Community::read(context.pool(), community_id).await?;
    let language = LanguageTag::new_single(self.language_id, context.pool()).await?;

    let page = Page {
      kind: PageType::Page,
      id: ObjectId::new(self.ap_id.clone()),
      attributed_to: AttributedTo::Lemmy(ObjectId::new(creator.actor_id)),
      to: vec![community.actor_id.clone().into(), public()],
      cc: vec![],
      name: self.name.clone(),
      content: self.body.as_ref().map(|b| markdown_to_html(b)),
      media_type: Some(MediaTypeMarkdownOrHtml::Html),
      source: self.body.clone().map(Source::new),
      attachment: self.url.clone().map(Attachment::new).into_iter().collect(),
      image: self.thumbnail_url.clone().map(ImageObject::new),
      comments_enabled: Some(!self.locked),
      sensitive: Some(self.nsfw),
      stickied: Some(self.featured_community),
      language,
      published: Some(convert_datetime(self.published)),
      updated: self.updated.map(convert_datetime),
      audience: Some(ObjectId::new(community.actor_id)),
      // TODO: write emojis which are used in post
      tag: vec![]
    };
    Ok(page)
  }

  #[tracing::instrument(skip_all)]
  async fn verify(
    page: &Page,
    expected_domain: &Url,
    context: &LemmyContext,
    request_counter: &mut i32,
  ) -> Result<(), LemmyError> {
    // We can't verify the domain in case of mod action, because the mod may be on a different
    // instance from the post author.
    if !page.is_mod_action(context).await? {
      verify_domains_match(page.id.inner(), expected_domain)?;
      verify_is_remote_object(page.id.inner(), context.settings())?;
    };

    let local_site_data = fetch_local_site_data(context.pool()).await?;

    let community = page.community(context, request_counter).await?;
    check_apub_id_valid_with_strictness(
      page.id.inner(),
      community.local,
      &local_site_data,
      context.settings(),
    )?;
    verify_person_in_community(&page.creator()?, &community, context, request_counter).await?;

    let slur_regex = &local_site_opt_to_slur_regex(&local_site_data.local_site);
    check_slurs(&page.name, slur_regex)?;

    verify_domains_match(page.creator()?.inner(), page.id.inner())?;
    verify_is_public(&page.to, &page.cc)?;
    Ok(())
  }

  #[tracing::instrument(skip_all)]
  async fn from_apub(
    page: Page,
    context: &LemmyContext,
    request_counter: &mut i32,
  ) -> Result<ApubPost, LemmyError> {
    let creator = page
      .creator()?
      .dereference(context, local_instance(context).await, request_counter)
      .await?;
    let community = page.community(context, request_counter).await?;

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
      let (metadata_res, thumbnail_url) = if let Some(url) = &url {
        fetch_site_data(context.client(), context.settings(), Some(url)).await
      } else {
        (None, page.image.map(|i| i.url.into()))
      };
      let (embed_title, embed_description, embed_video_url) = metadata_res
        .map(|u| (u.title, u.description, u.embed_video_url))
        .unwrap_or_default();
      let local_site = LocalSite::read(context.pool()).await.ok();
      let slur_regex = &local_site_opt_to_slur_regex(&local_site);

      let body_slurs_removed =
        read_from_string_or_source_opt(&page.content, &page.media_type, &page.source)
          .map(|s| remove_slurs(&s, slur_regex));
      let language_id = LanguageTag::to_language_id_single(page.language, context.pool()).await?;

      // TODO: read emojis which are used in post from page.tag, and write them to db

      PostInsertForm {
        name: page.name.clone(),
        url: url.map(Into::into),
        body: body_slurs_removed,
        creator_id: creator.id,
        community_id: community.id,
        removed: None,
        locked: page.comments_enabled.map(|e| !e),
        published: page.published.map(|u| u.naive_local()),
        updated: page.updated.map(|u| u.naive_local()),
        deleted: Some(false),
        nsfw: page.sensitive,
        embed_title,
        embed_description,
        embed_video_url,
        thumbnail_url,
        ap_id: Some(page.id.clone().into()),
        local: Some(false),
        language_id,
        featured_community: page.stickied,
        featured_local: None,
      }
    } else {
      // if is mod action, only update locked/stickied fields, nothing else
      PostInsertForm::builder()
        .name(page.name.clone())
        .creator_id(creator.id)
        .community_id(community.id)
        .ap_id(Some(page.id.clone().into()))
        .locked(page.comments_enabled.map(|e| !e))
        .featured_community(page.stickied)
        .updated(page.updated.map(|u| u.naive_local()))
        .build()
    };
    // read existing, local post if any (for generating mod log)
    let old_post = ObjectId::<ApubPost>::new(page.id.clone())
      .dereference_local(context)
      .await;

    let post = Post::create(context.pool(), &form).await?;

    // write mod log entries for feature/lock
    if Page::is_featured_changed(&old_post, &page.stickied) {
      let form = ModFeaturePostForm {
        mod_person_id: creator.id,
        post_id: post.id,
        featured: post.featured_community,
        is_featured_community: true,
      };
      ModFeaturePost::create(context.pool(), &form).await?;
    }
    if Page::is_locked_changed(&old_post, &page.comments_enabled) {
      let form = ModLockPostForm {
        mod_person_id: creator.id,
        post_id: post.id,
        locked: Some(post.locked),
      };
      ModLockPost::create(context.pool(), &form).await?;
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
      person::tests::parse_lemmy_person,
      post::ApubPost,
      tests::init_context,
    },
    protocol::tests::file_to_json_object,
  };
  use lemmy_db_schema::source::site::Site;
  use serial_test::serial;

  #[actix_rt::test]
  #[serial]
  async fn test_parse_lemmy_post() {
    let context = init_context().await;
    let (person, site) = parse_lemmy_person(&context).await;
    let community = parse_lemmy_community(&context).await;

    let json = file_to_json_object("assets/lemmy/objects/page.json").unwrap();
    let url = Url::parse("https://enterprise.lemmy.ml/post/55143").unwrap();
    let mut request_counter = 0;
    ApubPost::verify(&json, &url, &context, &mut request_counter)
      .await
      .unwrap();
    let post = ApubPost::from_apub(json, &context, &mut request_counter)
      .await
      .unwrap();

    assert_eq!(post.ap_id, url.into());
    assert_eq!(post.name, "Post title");
    assert!(post.body.is_some());
    assert_eq!(post.body.as_ref().unwrap().len(), 53);
    assert!(!post.locked);
    assert!(post.featured_community);
    assert_eq!(request_counter, 0);

    Post::delete(context.pool(), post.id).await.unwrap();
    Person::delete(context.pool(), person.id).await.unwrap();
    Community::delete(context.pool(), community.id)
      .await
      .unwrap();
    Site::delete(context.pool(), site.id).await.unwrap();
  }
}
