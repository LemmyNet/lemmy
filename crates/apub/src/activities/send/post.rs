use crate::{
  activities::send::generate_activity_id,
  activity_queue::send_to_community,
  extensions::context::lemmy_context,
  objects::ToApub,
  ActorType,
  ApubLikeableType,
  ApubObjectType,
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
  /// Send out information about a newly created post, to the followers of the community.
  async fn send_create(&self, creator: &Person, context: &LemmyContext) -> Result<(), LemmyError> {
    let page = self.to_apub(context.pool()).await?;

    let community_id = self.community_id;
    let community = blocking(context.pool(), move |conn| {
      Community::read(conn, community_id)
    })
    .await??;

    let mut create = Create::new(
      creator.actor_id.to_owned().into_inner(),
      page.into_any_base()?,
    );
    create
      .set_many_contexts(lemmy_context()?)
      .set_id(generate_activity_id(CreateType::Create)?)
      .set_to(public())
      .set_many_ccs(vec![community.actor_id()]);

    send_to_community(create, creator, &community, context).await?;
    Ok(())
  }

  /// Send out information about an edited post, to the followers of the community.
  async fn send_update(&self, creator: &Person, context: &LemmyContext) -> Result<(), LemmyError> {
    let page = self.to_apub(context.pool()).await?;

    let community_id = self.community_id;
    let community = blocking(context.pool(), move |conn| {
      Community::read(conn, community_id)
    })
    .await??;

    let mut update = Update::new(
      creator.actor_id.to_owned().into_inner(),
      page.into_any_base()?,
    );
    update
      .set_many_contexts(lemmy_context()?)
      .set_id(generate_activity_id(UpdateType::Update)?)
      .set_to(public())
      .set_many_ccs(vec![community.actor_id()]);

    send_to_community(update, creator, &community, context).await?;
    Ok(())
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
      .set_many_contexts(lemmy_context()?)
      .set_id(generate_activity_id(DeleteType::Delete)?)
      .set_to(public())
      .set_many_ccs(vec![community.actor_id()]);

    send_to_community(delete, creator, &community, context).await?;
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
      .set_many_contexts(lemmy_context()?)
      .set_id(generate_activity_id(DeleteType::Delete)?)
      .set_to(public())
      .set_many_ccs(vec![community.actor_id()]);

    // Undo that fake activity
    let mut undo = Undo::new(
      creator.actor_id.to_owned().into_inner(),
      delete.into_any_base()?,
    );
    undo
      .set_many_contexts(lemmy_context()?)
      .set_id(generate_activity_id(UndoType::Undo)?)
      .set_to(public())
      .set_many_ccs(vec![community.actor_id()]);

    send_to_community(undo, creator, &community, context).await?;
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
      .set_many_contexts(lemmy_context()?)
      .set_id(generate_activity_id(RemoveType::Remove)?)
      .set_to(public())
      .set_many_ccs(vec![community.actor_id()]);

    send_to_community(remove, mod_, &community, context).await?;
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
      .set_many_contexts(lemmy_context()?)
      .set_id(generate_activity_id(RemoveType::Remove)?)
      .set_to(public())
      .set_many_ccs(vec![community.actor_id()]);

    // Undo that fake activity
    let mut undo = Undo::new(
      mod_.actor_id.to_owned().into_inner(),
      remove.into_any_base()?,
    );
    undo
      .set_many_contexts(lemmy_context()?)
      .set_id(generate_activity_id(UndoType::Undo)?)
      .set_to(public())
      .set_many_ccs(vec![community.actor_id()]);

    send_to_community(undo, mod_, &community, context).await?;
    Ok(())
  }
}

#[async_trait::async_trait(?Send)]
impl ApubLikeableType for Post {
  async fn send_like(&self, creator: &Person, context: &LemmyContext) -> Result<(), LemmyError> {
    let community_id = self.community_id;
    let community = blocking(context.pool(), move |conn| {
      Community::read(conn, community_id)
    })
    .await??;

    let mut like = Like::new(
      creator.actor_id.to_owned().into_inner(),
      self.ap_id.to_owned().into_inner(),
    );
    like
      .set_many_contexts(lemmy_context()?)
      .set_id(generate_activity_id(LikeType::Like)?)
      .set_to(public())
      .set_many_ccs(vec![community.actor_id()]);

    send_to_community(like, &creator, &community, context).await?;
    Ok(())
  }

  async fn send_dislike(&self, creator: &Person, context: &LemmyContext) -> Result<(), LemmyError> {
    let community_id = self.community_id;
    let community = blocking(context.pool(), move |conn| {
      Community::read(conn, community_id)
    })
    .await??;

    let mut dislike = Dislike::new(
      creator.actor_id.to_owned().into_inner(),
      self.ap_id.to_owned().into_inner(),
    );
    dislike
      .set_many_contexts(lemmy_context()?)
      .set_id(generate_activity_id(DislikeType::Dislike)?)
      .set_to(public())
      .set_many_ccs(vec![community.actor_id()]);

    send_to_community(dislike, &creator, &community, context).await?;
    Ok(())
  }

  async fn send_undo_like(
    &self,
    creator: &Person,
    context: &LemmyContext,
  ) -> Result<(), LemmyError> {
    let community_id = self.community_id;
    let community = blocking(context.pool(), move |conn| {
      Community::read(conn, community_id)
    })
    .await??;

    let mut like = Like::new(
      creator.actor_id.to_owned().into_inner(),
      self.ap_id.to_owned().into_inner(),
    );
    like
      .set_many_contexts(lemmy_context()?)
      .set_id(generate_activity_id(LikeType::Like)?)
      .set_to(public())
      .set_many_ccs(vec![community.actor_id()]);

    // Undo that fake activity
    let mut undo = Undo::new(
      creator.actor_id.to_owned().into_inner(),
      like.into_any_base()?,
    );
    undo
      .set_many_contexts(lemmy_context()?)
      .set_id(generate_activity_id(UndoType::Undo)?)
      .set_to(public())
      .set_many_ccs(vec![community.actor_id()]);

    send_to_community(undo, &creator, &community, context).await?;
    Ok(())
  }
}
