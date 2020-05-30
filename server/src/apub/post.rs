use crate::{
  apub::{
    activities::{populate_object_props, send_activity},
    create_apub_response,
    create_apub_tombstone_response,
    create_tombstone,
    extensions::page_extension::PageExtension,
    fetcher::{get_or_fetch_and_upsert_remote_community, get_or_fetch_and_upsert_remote_user},
    get_apub_protocol_string,
    ActorType,
    ApubLikeableType,
    ApubObjectType,
    FromApub,
    PageExt,
    ToApub,
  },
  convert_datetime,
  db::{
    activity::insert_activity,
    community::Community,
    post::{Post, PostForm},
    user::User_,
    Crud,
  },
  routes::DbPoolParam,
  Settings,
};
use activitystreams::{
  activity::{Create, Delete, Dislike, Like, Remove, Undo, Update},
  context,
  object::{kind::PageType, properties::ObjectProperties, AnyImage, Image, Page, Tombstone},
  Activity,
  Base,
  BaseBox,
};
use activitystreams_ext::Ext1;
use actix_web::{body::Body, web::Path, HttpResponse, Result};
use diesel::PgConnection;
use failure::{Error, _core::fmt::Debug};
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct PostQuery {
  post_id: String,
}

/// Return the post json over HTTP.
pub async fn get_apub_post(
  info: Path<PostQuery>,
  db: DbPoolParam,
) -> Result<HttpResponse<Body>, Error> {
  let id = info.post_id.parse::<i32>()?;
  let post = Post::read(&&db.get()?, id)?;
  if !post.deleted {
    Ok(create_apub_response(&post.to_apub(&db.get().unwrap())?))
  } else {
    Ok(create_apub_tombstone_response(&post.to_tombstone()?))
  }
}

impl ToApub for Post {
  type Response = PageExt;

  // Turn a Lemmy post into an ActivityPub page that can be sent out over the network.
  fn to_apub(&self, conn: &PgConnection) -> Result<PageExt, Error> {
    let mut page = Page::default();
    let oprops: &mut ObjectProperties = page.as_mut();
    let creator = User_::read(conn, self.creator_id)?;
    let community = Community::read(conn, self.community_id)?;

    oprops
      // Not needed when the Post is embedded in a collection (like for community outbox)
      // TODO: need to set proper context defining sensitive/commentsEnabled fields
      // https://git.asonix.dog/Aardwolf/activitystreams/issues/5
      .set_context_xsd_any_uri(context())?
      .set_id(self.ap_id.to_owned())?
      // Use summary field to be consistent with mastodon content warning.
      // https://mastodon.xyz/@Louisa/103987265222901387.json
      .set_summary_xsd_string(self.name.to_owned())?
      .set_published(convert_datetime(self.published))?
      .set_to_xsd_any_uri(community.actor_id)?
      .set_attributed_to_xsd_any_uri(creator.actor_id)?;

    if let Some(body) = &self.body {
      oprops.set_content_xsd_string(body.to_owned())?;
    }

    // TODO: hacky code because we get self.url == Some("")
    // https://github.com/LemmyNet/lemmy/issues/602
    let url = self.url.as_ref().filter(|u| !u.is_empty());
    if let Some(u) = url {
      oprops.set_url_xsd_any_uri(u.to_owned())?;

      // Embeds
      let mut page_preview = Page::new();
      page_preview
        .object_props
        .set_url_xsd_any_uri(u.to_owned())?;

      if let Some(embed_title) = &self.embed_title {
        page_preview
          .object_props
          .set_name_xsd_string(embed_title.to_owned())?;
      }

      if let Some(embed_description) = &self.embed_description {
        page_preview
          .object_props
          .set_summary_xsd_string(embed_description.to_owned())?;
      }

      if let Some(embed_html) = &self.embed_html {
        page_preview
          .object_props
          .set_content_xsd_string(embed_html.to_owned())?;
      }

      oprops.set_preview_base_box(page_preview)?;
    }

    if let Some(thumbnail_url) = &self.thumbnail_url {
      let full_url = format!(
        "{}://{}/pictshare/{}",
        get_apub_protocol_string(),
        Settings::get().hostname,
        thumbnail_url
      );

      let mut image = Image::new();
      image.object_props.set_url_xsd_any_uri(full_url)?;
      let any_image = AnyImage::from_concrete(image)?;
      oprops.set_image_any_image(any_image)?;
    }

    if let Some(u) = self.updated {
      oprops.set_updated(convert_datetime(u))?;
    }

    let ext = PageExtension {
      comments_enabled: !self.locked,
      sensitive: self.nsfw,
    };
    Ok(Ext1::new(page, ext))
  }

  fn to_tombstone(&self) -> Result<Tombstone, Error> {
    create_tombstone(
      self.deleted,
      &self.ap_id,
      self.updated,
      PageType.to_string(),
    )
  }
}

impl FromApub for PostForm {
  type ApubType = PageExt;

  /// Parse an ActivityPub page received from another instance into a Lemmy post.
  fn from_apub(page: &PageExt, conn: &PgConnection) -> Result<PostForm, Error> {
    let ext = &page.ext_one;
    let oprops = &page.inner.object_props;
    let creator_actor_id = &oprops.get_attributed_to_xsd_any_uri().unwrap().to_string();
    let creator = get_or_fetch_and_upsert_remote_user(&creator_actor_id, &conn)?;
    let community_actor_id = &oprops.get_to_xsd_any_uri().unwrap().to_string();
    let community = get_or_fetch_and_upsert_remote_community(&community_actor_id, &conn)?;

    let thumbnail_url = match oprops.get_image_any_image() {
      Some(any_image) => any_image
        .to_owned()
        .into_concrete::<Image>()?
        .object_props
        .get_url_xsd_any_uri()
        .map(|u| u.to_string()),
      None => None,
    };

    let url = oprops.get_url_xsd_any_uri().map(|u| u.to_string());
    let (embed_title, embed_description, embed_html) = match oprops.get_preview_base_box() {
      Some(preview) => {
        let preview_page = preview.to_owned().into_concrete::<Page>()?;
        let name = preview_page
          .object_props
          .get_name_xsd_string()
          .map(|n| n.to_string());
        let summary = preview_page
          .object_props
          .get_summary_xsd_string()
          .map(|s| s.to_string());
        let content = preview_page
          .object_props
          .get_content_xsd_string()
          .map(|c| c.to_string());
        (name, summary, content)
      }
      None => (None, None, None),
    };

    Ok(PostForm {
      name: oprops.get_summary_xsd_string().unwrap().to_string(),
      url,
      body: oprops.get_content_xsd_string().map(|c| c.to_string()),
      creator_id: creator.id,
      community_id: community.id,
      removed: None,
      locked: Some(!ext.comments_enabled),
      published: oprops
        .get_published()
        .map(|u| u.as_ref().to_owned().naive_local()),
      updated: oprops
        .get_updated()
        .map(|u| u.as_ref().to_owned().naive_local()),
      deleted: None,
      nsfw: ext.sensitive,
      stickied: None, // -> put it in "featured" collection of the community
      embed_title,
      embed_description,
      embed_html,
      thumbnail_url,
      ap_id: oprops.get_id().unwrap().to_string(),
      local: false,
    })
  }
}

impl ApubObjectType for Post {
  /// Send out information about a newly created post, to the followers of the community.
  fn send_create(&self, creator: &User_, conn: &PgConnection) -> Result<(), Error> {
    let page = self.to_apub(conn)?;
    let community = Community::read(conn, self.community_id)?;
    let id = format!("{}/create/{}", self.ap_id, uuid::Uuid::new_v4());

    let mut create = Create::new();
    populate_object_props(
      &mut create.object_props,
      vec![community.get_followers_url()],
      &id,
    )?;
    create
      .create_props
      .set_actor_xsd_any_uri(creator.actor_id.to_owned())?
      .set_object_base_box(BaseBox::from_concrete(page)?)?;

    Post::send_post_activity(creator, conn, &community, create)?;
    Ok(())
  }

  /// Send out information about an edited post, to the followers of the community.
  fn send_update(&self, creator: &User_, conn: &PgConnection) -> Result<(), Error> {
    let page = self.to_apub(conn)?;
    let community = Community::read(conn, self.community_id)?;
    let id = format!("{}/update/{}", self.ap_id, uuid::Uuid::new_v4());

    let mut update = Update::new();
    populate_object_props(
      &mut update.object_props,
      vec![community.get_followers_url()],
      &id,
    )?;
    update
      .update_props
      .set_actor_xsd_any_uri(creator.actor_id.to_owned())?
      .set_object_base_box(BaseBox::from_concrete(page)?)?;

    Post::send_post_activity(creator, conn, &community, update)?;
    Ok(())
  }

  fn send_delete(&self, creator: &User_, conn: &PgConnection) -> Result<(), Error> {
    let page = self.to_apub(conn)?;
    let community = Community::read(conn, self.community_id)?;
    let id = format!("{}/delete/{}", self.ap_id, uuid::Uuid::new_v4());
    let mut delete = Delete::default();

    populate_object_props(
      &mut delete.object_props,
      vec![community.get_followers_url()],
      &id,
    )?;

    delete
      .delete_props
      .set_actor_xsd_any_uri(creator.actor_id.to_owned())?
      .set_object_base_box(BaseBox::from_concrete(page)?)?;

    let community = Community::read(conn, self.community_id)?;

    Post::send_post_activity(creator, conn, &community, delete)?;
    Ok(())
  }

  fn send_undo_delete(&self, creator: &User_, conn: &PgConnection) -> Result<(), Error> {
    let page = self.to_apub(conn)?;
    let community = Community::read(conn, self.community_id)?;
    let id = format!("{}/delete/{}", self.ap_id, uuid::Uuid::new_v4());
    let mut delete = Delete::default();

    populate_object_props(
      &mut delete.object_props,
      vec![community.get_followers_url()],
      &id,
    )?;

    delete
      .delete_props
      .set_actor_xsd_any_uri(creator.actor_id.to_owned())?
      .set_object_base_box(BaseBox::from_concrete(page)?)?;

    // TODO
    // Undo that fake activity
    let undo_id = format!("{}/undo/delete/{}", self.ap_id, uuid::Uuid::new_v4());
    let mut undo = Undo::default();

    populate_object_props(
      &mut undo.object_props,
      vec![community.get_followers_url()],
      &undo_id,
    )?;

    undo
      .undo_props
      .set_actor_xsd_any_uri(creator.actor_id.to_owned())?
      .set_object_base_box(delete)?;

    let community = Community::read(conn, self.community_id)?;
    Post::send_post_activity(creator, conn, &community, undo)?;
    Ok(())
  }

  fn send_remove(&self, mod_: &User_, conn: &PgConnection) -> Result<(), Error> {
    let page = self.to_apub(conn)?;
    let community = Community::read(conn, self.community_id)?;
    let id = format!("{}/remove/{}", self.ap_id, uuid::Uuid::new_v4());
    let mut remove = Remove::default();

    populate_object_props(
      &mut remove.object_props,
      vec![community.get_followers_url()],
      &id,
    )?;

    remove
      .remove_props
      .set_actor_xsd_any_uri(mod_.actor_id.to_owned())?
      .set_object_base_box(BaseBox::from_concrete(page)?)?;

    let community = Community::read(conn, self.community_id)?;

    Post::send_post_activity(mod_, conn, &community, remove)?;
    Ok(())
  }
  fn send_undo_remove(&self, mod_: &User_, conn: &PgConnection) -> Result<(), Error> {
    let page = self.to_apub(conn)?;
    let community = Community::read(conn, self.community_id)?;
    let id = format!("{}/remove/{}", self.ap_id, uuid::Uuid::new_v4());
    let mut remove = Remove::default();

    populate_object_props(
      &mut remove.object_props,
      vec![community.get_followers_url()],
      &id,
    )?;

    remove
      .remove_props
      .set_actor_xsd_any_uri(mod_.actor_id.to_owned())?
      .set_object_base_box(BaseBox::from_concrete(page)?)?;

    // Undo that fake activity
    let undo_id = format!("{}/undo/remove/{}", self.ap_id, uuid::Uuid::new_v4());
    let mut undo = Undo::default();

    populate_object_props(
      &mut undo.object_props,
      vec![community.get_followers_url()],
      &undo_id,
    )?;

    undo
      .undo_props
      .set_actor_xsd_any_uri(mod_.actor_id.to_owned())?
      .set_object_base_box(remove)?;

    let community = Community::read(conn, self.community_id)?;
    Post::send_post_activity(mod_, conn, &community, undo)?;
    Ok(())
  }
}

impl ApubLikeableType for Post {
  fn send_like(&self, creator: &User_, conn: &PgConnection) -> Result<(), Error> {
    let page = self.to_apub(conn)?;
    let community = Community::read(conn, self.community_id)?;
    let id = format!("{}/like/{}", self.ap_id, uuid::Uuid::new_v4());

    let mut like = Like::new();
    populate_object_props(
      &mut like.object_props,
      vec![community.get_followers_url()],
      &id,
    )?;
    like
      .like_props
      .set_actor_xsd_any_uri(creator.actor_id.to_owned())?
      .set_object_base_box(BaseBox::from_concrete(page)?)?;

    Post::send_post_activity(&creator, &conn, &community, like)?;
    Ok(())
  }

  fn send_dislike(&self, creator: &User_, conn: &PgConnection) -> Result<(), Error> {
    let page = self.to_apub(conn)?;
    let community = Community::read(conn, self.community_id)?;
    let id = format!("{}/dislike/{}", self.ap_id, uuid::Uuid::new_v4());

    let mut dislike = Dislike::new();
    populate_object_props(
      &mut dislike.object_props,
      vec![community.get_followers_url()],
      &id,
    )?;
    dislike
      .dislike_props
      .set_actor_xsd_any_uri(creator.actor_id.to_owned())?
      .set_object_base_box(BaseBox::from_concrete(page)?)?;

    Post::send_post_activity(&creator, &conn, &community, dislike)?;
    Ok(())
  }

  fn send_undo_like(&self, creator: &User_, conn: &PgConnection) -> Result<(), Error> {
    let page = self.to_apub(conn)?;
    let community = Community::read(conn, self.community_id)?;
    let id = format!("{}/like/{}", self.ap_id, uuid::Uuid::new_v4());

    let mut like = Like::new();
    populate_object_props(
      &mut like.object_props,
      vec![community.get_followers_url()],
      &id,
    )?;
    like
      .like_props
      .set_actor_xsd_any_uri(creator.actor_id.to_owned())?
      .set_object_base_box(BaseBox::from_concrete(page)?)?;

    // TODO
    // Undo that fake activity
    let undo_id = format!("{}/undo/like/{}", self.ap_id, uuid::Uuid::new_v4());
    let mut undo = Undo::default();

    populate_object_props(
      &mut undo.object_props,
      vec![community.get_followers_url()],
      &undo_id,
    )?;

    undo
      .undo_props
      .set_actor_xsd_any_uri(creator.actor_id.to_owned())?
      .set_object_base_box(like)?;

    Post::send_post_activity(&creator, &conn, &community, undo)?;
    Ok(())
  }
}

impl Post {
  fn send_post_activity<A>(
    creator: &User_,
    conn: &PgConnection,
    community: &Community,
    activity: A,
  ) -> Result<(), Error>
  where
    A: Activity + Base + Serialize + Debug,
  {
    insert_activity(&conn, creator.id, &activity, true)?;

    // if this is a local community, we need to do an announce from the community instead
    if community.local {
      Community::do_announce(activity, &community, &creator.actor_id, conn, true)?;
    } else {
      send_activity(&activity, creator, vec![community.get_shared_inbox_url()])?;
    }
    Ok(())
  }
}
