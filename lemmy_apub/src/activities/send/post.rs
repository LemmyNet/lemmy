use crate::{
  activities::send::generate_activity_id,
  activity_queue::send_to_community,
  ActorType,
  ApubLikeableType,
  ApubObjectType,
  ToApub,
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
use lemmy_db::{community::Community, post::Post, user::User_, Crud};
use lemmy_structs::blocking;
use lemmy_utils::LemmyError;
use lemmy_websocket::LemmyContext;
use url::Url;

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
      .set_many_ccs(vec![community.actor_id()?]);

    send_to_community(creator, &community, create, context).await?;
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
      .set_many_ccs(vec![community.actor_id()?]);

    send_to_community(creator, &community, update, context).await?;
    Ok(())
  }

  async fn send_delete(&self, creator: &User_, context: &LemmyContext) -> Result<(), LemmyError> {
    let community_id = self.community_id;
    let community = blocking(context.pool(), move |conn| {
      Community::read(conn, community_id)
    })
    .await??;

    let mut delete = Delete::new(creator.actor_id.to_owned(), Url::parse(&self.ap_id)?);
    delete
      .set_context(activitystreams::context())
      .set_id(generate_activity_id(DeleteType::Delete)?)
      .set_to(public())
      .set_many_ccs(vec![community.actor_id()?]);

    send_to_community(creator, &community, delete, context).await?;
    Ok(())
  }

  async fn send_undo_delete(
    &self,
    creator: &User_,
    context: &LemmyContext,
  ) -> Result<(), LemmyError> {
    let community_id = self.community_id;
    let community = blocking(context.pool(), move |conn| {
      Community::read(conn, community_id)
    })
    .await??;

    let mut delete = Delete::new(creator.actor_id.to_owned(), Url::parse(&self.ap_id)?);
    delete
      .set_context(activitystreams::context())
      .set_id(generate_activity_id(DeleteType::Delete)?)
      .set_to(public())
      .set_many_ccs(vec![community.actor_id()?]);

    // Undo that fake activity
    let mut undo = Undo::new(creator.actor_id.to_owned(), delete.into_any_base()?);
    undo
      .set_context(activitystreams::context())
      .set_id(generate_activity_id(UndoType::Undo)?)
      .set_to(public())
      .set_many_ccs(vec![community.actor_id()?]);

    send_to_community(creator, &community, undo, context).await?;
    Ok(())
  }

  async fn send_remove(&self, mod_: &User_, context: &LemmyContext) -> Result<(), LemmyError> {
    let community_id = self.community_id;
    let community = blocking(context.pool(), move |conn| {
      Community::read(conn, community_id)
    })
    .await??;

    let mut remove = Remove::new(mod_.actor_id.to_owned(), Url::parse(&self.ap_id)?);
    remove
      .set_context(activitystreams::context())
      .set_id(generate_activity_id(RemoveType::Remove)?)
      .set_to(public())
      .set_many_ccs(vec![community.actor_id()?]);

    send_to_community(mod_, &community, remove, context).await?;
    Ok(())
  }

  async fn send_undo_remove(&self, mod_: &User_, context: &LemmyContext) -> Result<(), LemmyError> {
    let community_id = self.community_id;
    let community = blocking(context.pool(), move |conn| {
      Community::read(conn, community_id)
    })
    .await??;

    let mut remove = Remove::new(mod_.actor_id.to_owned(), Url::parse(&self.ap_id)?);
    remove
      .set_context(activitystreams::context())
      .set_id(generate_activity_id(RemoveType::Remove)?)
      .set_to(public())
      .set_many_ccs(vec![community.actor_id()?]);

    // Undo that fake activity
    let mut undo = Undo::new(mod_.actor_id.to_owned(), remove.into_any_base()?);
    undo
      .set_context(activitystreams::context())
      .set_id(generate_activity_id(UndoType::Undo)?)
      .set_to(public())
      .set_many_ccs(vec![community.actor_id()?]);

    send_to_community(mod_, &community, undo, context).await?;
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
      .set_many_ccs(vec![community.actor_id()?]);

    send_to_community(&creator, &community, like, context).await?;
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
      .set_many_ccs(vec![community.actor_id()?]);

    send_to_community(&creator, &community, dislike, context).await?;
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
      .set_many_ccs(vec![community.actor_id()?]);

    // Undo that fake activity
    let mut undo = Undo::new(creator.actor_id.to_owned(), like.into_any_base()?);
    undo
      .set_context(activitystreams::context())
      .set_id(generate_activity_id(UndoType::Undo)?)
      .set_to(public())
      .set_many_ccs(vec![community.actor_id()?]);

    send_to_community(&creator, &community, undo, context).await?;
    Ok(())
  }
}
