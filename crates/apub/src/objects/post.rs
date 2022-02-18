use crate::{
  activities::{verify_is_public, verify_person_in_community},
  check_is_apub_id_valid,
  protocol::{
    objects::{
      page::{Page, PageType},
      tombstone::Tombstone,
    },
    ImageObject,
    Source,
  },
};
use activitystreams_kinds::public;
use chrono::NaiveDateTime;
use lemmy_api_common::blocking;
use lemmy_apub_lib::{
  object_id::ObjectId,
  traits::ApubObject,
  values::{MediaTypeHtml, MediaTypeMarkdown},
  verify::verify_domains_match,
};
use lemmy_db_schema::{
  self,
  source::{
    community::Community,
    person::Person,
    post::{Post, PostForm},
  },
  traits::Crud,
};
use lemmy_utils::{
  request::fetch_site_data,
  utils::{check_slurs, convert_datetime, markdown_to_html, remove_slurs},
  LemmyError,
};
use lemmy_websocket::LemmyContext;
use std::ops::Deref;
use url::Url;

#[derive(Clone, Debug)]
pub struct ApubPost(Post);

impl Deref for ApubPost {
  type Target = Post;
  fn deref(&self) -> &Self::Target {
    &self.0
  }
}

impl From<Post> for ApubPost {
  fn from(p: Post) -> Self {
    ApubPost { 0: p }
  }
}

#[async_trait::async_trait(?Send)]
impl ApubObject for ApubPost {
  type DataType = LemmyContext;
  type ApubType = Page;
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
        Post::read_from_apub_id(conn, object_id)
      })
      .await??
      .map(Into::into),
    )
  }

  #[tracing::instrument(skip_all)]
  async fn delete(self, context: &LemmyContext) -> Result<(), LemmyError> {
    if !self.deleted {
      blocking(context.pool(), move |conn| {
        Post::update_deleted(conn, self.id, true)
      })
      .await??;
    }
    Ok(())
  }

  // Turn a Lemmy post into an ActivityPub page that can be sent out over the network.
  #[tracing::instrument(skip_all)]
  async fn into_apub(self, context: &LemmyContext) -> Result<Page, LemmyError> {
    let creator_id = self.creator_id;
    let creator = blocking(context.pool(), move |conn| Person::read(conn, creator_id)).await??;
    let community_id = self.community_id;
    let community = blocking(context.pool(), move |conn| {
      Community::read(conn, community_id)
    })
    .await??;

    let source = self.body.clone().map(|body| Source {
      content: body,
      media_type: MediaTypeMarkdown::Markdown,
    });
    let image = self.thumbnail_url.clone().map(ImageObject::new);

    let page = Page {
      r#type: PageType::Page,
      id: ObjectId::new(self.ap_id.clone()),
      attributed_to: ObjectId::new(creator.actor_id),
      to: vec![community.actor_id.into(), public()],
      cc: vec![],
      name: self.name.clone(),
      content: self.body.as_ref().map(|b| markdown_to_html(b)),
      media_type: Some(MediaTypeHtml::Html),
      source,
      url: self.url.clone().map(|u| u.into()),
      image,
      comments_enabled: Some(!self.locked),
      sensitive: Some(self.nsfw),
      stickied: Some(self.stickied),
      published: Some(convert_datetime(self.published)),
      updated: self.updated.map(convert_datetime),
    };
    Ok(page)
  }

  fn to_tombstone(&self) -> Result<Tombstone, LemmyError> {
    Ok(Tombstone::new(self.ap_id.clone().into()))
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
    };

    let community = page.extract_community(context, request_counter).await?;
    check_is_apub_id_valid(page.id.inner(), community.local, &context.settings())?;
    verify_person_in_community(&page.attributed_to, &community, context, request_counter).await?;
    check_slurs(&page.name, &context.settings().slur_regex())?;
    verify_domains_match(page.attributed_to.inner(), page.id.inner())?;
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
      .attributed_to
      .dereference(context, context.client(), request_counter)
      .await?;
    let community = page.extract_community(context, request_counter).await?;

    let thumbnail_url: Option<Url> = page.image.map(|i| i.url);
    let (metadata_res, pictrs_thumbnail) = if let Some(url) = &page.url {
      fetch_site_data(context.client(), &context.settings(), Some(url)).await
    } else {
      (None, thumbnail_url)
    };
    let (embed_title, embed_description, embed_html) = metadata_res
      .map(|u| (u.title, u.description, u.html))
      .unwrap_or((None, None, None));

    let body_slurs_removed = page
      .source
      .as_ref()
      .map(|s| remove_slurs(&s.content, &context.settings().slur_regex()));
    let form = PostForm {
      name: page.name,
      url: page.url.map(|u| u.into()),
      body: body_slurs_removed,
      creator_id: creator.id,
      community_id: community.id,
      removed: None,
      locked: page.comments_enabled.map(|e| !e),
      published: page.published.map(|u| u.naive_local()),
      updated: page.updated.map(|u| u.naive_local()),
      deleted: None,
      nsfw: page.sensitive,
      stickied: page.stickied,
      embed_title,
      embed_description,
      embed_html,
      thumbnail_url: pictrs_thumbnail.map(|u| u.into()),
      ap_id: Some(page.id.into()),
      local: Some(false),
    };
    let post = blocking(context.pool(), move |conn| Post::upsert(conn, &form)).await??;
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
  use lemmy_apub_lib::activity_queue::create_activity_queue;
  use lemmy_db_schema::source::site::Site;
  use serial_test::serial;

  #[actix_rt::test]
  #[serial]
  async fn test_parse_lemmy_post() {
    let client = reqwest::Client::new().into();
    let manager = create_activity_queue(client);
    let context = init_context(manager.queue_handle().clone());
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
    assert_eq!(post.body.as_ref().unwrap().len(), 45);
    assert!(!post.locked);
    assert!(post.stickied);
    assert_eq!(request_counter, 0);

    Post::delete(&*context.pool().get().unwrap(), post.id).unwrap();
    Person::delete(&*context.pool().get().unwrap(), person.id).unwrap();
    Community::delete(&*context.pool().get().unwrap(), community.id).unwrap();
    Site::delete(&*context.pool().get().unwrap(), site.id).unwrap();
  }
}
