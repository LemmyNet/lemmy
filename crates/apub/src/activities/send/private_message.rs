use crate::{
  activities::send::generate_activity_id,
  activity_queue::send_activity_single_dest,
  extensions::context::lemmy_context,
  objects::ToApub,
  ActorType,
  ApubObjectType,
};
use activitystreams::{
  activity::{
    kind::{CreateType, DeleteType, UndoType, UpdateType},
    Create,
    Delete,
    Undo,
    Update,
  },
  prelude::*,
};
use lemmy_api_common::blocking;
use lemmy_db_queries::Crud;
use lemmy_db_schema::source::{person::Person, private_message::PrivateMessage};
use lemmy_utils::LemmyError;
use lemmy_websocket::LemmyContext;

#[async_trait::async_trait(?Send)]
impl ApubObjectType for PrivateMessage {
  /// Send out information about a newly created private message
  async fn send_create(&self, creator: &Person, context: &LemmyContext) -> Result<(), LemmyError> {
    let note = self.to_apub(context.pool()).await?;

    let recipient_id = self.recipient_id;
    let recipient =
      blocking(context.pool(), move |conn| Person::read(conn, recipient_id)).await??;

    let mut create = Create::new(
      creator.actor_id.to_owned().into_inner(),
      note.into_any_base()?,
    );

    create
      .set_many_contexts(lemmy_context()?)
      .set_id(generate_activity_id(CreateType::Create)?)
      .set_to(recipient.actor_id());

    send_activity_single_dest(create, creator, recipient.inbox_url.into(), context).await?;
    Ok(())
  }

  /// Send out information about an edited private message, to the followers of the community.
  async fn send_update(&self, creator: &Person, context: &LemmyContext) -> Result<(), LemmyError> {
    let note = self.to_apub(context.pool()).await?;

    let recipient_id = self.recipient_id;
    let recipient =
      blocking(context.pool(), move |conn| Person::read(conn, recipient_id)).await??;

    let mut update = Update::new(
      creator.actor_id.to_owned().into_inner(),
      note.into_any_base()?,
    );
    update
      .set_many_contexts(lemmy_context()?)
      .set_id(generate_activity_id(UpdateType::Update)?)
      .set_to(recipient.actor_id());

    send_activity_single_dest(update, creator, recipient.inbox_url.into(), context).await?;
    Ok(())
  }

  async fn send_delete(&self, creator: &Person, context: &LemmyContext) -> Result<(), LemmyError> {
    let recipient_id = self.recipient_id;
    let recipient =
      blocking(context.pool(), move |conn| Person::read(conn, recipient_id)).await??;

    let mut delete = Delete::new(
      creator.actor_id.to_owned().into_inner(),
      self.ap_id.to_owned().into_inner(),
    );
    delete
      .set_many_contexts(lemmy_context()?)
      .set_id(generate_activity_id(DeleteType::Delete)?)
      .set_to(recipient.actor_id());

    send_activity_single_dest(delete, creator, recipient.inbox_url.into(), context).await?;
    Ok(())
  }

  async fn send_undo_delete(
    &self,
    creator: &Person,
    context: &LemmyContext,
  ) -> Result<(), LemmyError> {
    let recipient_id = self.recipient_id;
    let recipient =
      blocking(context.pool(), move |conn| Person::read(conn, recipient_id)).await??;

    let mut delete = Delete::new(
      creator.actor_id.to_owned().into_inner(),
      self.ap_id.to_owned().into_inner(),
    );
    delete
      .set_many_contexts(lemmy_context()?)
      .set_id(generate_activity_id(DeleteType::Delete)?)
      .set_to(recipient.actor_id());

    // Undo that fake activity
    let mut undo = Undo::new(
      creator.actor_id.to_owned().into_inner(),
      delete.into_any_base()?,
    );
    undo
      .set_many_contexts(lemmy_context()?)
      .set_id(generate_activity_id(UndoType::Undo)?)
      .set_to(recipient.actor_id());

    send_activity_single_dest(undo, creator, recipient.inbox_url.into(), context).await?;
    Ok(())
  }

  async fn send_remove(&self, _mod_: &Person, _context: &LemmyContext) -> Result<(), LemmyError> {
    unimplemented!()
  }

  async fn send_undo_remove(
    &self,
    _mod_: &Person,
    _context: &LemmyContext,
  ) -> Result<(), LemmyError> {
    unimplemented!()
  }
}
