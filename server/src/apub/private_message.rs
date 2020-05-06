use super::*;

impl ToApub for PrivateMessage {
  type Response = Note;

  fn to_apub(&self, conn: &PgConnection) -> Result<Note, Error> {
    let mut private_message = Note::default();
    let oprops: &mut ObjectProperties = private_message.as_mut();
    let creator = User_::read(&conn, self.creator_id)?;
    let recipient = User_::read(&conn, self.recipient_id)?;

    oprops
      .set_context_xsd_any_uri(context())?
      .set_id(self.ap_id.to_owned())?
      .set_published(convert_datetime(self.published))?
      .set_content_xsd_string(self.content.to_owned())?
      .set_to_xsd_any_uri(recipient.actor_id)?
      .set_attributed_to_xsd_any_uri(creator.actor_id)?;

    if let Some(u) = self.updated {
      oprops.set_updated(convert_datetime(u))?;
    }

    Ok(private_message)
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

impl FromApub for PrivateMessageForm {
  type ApubType = Note;

  /// Parse an ActivityPub note received from another instance into a Lemmy Private message
  fn from_apub(note: &Note, conn: &PgConnection) -> Result<PrivateMessageForm, Error> {
    let oprops = &note.object_props;
    let creator_actor_id = &oprops.get_attributed_to_xsd_any_uri().unwrap().to_string();
    let creator = get_or_fetch_and_upsert_remote_user(&creator_actor_id, &conn)?;
    let recipient_actor_id = &oprops.get_to_xsd_any_uri().unwrap().to_string();
    let recipient = get_or_fetch_and_upsert_remote_user(&recipient_actor_id, &conn)?;

    Ok(PrivateMessageForm {
      creator_id: creator.id,
      recipient_id: recipient.id,
      content: oprops
        .get_content_xsd_string()
        .map(|c| c.to_string())
        .unwrap(),
      published: oprops
        .get_published()
        .map(|u| u.as_ref().to_owned().naive_local()),
      updated: oprops
        .get_updated()
        .map(|u| u.as_ref().to_owned().naive_local()),
      deleted: None,
      read: None,
      ap_id: oprops.get_id().unwrap().to_string(),
      local: false,
    })
  }
}

impl ApubObjectType for PrivateMessage {
  /// Send out information about a newly created private message
  fn send_create(&self, creator: &User_, conn: &PgConnection) -> Result<(), Error> {
    let note = self.to_apub(conn)?;
    let id = format!("{}/create/{}", self.ap_id, uuid::Uuid::new_v4());
    let recipient = User_::read(&conn, self.recipient_id)?;

    let mut create = Create::new();
    create
      .object_props
      .set_context_xsd_any_uri(context())?
      .set_id(id)?;
    let to = format!("{}/inbox", recipient.actor_id);

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
      vec![to],
    )?;
    Ok(())
  }

  /// Send out information about an edited post, to the followers of the community.
  fn send_update(&self, creator: &User_, conn: &PgConnection) -> Result<(), Error> {
    let note = self.to_apub(conn)?;
    let id = format!("{}/update/{}", self.ap_id, uuid::Uuid::new_v4());
    let recipient = User_::read(&conn, self.recipient_id)?;

    let mut update = Update::new();
    update
      .object_props
      .set_context_xsd_any_uri(context())?
      .set_id(id)?;
    let to = format!("{}/inbox", recipient.actor_id);

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
      vec![to],
    )?;
    Ok(())
  }

  fn send_delete(&self, creator: &User_, conn: &PgConnection) -> Result<(), Error> {
    let note = self.to_apub(conn)?;
    let id = format!("{}/delete/{}", self.ap_id, uuid::Uuid::new_v4());
    let recipient = User_::read(&conn, self.recipient_id)?;

    let mut delete = Delete::new();
    delete
      .object_props
      .set_context_xsd_any_uri(context())?
      .set_id(id)?;
    let to = format!("{}/inbox", recipient.actor_id);

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
      vec![to],
    )?;
    Ok(())
  }

  fn send_undo_delete(&self, creator: &User_, conn: &PgConnection) -> Result<(), Error> {
    let note = self.to_apub(conn)?;
    let id = format!("{}/delete/{}", self.ap_id, uuid::Uuid::new_v4());
    let recipient = User_::read(&conn, self.recipient_id)?;

    let mut delete = Delete::new();
    delete
      .object_props
      .set_context_xsd_any_uri(context())?
      .set_id(id)?;
    let to = format!("{}/inbox", recipient.actor_id);

    delete
      .delete_props
      .set_actor_xsd_any_uri(creator.actor_id.to_owned())?
      .set_object_base_box(note)?;

    // TODO
    // Undo that fake activity
    let undo_id = format!("{}/undo/delete/{}", self.ap_id, uuid::Uuid::new_v4());
    let mut undo = Undo::default();

    undo
      .object_props
      .set_context_xsd_any_uri(context())?
      .set_id(undo_id)?;

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
      vec![to],
    )?;
    Ok(())
  }

  fn send_remove(&self, _mod_: &User_, _conn: &PgConnection) -> Result<(), Error> {
    unimplemented!()
  }

  fn send_undo_remove(&self, _mod_: &User_, _conn: &PgConnection) -> Result<(), Error> {
    unimplemented!()
  }
}
