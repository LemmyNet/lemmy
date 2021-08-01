use crate::{
  activities::generate_activity_id,
  activity_queue::send_to_community,
  extensions::context::lemmy_context,
  ActorType,
  ApubObjectType,
};
use activitystreams::{
  activity::{
    kind::{DeleteType, RemoveType, UndoType},
    Delete,
    Remove,
    Undo,
  },
  prelude::*,
  public,
};
use lemmy_api_common::blocking;
use lemmy_db_queries::Crud;
use lemmy_db_schema::source::{community::Community, person::Person, post::Post};
use lemmy_utils::LemmyError;
use lemmy_websocket::LemmyContext;

#[async_trait::async_trait(?Send)]
impl ApubObjectType for Post {
  async fn send_create(
    &self,
    _creator: &Person,
    _context: &LemmyContext,
  ) -> Result<(), LemmyError> {
    unimplemented!()
  }

  async fn send_update(
    &self,
    _creator: &Person,
    _context: &LemmyContext,
  ) -> Result<(), LemmyError> {
    unimplemented!()
  }

  async fn send_delete(&self, creator: &Person, context: &LemmyContext) -> Result<(), LemmyError> {
    let community_id = self.community_id;
    let community = blocking(context.pool(), move |conn| {
      Community::read(conn, community_id)
    })
    .await??;

    let mut delete = Delete::new(
      creator.actor_id.to_owned().into_inner(),
      self.ap_id.to_owned().into_inner(),
    );
    delete
      .set_many_contexts(lemmy_context())
      .set_id(generate_activity_id(DeleteType::Delete)?)
      .set_to(public())
      .set_many_ccs(vec![community.actor_id()]);

    send_to_community(delete, creator, &community, None, context).await?;
    Ok(())
  }

  async fn send_undo_delete(
    &self,
    creator: &Person,
    context: &LemmyContext,
  ) -> Result<(), LemmyError> {
    let community_id = self.community_id;
    let community = blocking(context.pool(), move |conn| {
      Community::read(conn, community_id)
    })
    .await??;

    let mut delete = Delete::new(
      creator.actor_id.to_owned().into_inner(),
      self.ap_id.to_owned().into_inner(),
    );
    delete
      .set_many_contexts(lemmy_context())
      .set_id(generate_activity_id(DeleteType::Delete)?)
      .set_to(public())
      .set_many_ccs(vec![community.actor_id()]);

    // Undo that fake activity
    let mut undo = Undo::new(
      creator.actor_id.to_owned().into_inner(),
      delete.into_any_base()?,
    );
    undo
      .set_many_contexts(lemmy_context())
      .set_id(generate_activity_id(UndoType::Undo)?)
      .set_to(public())
      .set_many_ccs(vec![community.actor_id()]);

    send_to_community(undo, creator, &community, None, context).await?;
    Ok(())
  }

  async fn send_remove(&self, mod_: &Person, context: &LemmyContext) -> Result<(), LemmyError> {
    let community_id = self.community_id;
    let community = blocking(context.pool(), move |conn| {
      Community::read(conn, community_id)
    })
    .await??;

    let mut remove = Remove::new(
      mod_.actor_id.to_owned().into_inner(),
      self.ap_id.to_owned().into_inner(),
    );
    remove
      .set_many_contexts(lemmy_context())
      .set_id(generate_activity_id(RemoveType::Remove)?)
      .set_to(public())
      .set_many_ccs(vec![community.actor_id()]);

    send_to_community(remove, mod_, &community, None, context).await?;
    Ok(())
  }

  async fn send_undo_remove(
    &self,
    mod_: &Person,
    context: &LemmyContext,
  ) -> Result<(), LemmyError> {
    let community_id = self.community_id;
    let community = blocking(context.pool(), move |conn| {
      Community::read(conn, community_id)
    })
    .await??;

    let mut remove = Remove::new(
      mod_.actor_id.to_owned().into_inner(),
      self.ap_id.to_owned().into_inner(),
    );
    remove
      .set_many_contexts(lemmy_context())
      .set_id(generate_activity_id(RemoveType::Remove)?)
      .set_to(public())
      .set_many_ccs(vec![community.actor_id()]);

    // Undo that fake activity
    let mut undo = Undo::new(
      mod_.actor_id.to_owned().into_inner(),
      remove.into_any_base()?,
    );
    undo
      .set_many_contexts(lemmy_context())
      .set_id(generate_activity_id(UndoType::Undo)?)
      .set_to(public())
      .set_many_ccs(vec![community.actor_id()]);

    send_to_community(undo, mod_, &community, None, context).await?;
    Ok(())
  }
}
