use crate::{
  activities::send::generate_activity_id,
  activity_queue::{send_activity_single_dest, send_to_community, send_to_community_followers},
  check_is_apub_id_valid,
  extensions::context::lemmy_context,
  fetcher::person::get_or_fetch_and_upsert_person,
  generate_moderators_url,
  insert_activity,
  ActorType,
  CommunityType,
};
use activitystreams::{
  activity::{
    kind::{AcceptType, AddType, AnnounceType, DeleteType, LikeType, RemoveType, UndoType},
    Accept,
    ActorAndObjectRefExt,
    Add,
    Announce,
    Delete,
    Follow,
    OptTargetRefExt,
    Remove,
    Undo,
  },
  base::{AnyBase, BaseExt, ExtendsExt},
  object::ObjectExt,
  public,
};
use anyhow::Context;
use itertools::Itertools;
use lemmy_api_common::blocking;
use lemmy_db_queries::DbPool;
use lemmy_db_schema::source::{community::Community, person::Person};
use lemmy_db_views_actor::community_follower_view::CommunityFollowerView;
use lemmy_utils::{location_info, settings::structs::Settings, LemmyError};
use lemmy_websocket::LemmyContext;
use url::Url;

#[async_trait::async_trait(?Send)]
impl ActorType for Community {
  fn is_local(&self) -> bool {
    self.local
  }
  fn actor_id(&self) -> Url {
    self.actor_id.to_owned().into_inner()
  }
  fn public_key(&self) -> Option<String> {
    self.public_key.to_owned()
  }
  fn private_key(&self) -> Option<String> {
    self.private_key.to_owned()
  }

  fn get_shared_inbox_or_inbox_url(&self) -> Url {
    self
      .shared_inbox_url
      .clone()
      .unwrap_or_else(|| self.inbox_url.to_owned())
      .into()
  }
}

#[async_trait::async_trait(?Send)]
impl CommunityType for Community {
  /// As a local community, accept the follow request from a remote person.
  async fn send_accept_follow(
    &self,
    follow: Follow,
    context: &LemmyContext,
  ) -> Result<(), LemmyError> {
    let actor_uri = follow
      .actor()?
      .as_single_xsd_any_uri()
      .context(location_info!())?;
    let person = get_or_fetch_and_upsert_person(actor_uri, context, &mut 0).await?;

    let mut accept = Accept::new(
      self.actor_id.to_owned().into_inner(),
      follow.into_any_base()?,
    );
    accept
      .set_many_contexts(lemmy_context()?)
      .set_id(generate_activity_id(AcceptType::Accept)?)
      .set_to(person.actor_id());

    send_activity_single_dest(accept, self, person.inbox_url.into(), context).await?;
    Ok(())
  }

  /// If the creator of a community deletes the community, send this to all followers.
  async fn send_delete(&self, context: &LemmyContext) -> Result<(), LemmyError> {
    let mut delete = Delete::new(self.actor_id(), self.actor_id());
    delete
      .set_many_contexts(lemmy_context()?)
      .set_id(generate_activity_id(DeleteType::Delete)?)
      .set_to(public())
      .set_many_ccs(vec![self.followers_url.clone().into_inner()]);

    send_to_community_followers(delete, self, context).await?;
    Ok(())
  }

  /// If the creator of a community reverts the deletion of a community, send this to all followers.
  async fn send_undo_delete(&self, context: &LemmyContext) -> Result<(), LemmyError> {
    let mut delete = Delete::new(self.actor_id(), self.actor_id());
    delete
      .set_many_contexts(lemmy_context()?)
      .set_id(generate_activity_id(DeleteType::Delete)?)
      .set_to(public())
      .set_many_ccs(vec![self.followers_url.clone().into_inner()]);

    let mut undo = Undo::new(self.actor_id(), delete.into_any_base()?);
    undo
      .set_many_contexts(lemmy_context()?)
      .set_id(generate_activity_id(UndoType::Undo)?)
      .set_to(public())
      .set_many_ccs(vec![self.followers_url.clone().into_inner()]);

    send_to_community_followers(undo, self, context).await?;
    Ok(())
  }

  /// If an admin removes a community, send this to all followers.
  async fn send_remove(&self, context: &LemmyContext) -> Result<(), LemmyError> {
    let mut remove = Remove::new(self.actor_id(), self.actor_id());
    remove
      .set_many_contexts(lemmy_context()?)
      .set_id(generate_activity_id(RemoveType::Remove)?)
      .set_to(public())
      .set_many_ccs(vec![self.followers_url.clone().into_inner()]);

    send_to_community_followers(remove, self, context).await?;
    Ok(())
  }

  /// If an admin reverts the removal of a community, send this to all followers.
  async fn send_undo_remove(&self, context: &LemmyContext) -> Result<(), LemmyError> {
    let mut remove = Remove::new(self.actor_id(), self.actor_id());
    remove
      .set_many_contexts(lemmy_context()?)
      .set_id(generate_activity_id(RemoveType::Remove)?)
      .set_to(public())
      .set_many_ccs(vec![self.followers_url.clone().into_inner()]);

    // Undo that fake activity
    let mut undo = Undo::new(self.actor_id(), remove.into_any_base()?);
    undo
      .set_many_contexts(lemmy_context()?)
      .set_id(generate_activity_id(LikeType::Like)?)
      .set_to(public())
      .set_many_ccs(vec![self.followers_url.clone().into_inner()]);

    send_to_community_followers(undo, self, context).await?;
    Ok(())
  }

  /// Wraps an activity sent to the community in an announce, and then sends the announce to all
  /// community followers.
  ///
  /// If we are announcing a local activity, it hasn't been stored in the database yet, and we need
  /// to do it here, so that it can be fetched by ID. Remote activities are inserted into DB in the
  /// inbox.
  async fn send_announce(
    &self,
    activity: AnyBase,
    context: &LemmyContext,
  ) -> Result<(), LemmyError> {
    let inner_id = activity.id().context(location_info!())?;
    if inner_id.domain() == Some(&Settings::get().get_hostname_without_port()?) {
      insert_activity(inner_id, activity.clone(), true, false, context.pool()).await?;
    }

    let mut announce = Announce::new(self.actor_id.to_owned().into_inner(), activity);
    announce
      .set_many_contexts(lemmy_context()?)
      .set_id(generate_activity_id(AnnounceType::Announce)?)
      .set_to(public())
      .set_many_ccs(vec![self.followers_url.clone().into_inner()]);

    send_to_community_followers(announce, self, context).await?;

    Ok(())
  }

  /// For a given community, returns the inboxes of all followers.
  async fn get_follower_inboxes(&self, pool: &DbPool) -> Result<Vec<Url>, LemmyError> {
    let id = self.id;

    let follows = blocking(pool, move |conn| {
      CommunityFollowerView::for_community(conn, id)
    })
    .await??;
    let inboxes = follows
      .into_iter()
      .filter(|f| !f.follower.local)
      .map(|f| f.follower.shared_inbox_url.unwrap_or(f.follower.inbox_url))
      .map(|i| i.into_inner())
      .unique()
      // Don't send to blocked instances
      .filter(|inbox| check_is_apub_id_valid(inbox).is_ok())
      .collect();

    Ok(inboxes)
  }

  async fn send_add_mod(
    &self,
    actor: &Person,
    added_mod: Person,
    context: &LemmyContext,
  ) -> Result<(), LemmyError> {
    let mut add = Add::new(
      actor.actor_id.clone().into_inner(),
      added_mod.actor_id.into_inner(),
    );
    add
      .set_many_contexts(lemmy_context()?)
      .set_id(generate_activity_id(AddType::Add)?)
      .set_to(public())
      .set_many_ccs(vec![self.actor_id()])
      .set_target(generate_moderators_url(&self.actor_id)?.into_inner());

    send_to_community(add, actor, self, context).await?;
    Ok(())
  }

  async fn send_remove_mod(
    &self,
    actor: &Person,
    removed_mod: Person,
    context: &LemmyContext,
  ) -> Result<(), LemmyError> {
    let mut remove = Remove::new(
      actor.actor_id.clone().into_inner(),
      removed_mod.actor_id.into_inner(),
    );
    remove
      .set_many_contexts(lemmy_context()?)
      .set_id(generate_activity_id(RemoveType::Remove)?)
      .set_to(public())
      .set_many_ccs(vec![self.actor_id()])
      .set_target(generate_moderators_url(&self.actor_id)?.into_inner());

    send_to_community(remove, &actor, self, context).await?;
    Ok(())
  }
}
