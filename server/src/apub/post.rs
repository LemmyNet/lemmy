use super::*;

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
    }

    if let Some(u) = self.updated {
      oprops.set_updated(convert_datetime(u))?;
    }

    let ext = PageExtension {
      comments_enabled: !self.locked,
      sensitive: self.nsfw,
    };
    Ok(page.extend(ext))
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
    let ext = &page.extension;
    let page = &page.base;
    let oprops = &page.object_props;
    let creator_actor_id = &oprops.get_attributed_to_xsd_any_uri().unwrap().to_string();
    let creator = get_or_fetch_and_upsert_remote_user(&creator_actor_id, &conn)?;
    let community_actor_id = &oprops.get_to_xsd_any_uri().unwrap().to_string();
    let community = get_or_fetch_and_upsert_remote_community(&community_actor_id, &conn)?;

    Ok(PostForm {
      name: oprops.get_summary_xsd_string().unwrap().to_string(),
      url: oprops.get_url_xsd_any_uri().map(|u| u.to_string()),
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
      stickied: None,    // -> put it in "featured" collection of the community
      embed_title: None, // -> attachment? or fetch the embed locally
      embed_description: None,
      embed_html: None,
      thumbnail_url: None,
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
      &community.get_followers_url(),
      &id,
    )?;
    create
      .create_props
      .set_actor_xsd_any_uri(creator.actor_id.to_owned())?
      .set_object_base_box(page)?;

    // Insert the sent activity into the activity table
    let activity_form = activity::ActivityForm {
      user_id: creator.id,
      data: serde_json::to_value(&create)?,
      local: true,
      updated: None,
    };
    activity::Activity::create(&conn, &activity_form)?;

    send_activity(
      &create,
      &creator.private_key.as_ref().unwrap(),
      &creator.actor_id,
      community.get_follower_inboxes(&conn)?,
    )?;
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
      &community.get_followers_url(),
      &id,
    )?;
    update
      .update_props
      .set_actor_xsd_any_uri(creator.actor_id.to_owned())?
      .set_object_base_box(page)?;

    // Insert the sent activity into the activity table
    let activity_form = activity::ActivityForm {
      user_id: creator.id,
      data: serde_json::to_value(&update)?,
      local: true,
      updated: None,
    };
    activity::Activity::create(&conn, &activity_form)?;

    send_activity(
      &update,
      &creator.private_key.as_ref().unwrap(),
      &creator.actor_id,
      community.get_follower_inboxes(&conn)?,
    )?;
    Ok(())
  }

  fn send_delete(&self, creator: &User_, conn: &PgConnection) -> Result<(), Error> {
    let page = self.to_apub(conn)?;
    let community = Community::read(conn, self.community_id)?;
    let id = format!("{}/delete/{}", self.ap_id, uuid::Uuid::new_v4());
    let mut delete = Delete::default();

    populate_object_props(
      &mut delete.object_props,
      &community.get_followers_url(),
      &id,
    )?;

    delete
      .delete_props
      .set_actor_xsd_any_uri(creator.actor_id.to_owned())?
      .set_object_base_box(page)?;

    // Insert the sent activity into the activity table
    let activity_form = activity::ActivityForm {
      user_id: self.creator_id,
      data: serde_json::to_value(&delete)?,
      local: true,
      updated: None,
    };
    activity::Activity::create(&conn, &activity_form)?;

    let community = Community::read(conn, self.community_id)?;
    send_activity(
      &delete,
      &creator.private_key.as_ref().unwrap(),
      &creator.actor_id,
      community.get_follower_inboxes(&conn)?,
    )?;
    Ok(())
  }

  fn send_undo_delete(&self, creator: &User_, conn: &PgConnection) -> Result<(), Error> {
    let page = self.to_apub(conn)?;
    let community = Community::read(conn, self.community_id)?;
    let id = format!("{}/delete/{}", self.ap_id, uuid::Uuid::new_v4());
    let mut delete = Delete::default();

    populate_object_props(
      &mut delete.object_props,
      &community.get_followers_url(),
      &id,
    )?;

    delete
      .delete_props
      .set_actor_xsd_any_uri(creator.actor_id.to_owned())?
      .set_object_base_box(page)?;

    // TODO
    // Undo that fake activity
    let undo_id = format!("{}/undo/delete/{}", self.ap_id, uuid::Uuid::new_v4());
    let mut undo = Undo::default();

    populate_object_props(
      &mut undo.object_props,
      &community.get_followers_url(),
      &undo_id,
    )?;

    undo
      .undo_props
      .set_actor_xsd_any_uri(creator.actor_id.to_owned())?
      .set_object_base_box(delete)?;

    // Insert the sent activity into the activity table
    let activity_form = activity::ActivityForm {
      user_id: self.creator_id,
      data: serde_json::to_value(&undo)?,
      local: true,
      updated: None,
    };
    activity::Activity::create(&conn, &activity_form)?;

    let community = Community::read(conn, self.community_id)?;
    send_activity(
      &undo,
      &creator.private_key.as_ref().unwrap(),
      &creator.actor_id,
      community.get_follower_inboxes(&conn)?,
    )?;
    Ok(())
  }

  fn send_remove(&self, mod_: &User_, conn: &PgConnection) -> Result<(), Error> {
    let page = self.to_apub(conn)?;
    let community = Community::read(conn, self.community_id)?;
    let id = format!("{}/remove/{}", self.ap_id, uuid::Uuid::new_v4());
    let mut remove = Remove::default();

    populate_object_props(
      &mut remove.object_props,
      &community.get_followers_url(),
      &id,
    )?;

    remove
      .remove_props
      .set_actor_xsd_any_uri(mod_.actor_id.to_owned())?
      .set_object_base_box(page)?;

    // Insert the sent activity into the activity table
    let activity_form = activity::ActivityForm {
      user_id: mod_.id,
      data: serde_json::to_value(&remove)?,
      local: true,
      updated: None,
    };
    activity::Activity::create(&conn, &activity_form)?;

    let community = Community::read(conn, self.community_id)?;
    send_activity(
      &remove,
      &mod_.private_key.as_ref().unwrap(),
      &mod_.actor_id,
      community.get_follower_inboxes(&conn)?,
    )?;
    Ok(())
  }
  fn send_undo_remove(&self, mod_: &User_, conn: &PgConnection) -> Result<(), Error> {
    let page = self.to_apub(conn)?;
    let community = Community::read(conn, self.community_id)?;
    let id = format!("{}/remove/{}", self.ap_id, uuid::Uuid::new_v4());
    let mut remove = Remove::default();

    populate_object_props(
      &mut remove.object_props,
      &community.get_followers_url(),
      &id,
    )?;

    remove
      .remove_props
      .set_actor_xsd_any_uri(mod_.actor_id.to_owned())?
      .set_object_base_box(page)?;

    // Undo that fake activity
    let undo_id = format!("{}/undo/remove/{}", self.ap_id, uuid::Uuid::new_v4());
    let mut undo = Undo::default();

    populate_object_props(
      &mut undo.object_props,
      &community.get_followers_url(),
      &undo_id,
    )?;

    undo
      .undo_props
      .set_actor_xsd_any_uri(mod_.actor_id.to_owned())?
      .set_object_base_box(remove)?;

    // Insert the sent activity into the activity table
    let activity_form = activity::ActivityForm {
      user_id: mod_.id,
      data: serde_json::to_value(&undo)?,
      local: true,
      updated: None,
    };
    activity::Activity::create(&conn, &activity_form)?;

    let community = Community::read(conn, self.community_id)?;
    send_activity(
      &undo,
      &mod_.private_key.as_ref().unwrap(),
      &mod_.actor_id,
      community.get_follower_inboxes(&conn)?,
    )?;
    Ok(())
  }
}

impl ApubLikeableType for Post {
  fn send_like(&self, creator: &User_, conn: &PgConnection) -> Result<(), Error> {
    let page = self.to_apub(conn)?;
    let community = Community::read(conn, self.community_id)?;
    let id = format!("{}/like/{}", self.ap_id, uuid::Uuid::new_v4());

    let mut like = Like::new();
    populate_object_props(&mut like.object_props, &community.get_followers_url(), &id)?;
    like
      .like_props
      .set_actor_xsd_any_uri(creator.actor_id.to_owned())?
      .set_object_base_box(page)?;

    // Insert the sent activity into the activity table
    let activity_form = activity::ActivityForm {
      user_id: creator.id,
      data: serde_json::to_value(&like)?,
      local: true,
      updated: None,
    };
    activity::Activity::create(&conn, &activity_form)?;

    send_activity(
      &like,
      &creator.private_key.as_ref().unwrap(),
      &creator.actor_id,
      community.get_follower_inboxes(&conn)?,
    )?;
    Ok(())
  }

  fn send_dislike(&self, creator: &User_, conn: &PgConnection) -> Result<(), Error> {
    let page = self.to_apub(conn)?;
    let community = Community::read(conn, self.community_id)?;
    let id = format!("{}/dislike/{}", self.ap_id, uuid::Uuid::new_v4());

    let mut dislike = Dislike::new();
    populate_object_props(
      &mut dislike.object_props,
      &community.get_followers_url(),
      &id,
    )?;
    dislike
      .dislike_props
      .set_actor_xsd_any_uri(creator.actor_id.to_owned())?
      .set_object_base_box(page)?;

    // Insert the sent activity into the activity table
    let activity_form = activity::ActivityForm {
      user_id: creator.id,
      data: serde_json::to_value(&dislike)?,
      local: true,
      updated: None,
    };
    activity::Activity::create(&conn, &activity_form)?;

    send_activity(
      &dislike,
      &creator.private_key.as_ref().unwrap(),
      &creator.actor_id,
      community.get_follower_inboxes(&conn)?,
    )?;
    Ok(())
  }

  fn send_undo_like(&self, creator: &User_, conn: &PgConnection) -> Result<(), Error> {
    let page = self.to_apub(conn)?;
    let community = Community::read(conn, self.community_id)?;
    let id = format!("{}/like/{}", self.ap_id, uuid::Uuid::new_v4());

    let mut like = Like::new();
    populate_object_props(&mut like.object_props, &community.get_followers_url(), &id)?;
    like
      .like_props
      .set_actor_xsd_any_uri(creator.actor_id.to_owned())?
      .set_object_base_box(page)?;

    // TODO
    // Undo that fake activity
    let undo_id = format!("{}/undo/like/{}", self.ap_id, uuid::Uuid::new_v4());
    let mut undo = Undo::default();

    populate_object_props(
      &mut undo.object_props,
      &community.get_followers_url(),
      &undo_id,
    )?;

    undo
      .undo_props
      .set_actor_xsd_any_uri(creator.actor_id.to_owned())?
      .set_object_base_box(like)?;

    // Insert the sent activity into the activity table
    let activity_form = activity::ActivityForm {
      user_id: creator.id,
      data: serde_json::to_value(&undo)?,
      local: true,
      updated: None,
    };
    activity::Activity::create(&conn, &activity_form)?;

    send_activity(
      &undo,
      &creator.private_key.as_ref().unwrap(),
      &creator.actor_id,
      community.get_follower_inboxes(&conn)?,
    )?;
    Ok(())
  }
}
