use crate::{
  activities::{verify_is_public, verify_person_in_community},
  check_apub_id_valid_with_strictness,
  local_instance,
  objects::{read_from_string_or_source_opt, verify_is_remote_object},
  protocol::{
    objects::page::{Attachment, AttributedTo, Page, PageType},
    ImageObject,
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
use lemmy_api_common::{request::fetch_site_data, utils::blocking};
use lemmy_db_schema::{
  self,
  source::{
    community::Community,
    moderator::{ModLockPost, ModLockPostForm, ModStickyPost, ModStickyPostForm},
    person::Person,
    post::{Post, PostForm},
  },
  traits::Crud,
};
use lemmy_utils::{
  error::LemmyError,
  utils::{check_slurs, convert_datetime, markdown_to_html, remove_slurs},
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

    let page = Page {
      kind: PageType::Page,
      id: ObjectId::new(self.ap_id.clone()),
      attributed_to: AttributedTo::Lemmy(ObjectId::new(creator.actor_id)),
      to: vec![community.actor_id.into(), public()],
      cc: vec![],
      name: self.name.clone(),
      content: self.body.as_ref().map(|b| markdown_to_html(b)),
      media_type: Some(MediaTypeMarkdownOrHtml::Html),
      source: self.body.clone().map(Source::new),
      url: self.url.clone().map(|u| u.into()),
      attachment: self.url.clone().map(Attachment::new).into_iter().collect(),
      image: self.thumbnail_url.clone().map(ImageObject::new),
      comments_enabled: Some(!self.locked),
      sensitive: Some(self.nsfw),
      stickied: Some(self.stickied),
      published: Some(convert_datetime(self.published)),
      updated: self.updated.map(convert_datetime),
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

    let community = page.extract_community(context, request_counter).await?;
    check_apub_id_valid_with_strictness(page.id.inner(), community.local, context.settings())?;
    verify_person_in_community(&page.creator()?, &community, context, request_counter).await?;
    check_slurs(&page.name, &context.settings().slur_regex())?;
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
      .dereference(context, local_instance(context), request_counter)
      .await?;
    let community = page.extract_community(context, request_counter).await?;

    let form = if !page.is_mod_action(context).await? {
      let url = if let Some(attachment) = page.attachment.first() {
        // url as sent by Lemmy (new)
        Some(attachment.href.clone())
      } else if page.kind == PageType::Video {
        // we cant display videos directly, so insert a link to external video page
        Some(page.id.inner().clone())
      } else {
        // url sent by lemmy (old)
        page.url
      };
      let (metadata_res, thumbnail_url) = if let Some(url) = &url {
        fetch_site_data(context.client(), context.settings(), Some(url)).await
      } else {
        (None, page.image.map(|i| i.url.into()))
      };
      let (embed_title, embed_description, embed_video_url) = metadata_res
        .map(|u| (Some(u.title), Some(u.description), Some(u.embed_video_url)))
        .unwrap_or_default();
      let body_slurs_removed =
        read_from_string_or_source_opt(&page.content, &page.media_type, &page.source)
          .map(|s| Some(remove_slurs(&s, &context.settings().slur_regex())));

      PostForm {
        name: page.name.clone(),
        url: Some(url.map(Into::into)),
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
        embed_video_url,
        thumbnail_url: Some(thumbnail_url),
        ap_id: Some(page.id.clone().into()),
        local: Some(false),
      }
    } else {
      // if is mod action, only update locked/stickied fields, nothing else
      PostForm {
        name: page.name.clone(),
        creator_id: creator.id,
        community_id: community.id,
        locked: page.comments_enabled.map(|e| !e),
        stickied: page.stickied,
        updated: page.updated.map(|u| u.naive_local()),
        ap_id: Some(page.id.clone().into()),
        ..Default::default()
      }
    };

    // read existing, local post if any (for generating mod log)
    let old_post = ObjectId::<ApubPost>::new(page.id.clone())
      .dereference_local(context)
      .await;

    let post = blocking(context.pool(), move |conn| Post::upsert(conn, &form)).await??;

    // write mod log entries for sticky/lock
    if Page::is_stickied_changed(&old_post, &page.stickied) {
      let form = ModStickyPostForm {
        mod_person_id: creator.id,
        post_id: post.id,
        stickied: Some(post.stickied),
      };
      blocking(context.pool(), move |conn| {
        ModStickyPost::create(conn, &form)
      })
      .await??;
    }
    if Page::is_locked_changed(&old_post, &page.comments_enabled) {
      let form = ModLockPostForm {
        mod_person_id: creator.id,
        post_id: post.id,
        locked: Some(post.locked),
      };
      blocking(context.pool(), move |conn| ModLockPost::create(conn, &form)).await??;
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
    let context = init_context();
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
