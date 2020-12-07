use crate::{
  extensions::{context::lemmy_context, page_extension::PageExtension},
  fetcher::{get_or_fetch_and_upsert_community, get_or_fetch_and_upsert_user},
  objects::{
    check_object_domain,
    check_object_for_community_or_site_ban,
    create_tombstone,
    get_object_from_apub,
    get_source_markdown_value,
    set_content_and_source,
    FromApub,
    FromApubToForm,
    ToApub,
  },
  PageExt,
};
use activitystreams::{
  object::{kind::PageType, ApObject, Image, Page, Tombstone},
  prelude::*,
};
use activitystreams_ext::Ext1;
use anyhow::Context;
use lemmy_db::{
  community::Community,
  post::{Post, PostForm},
  user::User_,
  Crud,
  DbPool,
};
use lemmy_structs::blocking;
use lemmy_utils::{
  location_info,
  request::fetch_iframely_and_pictrs_data,
  utils::{check_slurs, convert_datetime, remove_slurs},
  LemmyError,
};
use lemmy_websocket::LemmyContext;
use url::Url;

#[async_trait::async_trait(?Send)]
impl ToApub for Post {
  type ApubType = PageExt;

  // Turn a Lemmy post into an ActivityPub page that can be sent out over the network.
  async fn to_apub(&self, pool: &DbPool) -> Result<PageExt, LemmyError> {
    let mut page = ApObject::new(Page::new());

    let creator_id = self.creator_id;
    let creator = blocking(pool, move |conn| User_::read(conn, creator_id)).await??;

    let community_id = self.community_id;
    let community = blocking(pool, move |conn| Community::read(conn, community_id)).await??;

    page
      // Not needed when the Post is embedded in a collection (like for community outbox)
      // TODO: need to set proper context defining sensitive/commentsEnabled fields
      // https://git.asonix.dog/Aardwolf/activitystreams/issues/5
      .set_many_contexts(lemmy_context()?)
      .set_id(self.ap_id.parse::<Url>()?)
      // Use summary field to be consistent with mastodon content warning.
      // https://mastodon.xyz/@Louisa/103987265222901387.json
      .set_summary(self.name.to_owned())
      .set_published(convert_datetime(self.published))
      .set_to(community.actor_id)
      .set_attributed_to(creator.actor_id);

    if let Some(body) = &self.body {
      set_content_and_source(&mut page, &body)?;
    }

    // TODO: hacky code because we get self.url == Some("")
    // https://github.com/LemmyNet/lemmy/issues/602
    let url = self.url.as_ref().filter(|u| !u.is_empty());
    if let Some(u) = url {
      page.set_url(Url::parse(u)?);
    }

    if let Some(thumbnail_url) = &self.thumbnail_url {
      let mut image = Image::new();
      image.set_url(Url::parse(thumbnail_url)?);
      page.set_image(image.into_any_base()?);
    }

    if let Some(u) = self.updated {
      page.set_updated(convert_datetime(u));
    }

    let ext = PageExtension {
      comments_enabled: !self.locked,
      sensitive: self.nsfw,
      stickied: self.stickied,
    };
    Ok(Ext1::new(page, ext))
  }

  fn to_tombstone(&self) -> Result<Tombstone, LemmyError> {
    create_tombstone(self.deleted, &self.ap_id, self.updated, PageType::Page)
  }
}

#[async_trait::async_trait(?Send)]
impl FromApub for Post {
  type ApubType = PageExt;

  /// Converts a `PageExt` to `PostForm`.
  ///
  /// If the post's community or creator are not known locally, these are also fetched.
  async fn from_apub(
    page: &PageExt,
    context: &LemmyContext,
    expected_domain: Url,
    request_counter: &mut i32,
  ) -> Result<Post, LemmyError> {
    check_object_for_community_or_site_ban(page, context, request_counter).await?;
    get_object_from_apub(page, context, expected_domain, request_counter).await
  }
}

#[async_trait::async_trait(?Send)]
impl FromApubToForm<PageExt> for PostForm {
  async fn from_apub(
    page: &PageExt,
    context: &LemmyContext,
    expected_domain: Url,
    request_counter: &mut i32,
  ) -> Result<PostForm, LemmyError> {
    let ext = &page.ext_one;
    let creator_actor_id = page
      .inner
      .attributed_to()
      .as_ref()
      .context(location_info!())?
      .as_single_xsd_any_uri()
      .context(location_info!())?;

    let creator = get_or_fetch_and_upsert_user(creator_actor_id, context, request_counter).await?;

    let community_actor_id = page
      .inner
      .to()
      .as_ref()
      .context(location_info!())?
      .as_single_xsd_any_uri()
      .context(location_info!())?;

    let community =
      get_or_fetch_and_upsert_community(community_actor_id, context, request_counter).await?;

    let thumbnail_url = match &page.inner.image() {
      Some(any_image) => Image::from_any_base(
        any_image
          .to_owned()
          .as_one()
          .context(location_info!())?
          .to_owned(),
      )?
      .context(location_info!())?
      .url()
      .context(location_info!())?
      .as_single_xsd_any_uri()
      .map(|u| u.to_string()),
      None => None,
    };
    let url = page
      .inner
      .url()
      .map(|u| u.as_single_xsd_any_uri())
      .flatten()
      .map(|s| s.to_string());

    let (iframely_title, iframely_description, iframely_html, pictrs_thumbnail) =
      if let Some(url) = &url {
        fetch_iframely_and_pictrs_data(context.client(), Some(url.to_owned())).await
      } else {
        (None, None, None, thumbnail_url)
      };

    let name = page
      .inner
      .summary()
      .as_ref()
      .context(location_info!())?
      .as_single_xsd_string()
      .context(location_info!())?
      .to_string();
    let body = get_source_markdown_value(page)?;

    check_slurs(&name)?;
    let body_slurs_removed = body.map(|b| remove_slurs(&b));
    Ok(PostForm {
      name,
      url,
      body: body_slurs_removed,
      creator_id: creator.id,
      community_id: community.id,
      removed: None,
      locked: Some(!ext.comments_enabled),
      published: page
        .inner
        .published()
        .as_ref()
        .map(|u| u.to_owned().naive_local()),
      updated: page
        .inner
        .updated()
        .as_ref()
        .map(|u| u.to_owned().naive_local()),
      deleted: None,
      nsfw: ext.sensitive,
      stickied: Some(ext.stickied),
      embed_title: iframely_title,
      embed_description: iframely_description,
      embed_html: iframely_html,
      thumbnail_url: pictrs_thumbnail,
      ap_id: Some(check_object_domain(page, expected_domain)?),
      local: false,
    })
  }
}
