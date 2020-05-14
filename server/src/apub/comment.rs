use super::*;

impl ToApub for Comment {
  type Response = Note;

  fn to_apub(&self, conn: &PgConnection) -> Result<Note, Error> {
    let mut comment = Note::default();
    let oprops: &mut ObjectProperties = comment.as_mut();
    let creator = User_::read(&conn, self.creator_id)?;
    let post = Post::read(&conn, self.post_id)?;
    let community = Community::read(&conn, post.community_id)?;

    // Add a vector containing some important info to the "in_reply_to" field
    // [post_ap_id, Option(parent_comment_ap_id)]
    let mut in_reply_to_vec = vec![post.ap_id];

    if let Some(parent_id) = self.parent_id {
      let parent_comment = Comment::read(&conn, parent_id)?;
      in_reply_to_vec.push(parent_comment.ap_id);
    }

    oprops
      // Not needed when the Post is embedded in a collection (like for community outbox)
      .set_context_xsd_any_uri(context())?
      .set_id(self.ap_id.to_owned())?
      .set_published(convert_datetime(self.published))?
      .set_to_xsd_any_uri(community.actor_id)?
      .set_many_in_reply_to_xsd_any_uris(in_reply_to_vec)?
      .set_content_xsd_string(self.content.to_owned())?
      .set_attributed_to_xsd_any_uri(creator.actor_id)?;

    if let Some(u) = self.updated {
      oprops.set_updated(convert_datetime(u))?;
    }

    Ok(comment)
  }

  fn to_tombstone(&self) -> Result<Tombstone, Error> {
    create_tombstone(
      self.deleted,
      &self.ap_id,
      self.updated,
      NoteType.to_string(),
    )
  }
}

impl FromApub for CommentForm {
  type ApubType = Note;

  /// Parse an ActivityPub note received from another instance into a Lemmy comment
  fn from_apub(note: &Note, conn: &PgConnection) -> Result<CommentForm, Error> {
    let oprops = &note.object_props;
    let creator_actor_id = &oprops.get_attributed_to_xsd_any_uri().unwrap().to_string();
    let creator = get_or_fetch_and_upsert_remote_user(&creator_actor_id, &conn)?;

    let mut in_reply_tos = oprops.get_many_in_reply_to_xsd_any_uris().unwrap();
    let post_ap_id = in_reply_tos.next().unwrap().to_string();

    // The 2nd item, if it exists, is the parent comment apub_id
    let parent_id: Option<i32> = match in_reply_tos.next() {
      Some(parent_comment_uri) => {
        let parent_comment_uri_str = &parent_comment_uri.to_string();
        let parent_comment = Comment::read_from_apub_id(&conn, &parent_comment_uri_str)?;

        Some(parent_comment.id)
      }
      None => None,
    };

    // TODO this failed because a mention on a post that wasn't on this server yet. Has to do with
    // fetching replytos
    let post = Post::read_from_apub_id(&conn, &post_ap_id)?;

    Ok(CommentForm {
      creator_id: creator.id,
      post_id: post.id,
      parent_id,
      content: oprops
        .get_content_xsd_string()
        .map(|c| c.to_string())
        .unwrap(),
      removed: None,
      read: None,
      published: oprops
        .get_published()
        .map(|u| u.as_ref().to_owned().naive_local()),
      updated: oprops
        .get_updated()
        .map(|u| u.as_ref().to_owned().naive_local()),
      deleted: None,
      ap_id: oprops.get_id().unwrap().to_string(),
      local: false,
    })
  }
}

impl ApubObjectType for Comment {
  /// Send out information about a newly created comment, to the followers of the community.
  fn send_create(&self, creator: &User_, conn: &PgConnection) -> Result<(), Error> {
    let note = self.to_apub(conn)?;
    let post = Post::read(&conn, self.post_id)?;
    let community = Community::read(conn, post.community_id)?;
    let id = format!("{}/create/{}", self.ap_id, uuid::Uuid::new_v4());

    let maa: MentionsAndAddresses =
      collect_non_local_mentions_and_addresses(&conn, &self.content, &community)?;

    let mut create = Create::new();
    populate_object_props(&mut create.object_props, maa.addressed_ccs, &id)?;

    // Set the mention tags
    create.object_props.set_many_tag_base_boxes(maa.tags)?;

    create
      .create_props
      .set_actor_xsd_any_uri(creator.actor_id.to_owned())?
      .set_object_base_box(note)?;

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
      maa.inboxes,
    )?;
    Ok(())
  }

  /// Send out information about an edited post, to the followers of the community.
  fn send_update(&self, creator: &User_, conn: &PgConnection) -> Result<(), Error> {
    let note = self.to_apub(&conn)?;
    let post = Post::read(&conn, self.post_id)?;
    let community = Community::read(&conn, post.community_id)?;
    let id = format!("{}/update/{}", self.ap_id, uuid::Uuid::new_v4());

    let maa: MentionsAndAddresses =
      collect_non_local_mentions_and_addresses(&conn, &self.content, &community)?;

    let mut update = Update::new();
    populate_object_props(&mut update.object_props, maa.addressed_ccs, &id)?;

    // Set the mention tags
    update.object_props.set_many_tag_base_boxes(maa.tags)?;

    update
      .update_props
      .set_actor_xsd_any_uri(creator.actor_id.to_owned())?
      .set_object_base_box(note)?;

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
      maa.inboxes,
    )?;
    Ok(())
  }

  fn send_delete(&self, creator: &User_, conn: &PgConnection) -> Result<(), Error> {
    let note = self.to_apub(&conn)?;
    let post = Post::read(&conn, self.post_id)?;
    let community = Community::read(&conn, post.community_id)?;
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
      .set_object_base_box(note)?;

    // Insert the sent activity into the activity table
    let activity_form = activity::ActivityForm {
      user_id: creator.id,
      data: serde_json::to_value(&delete)?,
      local: true,
      updated: None,
    };
    activity::Activity::create(&conn, &activity_form)?;

    send_activity(
      &delete,
      &creator.private_key.as_ref().unwrap(),
      &creator.actor_id,
      community.get_follower_inboxes(&conn)?,
    )?;
    Ok(())
  }

  fn send_undo_delete(&self, creator: &User_, conn: &PgConnection) -> Result<(), Error> {
    let note = self.to_apub(&conn)?;
    let post = Post::read(&conn, self.post_id)?;
    let community = Community::read(&conn, post.community_id)?;

    // Generate a fake delete activity, with the correct object
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
      .set_object_base_box(note)?;

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

  fn send_remove(&self, mod_: &User_, conn: &PgConnection) -> Result<(), Error> {
    let note = self.to_apub(&conn)?;
    let post = Post::read(&conn, self.post_id)?;
    let community = Community::read(&conn, post.community_id)?;
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
      .set_object_base_box(note)?;

    // Insert the sent activity into the activity table
    let activity_form = activity::ActivityForm {
      user_id: mod_.id,
      data: serde_json::to_value(&remove)?,
      local: true,
      updated: None,
    };
    activity::Activity::create(&conn, &activity_form)?;

    send_activity(
      &remove,
      &mod_.private_key.as_ref().unwrap(),
      &mod_.actor_id,
      community.get_follower_inboxes(&conn)?,
    )?;
    Ok(())
  }

  fn send_undo_remove(&self, mod_: &User_, conn: &PgConnection) -> Result<(), Error> {
    let note = self.to_apub(&conn)?;
    let post = Post::read(&conn, self.post_id)?;
    let community = Community::read(&conn, post.community_id)?;

    // Generate a fake delete activity, with the correct object
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
      .set_object_base_box(note)?;

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

    // Insert the sent activity into the activity table
    let activity_form = activity::ActivityForm {
      user_id: mod_.id,
      data: serde_json::to_value(&undo)?,
      local: true,
      updated: None,
    };
    activity::Activity::create(&conn, &activity_form)?;

    send_activity(
      &undo,
      &mod_.private_key.as_ref().unwrap(),
      &mod_.actor_id,
      community.get_follower_inboxes(&conn)?,
    )?;
    Ok(())
  }
}

impl ApubLikeableType for Comment {
  fn send_like(&self, creator: &User_, conn: &PgConnection) -> Result<(), Error> {
    let note = self.to_apub(&conn)?;
    let post = Post::read(&conn, self.post_id)?;
    let community = Community::read(&conn, post.community_id)?;
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
      .set_object_base_box(note)?;

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
    let note = self.to_apub(&conn)?;
    let post = Post::read(&conn, self.post_id)?;
    let community = Community::read(&conn, post.community_id)?;
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
      .set_object_base_box(note)?;

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
    let note = self.to_apub(&conn)?;
    let post = Post::read(&conn, self.post_id)?;
    let community = Community::read(&conn, post.community_id)?;
    let id = format!("{}/dislike/{}", self.ap_id, uuid::Uuid::new_v4());

    let mut like = Like::new();
    populate_object_props(
      &mut like.object_props,
      vec![community.get_followers_url()],
      &id,
    )?;
    like
      .like_props
      .set_actor_xsd_any_uri(creator.actor_id.to_owned())?
      .set_object_base_box(note)?;

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

struct MentionsAndAddresses {
  addressed_ccs: Vec<String>,
  inboxes: Vec<String>,
  tags: Vec<Mention>,
}

fn collect_non_local_mentions_and_addresses(
  conn: &PgConnection,
  content: &str,
  community: &Community,
) -> Result<MentionsAndAddresses, Error> {
  let mut addressed_ccs = vec![community.get_followers_url()];

  // Add the mention tag
  let mut tags = Vec::new();

  // Get the inboxes for any mentions
  let mentions = scrape_text_for_mentions(&content)
    .into_iter()
    // Filter only the non-local ones
    .filter(|m| !m.is_local())
    .collect::<Vec<MentionData>>();
  let mut mention_inboxes = Vec::new();
  for mention in &mentions {
    // TODO should it be fetching it every time?
    if let Ok(actor_id) = fetch_webfinger_url(mention) {
      debug!("mention actor_id: {}", actor_id);
      addressed_ccs.push(actor_id.to_owned());
      let mention_user = get_or_fetch_and_upsert_remote_user(&actor_id, &conn)?;
      let shared_inbox = mention_user.get_shared_inbox_url();
      mention_inboxes.push(shared_inbox);
      let mut mention_tag = Mention::new();
      mention_tag
        .link_props
        .set_href(actor_id)?
        .set_name_xsd_string(mention.full_name())?;
      tags.push(mention_tag);
    }
  }

  let mut inboxes = community.get_follower_inboxes(&conn)?;
  inboxes.extend(mention_inboxes);
  inboxes = inboxes.into_iter().unique().collect();

  Ok(MentionsAndAddresses {
    addressed_ccs,
    inboxes,
    tags,
  })
}
