use crate::{
  apub::{
    activities::{generate_activity_id, send_activity_to_community},
    check_actor_domain,
    create_apub_response,
    create_apub_tombstone_response,
    create_tombstone,
    extensions::page_extension::PageExtension,
    fetcher::{get_or_fetch_and_upsert_community, get_or_fetch_and_upsert_user},
    ActorType,
    ApubLikeableType,
    ApubObjectType,
    FromApub,
    PageExt,
    ToApub,
  },
  DbPool,
  LemmyContext,
};
use activitystreams::{
  activity::{
    kind::{CreateType, DeleteType, DislikeType, LikeType, RemoveType, UndoType, UpdateType},
    Create,
    Delete,
    Dislike,
    Like,
    Remove,
    Undo,
    Update,
  },
  object::{kind::PageType, Image, Object, Page, Tombstone},
  prelude::*,
  public,
};
use activitystreams_ext::Ext1;
use actix_web::{body::Body, web, HttpResponse};
use anyhow::Context;
use lemmy_api_structs::blocking;
use lemmy_db::{
  community::Community,
  post::{Post, PostForm},
  user::User_,
  Crud,
};
use lemmy_utils::{
  location_info,
  utils::{check_slurs, convert_datetime, remove_slurs},
  LemmyError,
};
use serde::Deserialize;
use url::Url;

#[derive(Deserialize)]
pub struct PostQuery {
  post_id: String,
}

/// Return the post json over HTTP.
pub async fn get_apub_post(
  info: web::Path<PostQuery>,
  context: web::Data<LemmyContext>,
) -> Result<HttpResponse<Body>, LemmyError> {
  let id = info.post_id.parse::<i32>()?;
  let post = blocking(context.pool(), move |conn| Post::read(conn, id)).await??;

  if !post.deleted {
    Ok(create_apub_response(&post.to_apub(context.pool()).await?))
  } else {
    Ok(create_apub_tombstone_response(&post.to_tombstone()?))
  }
}

#[async_trait::async_trait(?Send)]
impl ToApub for Post {
  type Response = PageExt;

  // Turn a Lemmy post into an ActivityPub page that can be sent out over the network.
  async fn to_apub(&self, pool: &DbPool) -> Result<PageExt, LemmyError> {
    let mut page = Page::new();

    let creator_id = self.creator_id;
    let creator = blocking(pool, move |conn| User_::read(conn, creator_id)).await??;

    let community_id = self.community_id;
    let community = blocking(pool, move |conn| Community::read(conn, community_id)).await??;

    page
      // Not needed when the Post is embedded in a collection (like for community outbox)
      // TODO: need to set proper context defining sensitive/commentsEnabled fields
      // https://git.asonix.dog/Aardwolf/activitystreams/issues/5
      .set_context(activitystreams::context())
      .set_id(self.ap_id.parse::<Url>()?)
      // Use summary field to be consistent with mastodon content warning.
      // https://mastodon.xyz/@Louisa/103987265222901387.json
      .set_summary(self.name.to_owned())
      .set_published(convert_datetime(self.published))
      .set_to(community.actor_id)
      .set_attributed_to(creator.actor_id);

    if let Some(body) = &self.body {
      page.set_content(body.to_owned());
    }

    // TODO: hacky code because we get self.url == Some("")
    // https://github.com/LemmyNet/lemmy/issues/602
    let url = self.url.as_ref().filter(|u| !u.is_empty());
    if let Some(u) = url {
      page.set_url(u.to_owned());

      // Embeds
      let mut page_preview = Page::new();
      page_preview.set_url(u.to_owned());

      if let Some(embed_title) = &self.embed_title {
        page_preview.set_name(embed_title.to_owned());
      }

      if let Some(embed_description) = &self.embed_description {
        page_preview.set_summary(embed_description.to_owned());
      }

      if let Some(embed_html) = &self.embed_html {
        page_preview.set_content(embed_html.to_owned());
      }

      page.set_preview(page_preview.into_any_base()?);
    }

    if let Some(thumbnail_url) = &self.thumbnail_url {
      let mut image = Image::new();
      image.set_url(thumbnail_url.to_string());
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

struct EmbedType {
  title: Option<String>,
  description: Option<String>,
  html: Option<String>,
}

fn extract_embed_from_apub(
  page: &Ext1<Object<PageType>, PageExtension>,
) -> Result<EmbedType, LemmyError> {
  match page.inner.preview() {
    Some(preview) => {
      let preview_page = Page::from_any_base(preview.one().context(location_info!())?.to_owned())?
        .context(location_info!())?;
      let title = preview_page
        .name()
        .map(|n| n.one())
        .flatten()
        .map(|s| s.as_xsd_string())
        .flatten()
        .map(|s| s.to_string());
      let description = preview_page
        .summary()
        .map(|s| s.as_single_xsd_string())
        .flatten()
        .map(|s| s.to_string());
      let html = preview_page
        .content()
        .map(|c| c.as_single_xsd_string())
        .flatten()
        .map(|s| s.to_string());
      Ok(EmbedType {
        title,
        description,
        html,
      })
    }
    None => Ok(EmbedType {
      title: None,
      description: None,
      html: None,
    }),
  }
}

#[async_trait::async_trait(?Send)]
impl FromApub for PostForm {
  type ApubType = PageExt;

  /// Parse an ActivityPub page received from another instance into a Lemmy post.
  async fn from_apub(
    page: &PageExt,
    context: &LemmyContext,
    expected_domain: Option<Url>,
  ) -> Result<PostForm, LemmyError> {
    let ext = &page.ext_one;
    let creator_actor_id = page
      .inner
      .attributed_to()
      .as_ref()
      .context(location_info!())?
      .as_single_xsd_any_uri()
      .context(location_info!())?;

    let creator = get_or_fetch_and_upsert_user(creator_actor_id, context).await?;

    let community_actor_id = page
      .inner
      .to()
      .as_ref()
      .context(location_info!())?
      .as_single_xsd_any_uri()
      .context(location_info!())?;

    let community = get_or_fetch_and_upsert_community(community_actor_id, context).await?;

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

    let embed = extract_embed_from_apub(page)?;

    let name = page
      .inner
      .summary()
      .as_ref()
      .context(location_info!())?
      .as_single_xsd_string()
      .context(location_info!())?
      .to_string();
    let url = page
      .inner
      .url()
      .as_ref()
      .map(|u| u.as_single_xsd_string())
      .flatten()
      .map(|s| s.to_string());
    let body = page
      .inner
      .content()
      .as_ref()
      .map(|c| c.as_single_xsd_string())
      .flatten()
      .map(|s| s.to_string());
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
      embed_title: embed.title,
      embed_description: embed.description,
      embed_html: embed.html,
      thumbnail_url,
      ap_id: Some(check_actor_domain(page, expected_domain)?),
      local: false,
    })
  }
}

#[async_trait::async_trait(?Send)]
impl ApubObjectType for Post {
  /// Send out information about a newly created post, to the followers of the community.
  async fn send_create(&self, creator: &User_, context: &LemmyContext) -> Result<(), LemmyError> {
    let page = self.to_apub(context.pool()).await?;

    let community_id = self.community_id;
    let community = blocking(context.pool(), move |conn| {
      Community::read(conn, community_id)
    })
    .await??;

    let mut create = Create::new(creator.actor_id.to_owned(), page.into_any_base()?);
    create
      .set_context(activitystreams::context())
      .set_id(generate_activity_id(CreateType::Create)?)
      .set_to(public())
      .set_many_ccs(vec![community.get_followers_url()?]);

    send_activity_to_community(
      creator,
      &community,
      vec![community.get_shared_inbox_url()?],
      create,
      context,
    )
    .await?;
    Ok(())
  }

  /// Send out information about an edited post, to the followers of the community.
  async fn send_update(&self, creator: &User_, context: &LemmyContext) -> Result<(), LemmyError> {
    let page = self.to_apub(context.pool()).await?;

    let community_id = self.community_id;
    let community = blocking(context.pool(), move |conn| {
      Community::read(conn, community_id)
    })
    .await??;

    let mut update = Update::new(creator.actor_id.to_owned(), page.into_any_base()?);
    update
      .set_context(activitystreams::context())
      .set_id(generate_activity_id(UpdateType::Update)?)
      .set_to(public())
      .set_many_ccs(vec![community.get_followers_url()?]);

    send_activity_to_community(
      creator,
      &community,
      vec![community.get_shared_inbox_url()?],
      update,
      context,
    )
    .await?;
    Ok(())
  }

  async fn send_delete(&self, creator: &User_, context: &LemmyContext) -> Result<(), LemmyError> {
    let page = self.to_apub(context.pool()).await?;

    let community_id = self.community_id;
    let community = blocking(context.pool(), move |conn| {
      Community::read(conn, community_id)
    })
    .await??;

    let mut delete = Delete::new(creator.actor_id.to_owned(), page.into_any_base()?);
    delete
      .set_context(activitystreams::context())
      .set_id(generate_activity_id(DeleteType::Delete)?)
      .set_to(public())
      .set_many_ccs(vec![community.get_followers_url()?]);

    send_activity_to_community(
      creator,
      &community,
      vec![community.get_shared_inbox_url()?],
      delete,
      context,
    )
    .await?;
    Ok(())
  }

  async fn send_undo_delete(
    &self,
    creator: &User_,
    context: &LemmyContext,
  ) -> Result<(), LemmyError> {
    let page = self.to_apub(context.pool()).await?;

    let community_id = self.community_id;
    let community = blocking(context.pool(), move |conn| {
      Community::read(conn, community_id)
    })
    .await??;

    let mut delete = Delete::new(creator.actor_id.to_owned(), page.into_any_base()?);
    delete
      .set_context(activitystreams::context())
      .set_id(generate_activity_id(DeleteType::Delete)?)
      .set_to(public())
      .set_many_ccs(vec![community.get_followers_url()?]);

    // Undo that fake activity
    let mut undo = Undo::new(creator.actor_id.to_owned(), delete.into_any_base()?);
    undo
      .set_context(activitystreams::context())
      .set_id(generate_activity_id(UndoType::Undo)?)
      .set_to(public())
      .set_many_ccs(vec![community.get_followers_url()?]);

    send_activity_to_community(
      creator,
      &community,
      vec![community.get_shared_inbox_url()?],
      undo,
      context,
    )
    .await?;
    Ok(())
  }

  async fn send_remove(&self, mod_: &User_, context: &LemmyContext) -> Result<(), LemmyError> {
    let page = self.to_apub(context.pool()).await?;

    let community_id = self.community_id;
    let community = blocking(context.pool(), move |conn| {
      Community::read(conn, community_id)
    })
    .await??;

    let mut remove = Remove::new(mod_.actor_id.to_owned(), page.into_any_base()?);
    remove
      .set_context(activitystreams::context())
      .set_id(generate_activity_id(RemoveType::Remove)?)
      .set_to(public())
      .set_many_ccs(vec![community.get_followers_url()?]);

    send_activity_to_community(
      mod_,
      &community,
      vec![community.get_shared_inbox_url()?],
      remove,
      context,
    )
    .await?;
    Ok(())
  }

  async fn send_undo_remove(&self, mod_: &User_, context: &LemmyContext) -> Result<(), LemmyError> {
    let page = self.to_apub(context.pool()).await?;

    let community_id = self.community_id;
    let community = blocking(context.pool(), move |conn| {
      Community::read(conn, community_id)
    })
    .await??;

    let mut remove = Remove::new(mod_.actor_id.to_owned(), page.into_any_base()?);
    remove
      .set_context(activitystreams::context())
      .set_id(generate_activity_id(RemoveType::Remove)?)
      .set_to(public())
      .set_many_ccs(vec![community.get_followers_url()?]);

    // Undo that fake activity
    let mut undo = Undo::new(mod_.actor_id.to_owned(), remove.into_any_base()?);
    undo
      .set_context(activitystreams::context())
      .set_id(generate_activity_id(UndoType::Undo)?)
      .set_to(public())
      .set_many_ccs(vec![community.get_followers_url()?]);

    send_activity_to_community(
      mod_,
      &community,
      vec![community.get_shared_inbox_url()?],
      undo,
      context,
    )
    .await?;
    Ok(())
  }
}

#[async_trait::async_trait(?Send)]
impl ApubLikeableType for Post {
  async fn send_like(&self, creator: &User_, context: &LemmyContext) -> Result<(), LemmyError> {
    let page = self.to_apub(context.pool()).await?;

    let community_id = self.community_id;
    let community = blocking(context.pool(), move |conn| {
      Community::read(conn, community_id)
    })
    .await??;

    let mut like = Like::new(creator.actor_id.to_owned(), page.into_any_base()?);
    like
      .set_context(activitystreams::context())
      .set_id(generate_activity_id(LikeType::Like)?)
      .set_to(public())
      .set_many_ccs(vec![community.get_followers_url()?]);

    send_activity_to_community(
      &creator,
      &community,
      vec![community.get_shared_inbox_url()?],
      like,
      context,
    )
    .await?;
    Ok(())
  }

  async fn send_dislike(&self, creator: &User_, context: &LemmyContext) -> Result<(), LemmyError> {
    let page = self.to_apub(context.pool()).await?;

    let community_id = self.community_id;
    let community = blocking(context.pool(), move |conn| {
      Community::read(conn, community_id)
    })
    .await??;

    let mut dislike = Dislike::new(creator.actor_id.to_owned(), page.into_any_base()?);
    dislike
      .set_context(activitystreams::context())
      .set_id(generate_activity_id(DislikeType::Dislike)?)
      .set_to(public())
      .set_many_ccs(vec![community.get_followers_url()?]);

    send_activity_to_community(
      &creator,
      &community,
      vec![community.get_shared_inbox_url()?],
      dislike,
      context,
    )
    .await?;
    Ok(())
  }

  async fn send_undo_like(
    &self,
    creator: &User_,
    context: &LemmyContext,
  ) -> Result<(), LemmyError> {
    let page = self.to_apub(context.pool()).await?;

    let community_id = self.community_id;
    let community = blocking(context.pool(), move |conn| {
      Community::read(conn, community_id)
    })
    .await??;

    let mut like = Like::new(creator.actor_id.to_owned(), page.into_any_base()?);
    like
      .set_context(activitystreams::context())
      .set_id(generate_activity_id(LikeType::Like)?)
      .set_to(public())
      .set_many_ccs(vec![community.get_followers_url()?]);

    // Undo that fake activity
    let mut undo = Undo::new(creator.actor_id.to_owned(), like.into_any_base()?);
    undo
      .set_context(activitystreams::context())
      .set_id(generate_activity_id(UndoType::Undo)?)
      .set_to(public())
      .set_many_ccs(vec![community.get_followers_url()?]);

    send_activity_to_community(
      &creator,
      &community,
      vec![community.get_shared_inbox_url()?],
      undo,
      context,
    )
    .await?;
    Ok(())
  }
}
