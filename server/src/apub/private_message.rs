use crate::{
  apub::{
    activities::send_activity,
    create_tombstone,
    fetcher::get_or_fetch_and_upsert_remote_user,
    ApubObjectType,
    FromApub,
    ToApub,
  },
  blocking,
  convert_datetime,
  db::{
    activity::insert_activity,
    private_message::{PrivateMessage, PrivateMessageForm},
    user::User_,
    Crud,
  },
  DbPool,
  LemmyError,
};
use activitystreams::{
  activity::{Create, Delete, Undo, Update},
  context,
  object::{kind::NoteType, properties::ObjectProperties, Note},
};
use activitystreams_new::object::Tombstone;
use actix_web::client::Client;

#[async_trait::async_trait(?Send)]
impl ToApub for PrivateMessage {
  type Response = Note;

  async fn to_apub(&self, pool: &DbPool) -> Result<Note, LemmyError> {
    let mut private_message = Note::default();
    let oprops: &mut ObjectProperties = private_message.as_mut();

    let creator_id = self.creator_id;
    let creator = blocking(pool, move |conn| User_::read(conn, creator_id)).await??;

    let recipient_id = self.recipient_id;
    let recipient = blocking(pool, move |conn| User_::read(conn, recipient_id)).await??;

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

  fn to_tombstone(&self) -> Result<Tombstone, LemmyError> {
    create_tombstone(
      self.deleted,
      &self.ap_id,
      self.updated,
      NoteType.to_string(),
    )
  }
}

#[async_trait::async_trait(?Send)]
impl FromApub for PrivateMessageForm {
  type ApubType = Note;

  /// Parse an ActivityPub note received from another instance into a Lemmy Private message
  async fn from_apub(
    note: &Note,
    client: &Client,
    pool: &DbPool,
  ) -> Result<PrivateMessageForm, LemmyError> {
    let oprops = &note.object_props;
    let creator_actor_id = &oprops.get_attributed_to_xsd_any_uri().unwrap().to_string();

    let creator = get_or_fetch_and_upsert_remote_user(&creator_actor_id, client, pool).await?;

    let recipient_actor_id = &oprops.get_to_xsd_any_uri().unwrap().to_string();

    let recipient = get_or_fetch_and_upsert_remote_user(&recipient_actor_id, client, pool).await?;

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

#[async_trait::async_trait(?Send)]
impl ApubObjectType for PrivateMessage {
  /// Send out information about a newly created private message
  async fn send_create(
    &self,
    creator: &User_,
    client: &Client,
    pool: &DbPool,
  ) -> Result<(), LemmyError> {
    let note = self.to_apub(pool).await?;
    let id = format!("{}/create/{}", self.ap_id, uuid::Uuid::new_v4());

    let recipient_id = self.recipient_id;
    let recipient = blocking(pool, move |conn| User_::read(conn, recipient_id)).await??;

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

    insert_activity(creator.id, create.clone(), true, pool).await?;

    send_activity(client, &create, creator, vec![to]).await?;
    Ok(())
  }

  /// Send out information about an edited post, to the followers of the community.
  async fn send_update(
    &self,
    creator: &User_,
    client: &Client,
    pool: &DbPool,
  ) -> Result<(), LemmyError> {
    let note = self.to_apub(pool).await?;
    let id = format!("{}/update/{}", self.ap_id, uuid::Uuid::new_v4());

    let recipient_id = self.recipient_id;
    let recipient = blocking(pool, move |conn| User_::read(conn, recipient_id)).await??;

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

    insert_activity(creator.id, update.clone(), true, pool).await?;

    send_activity(client, &update, creator, vec![to]).await?;
    Ok(())
  }

  async fn send_delete(
    &self,
    creator: &User_,
    client: &Client,
    pool: &DbPool,
  ) -> Result<(), LemmyError> {
    let note = self.to_apub(pool).await?;
    let id = format!("{}/delete/{}", self.ap_id, uuid::Uuid::new_v4());

    let recipient_id = self.recipient_id;
    let recipient = blocking(pool, move |conn| User_::read(conn, recipient_id)).await??;

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

    insert_activity(creator.id, delete.clone(), true, pool).await?;

    send_activity(client, &delete, creator, vec![to]).await?;
    Ok(())
  }

  async fn send_undo_delete(
    &self,
    creator: &User_,
    client: &Client,
    pool: &DbPool,
  ) -> Result<(), LemmyError> {
    let note = self.to_apub(pool).await?;
    let id = format!("{}/delete/{}", self.ap_id, uuid::Uuid::new_v4());

    let recipient_id = self.recipient_id;
    let recipient = blocking(pool, move |conn| User_::read(conn, recipient_id)).await??;

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

    insert_activity(creator.id, undo.clone(), true, pool).await?;

    send_activity(client, &undo, creator, vec![to]).await?;
    Ok(())
  }

  async fn send_remove(
    &self,
    _mod_: &User_,
    _client: &Client,
    _pool: &DbPool,
  ) -> Result<(), LemmyError> {
    unimplemented!()
  }

  async fn send_undo_remove(
    &self,
    _mod_: &User_,
    _client: &Client,
    _pool: &DbPool,
  ) -> Result<(), LemmyError> {
    unimplemented!()
  }
}
