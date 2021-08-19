use crate::{
  activities::{extract_community, verify_person_in_community},
  extensions::context::lemmy_context,
  fetcher::person::get_or_fetch_and_upsert_person,
  objects::{create_tombstone, FromApub, ImageObject, Source, ToApub},
  ActorType,
};
use activitystreams::{
  base::AnyBase,
  object::{
    kind::{ImageType, PageType},
    Tombstone,
  },
  primitives::OneOrMany,
  public,
  unparsed::Unparsed,
};
use chrono::{DateTime, FixedOffset};
use lemmy_api_common::blocking;
use lemmy_apub_lib::{
  values::{MediaTypeHtml, MediaTypeMarkdown},
  verify_domains_match,
};
use lemmy_db_queries::{ApubObject, Crud, DbPool};
use lemmy_db_schema::{
  self,
  source::{
    community::Community,
    person::Person,
    post::{Post, PostForm},
  },
};
use lemmy_utils::{
  request::fetch_site_data,
  utils::{check_slurs, convert_datetime, markdown_to_html, remove_slurs},
  LemmyError,
};
use lemmy_websocket::LemmyContext;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
use url::Url;

#[skip_serializing_none]
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Page {
  #[serde(rename = "@context")]
  context: OneOrMany<AnyBase>,
  r#type: PageType,
  id: Url,
  pub(crate) attributed_to: Url,
  to: [Url; 2],
  name: String,
  content: Option<String>,
  media_type: MediaTypeHtml,
  source: Option<Source>,
  url: Option<Url>,
  image: Option<ImageObject>,
  pub(crate) comments_enabled: Option<bool>,
  sensitive: Option<bool>,
  pub(crate) stickied: Option<bool>,
  published: DateTime<FixedOffset>,
  updated: Option<DateTime<FixedOffset>>,
  #[serde(flatten)]
  unparsed: Unparsed,
}

impl Page {
  pub(crate) fn id_unchecked(&self) -> &Url {
    &self.id
  }
  pub(crate) fn id(&self, expected_domain: &Url) -> Result<&Url, LemmyError> {
    verify_domains_match(&self.id, expected_domain)?;
    Ok(&self.id)
  }

  /// Only mods can change the post's stickied/locked status. So if either of these is changed from
  /// the current value, it is a mod action and needs to be verified as such.
  ///
  /// Both stickied and locked need to be false on a newly created post (verified in [[CreatePost]].
  pub(crate) async fn is_mod_action(&self, pool: &DbPool) -> Result<bool, LemmyError> {
    let post_id = self.id.clone();
    let old_post = blocking(pool, move |conn| {
      Post::read_from_apub_id(conn, &post_id.into())
    })
    .await?;

    let is_mod_action = if let Ok(old_post) = old_post {
      self.stickied != Some(old_post.stickied) || self.comments_enabled != Some(!old_post.locked)
    } else {
      false
    };
    Ok(is_mod_action)
  }

  pub(crate) async fn verify(
    &self,
    context: &LemmyContext,
    request_counter: &mut i32,
  ) -> Result<(), LemmyError> {
    let community = extract_community(&self.to, context, request_counter).await?;

    check_slurs(&self.name)?;
    verify_domains_match(&self.attributed_to, &self.id)?;
    verify_person_in_community(
      &self.attributed_to,
      &community.actor_id(),
      context,
      request_counter,
    )
    .await?;
    Ok(())
  }
}

#[async_trait::async_trait(?Send)]
impl ToApub for Post {
  type ApubType = Page;

  // Turn a Lemmy post into an ActivityPub page that can be sent out over the network.
  async fn to_apub(&self, pool: &DbPool) -> Result<Page, LemmyError> {
    let creator_id = self.creator_id;
    let creator = blocking(pool, move |conn| Person::read(conn, creator_id)).await??;
    let community_id = self.community_id;
    let community = blocking(pool, move |conn| Community::read(conn, community_id)).await??;

    let source = self.body.clone().map(|body| Source {
      content: body,
      media_type: MediaTypeMarkdown::Markdown,
    });
    let image = self.thumbnail_url.clone().map(|thumb| ImageObject {
      kind: ImageType::Image,
      url: thumb.into(),
    });

    let page = Page {
      context: lemmy_context(),
      r#type: PageType::Page,
      id: self.ap_id.clone().into(),
      attributed_to: creator.actor_id.into(),
      to: [community.actor_id.into(), public()],
      name: self.name.clone(),
      content: self.body.as_ref().map(|b| markdown_to_html(b)),
      media_type: MediaTypeHtml::Html,
      source,
      url: self.url.clone().map(|u| u.into()),
      image,
      comments_enabled: Some(!self.locked),
      sensitive: Some(self.nsfw),
      stickied: Some(self.stickied),
      published: convert_datetime(self.published),
      updated: self.updated.map(convert_datetime),
      unparsed: Default::default(),
    };
    Ok(page)
  }

  fn to_tombstone(&self) -> Result<Tombstone, LemmyError> {
    create_tombstone(
      self.deleted,
      self.ap_id.to_owned().into(),
      self.updated,
      PageType::Page,
    )
  }
}

#[async_trait::async_trait(?Send)]
impl FromApub for Post {
  type ApubType = Page;

  async fn from_apub(
    page: &Page,
    context: &LemmyContext,
    expected_domain: &Url,
    request_counter: &mut i32,
  ) -> Result<Post, LemmyError> {
    // We can't verify the domain in case of mod action, because the mod may be on a different
    // instance from the post author.
    let ap_id = if page.is_mod_action(context.pool()).await? {
      page.id_unchecked()
    } else {
      page.id(expected_domain)?
    };
    let ap_id = Some(ap_id.clone().into());
    let creator =
      get_or_fetch_and_upsert_person(&page.attributed_to, context, request_counter).await?;
    let community = extract_community(&page.to, context, request_counter).await?;

    let thumbnail_url: Option<Url> = page.image.clone().map(|i| i.url);
    let (metadata_res, pictrs_thumbnail) = if let Some(url) = &page.url {
      fetch_site_data(context.client(), Some(url)).await
    } else {
      (None, thumbnail_url)
    };
    let (embed_title, embed_description, embed_html) = metadata_res
      .map(|u| (u.title, u.description, u.html))
      .unwrap_or((None, None, None));

    let body_slurs_removed = page.source.as_ref().map(|s| remove_slurs(&s.content));
    let form = PostForm {
      name: page.name.clone(),
      url: page.url.clone().map(|u| u.into()),
      body: body_slurs_removed,
      creator_id: creator.id,
      community_id: community.id,
      removed: None,
      locked: page.comments_enabled.map(|e| !e),
      published: Some(page.published.naive_local()),
      updated: page.updated.map(|u| u.naive_local()),
      deleted: None,
      nsfw: page.sensitive,
      stickied: page.stickied,
      embed_title,
      embed_description,
      embed_html,
      thumbnail_url: pictrs_thumbnail.map(|u| u.into()),
      ap_id,
      local: Some(false),
    };
    Ok(blocking(context.pool(), move |conn| Post::upsert(conn, &form)).await??)
  }
}
