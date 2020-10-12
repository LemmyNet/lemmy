use crate::{
  activities::send::generate_activity_id,
  activity_queue::send_activity_single_dest,
  fetcher::get_or_fetch_and_upsert_actor,
  ActorType,
};
use activitystreams::{
  activity::{
    kind::{FollowType, UndoType},
    Follow,
    Undo,
  },
  base::{AnyBase, BaseExt, ExtendsExt},
};
use lemmy_db::{user::User_, DbPool};
use lemmy_utils::LemmyError;
use lemmy_websocket::LemmyContext;
use url::Url;

#[async_trait::async_trait(?Send)]
impl ActorType for User_ {
  fn actor_id_str(&self) -> String {
    self.actor_id.to_owned()
  }

  fn public_key(&self) -> Option<String> {
    self.public_key.to_owned()
  }

  fn private_key(&self) -> Option<String> {
    self.private_key.to_owned()
  }

  fn user_id(&self) -> i32 {
    self.id
  }

  /// As a given local user, send out a follow request to a remote community.
  async fn send_follow(
    &self,
    follow_actor_id: &Url,
    context: &LemmyContext,
  ) -> Result<(), LemmyError> {
    let mut follow = Follow::new(self.actor_id.to_owned(), follow_actor_id.as_str());
    follow
      .set_context(activitystreams::context())
      .set_id(generate_activity_id(FollowType::Follow)?);
    let follow_actor = get_or_fetch_and_upsert_actor(follow_actor_id, context).await?;
    let to = follow_actor.get_inbox_url()?;

    send_activity_single_dest(follow, self, to, context).await?;
    Ok(())
  }

  async fn send_unfollow(
    &self,
    follow_actor_id: &Url,
    context: &LemmyContext,
  ) -> Result<(), LemmyError> {
    let mut follow = Follow::new(self.actor_id.to_owned(), follow_actor_id.as_str());
    follow
      .set_context(activitystreams::context())
      .set_id(generate_activity_id(FollowType::Follow)?);
    let follow_actor = get_or_fetch_and_upsert_actor(follow_actor_id, context).await?;

    let to = follow_actor.get_inbox_url()?;

    // Undo that fake activity
    let mut undo = Undo::new(Url::parse(&self.actor_id)?, follow.into_any_base()?);
    undo
      .set_context(activitystreams::context())
      .set_id(generate_activity_id(UndoType::Undo)?);

    send_activity_single_dest(undo, self, to, context).await?;
    Ok(())
  }

  async fn send_accept_follow(
    &self,
    _follow: Follow,
    _context: &LemmyContext,
  ) -> Result<(), LemmyError> {
    unimplemented!()
  }

  async fn send_delete(&self, _creator: &User_, _context: &LemmyContext) -> Result<(), LemmyError> {
    unimplemented!()
  }

  async fn send_undo_delete(
    &self,
    _creator: &User_,
    _context: &LemmyContext,
  ) -> Result<(), LemmyError> {
    unimplemented!()
  }

  async fn send_remove(&self, _creator: &User_, _context: &LemmyContext) -> Result<(), LemmyError> {
    unimplemented!()
  }

  async fn send_undo_remove(
    &self,
    _creator: &User_,
    _context: &LemmyContext,
  ) -> Result<(), LemmyError> {
    unimplemented!()
  }

  async fn send_announce(
    &self,
    _activity: AnyBase,
    _sender: &User_,
    _context: &LemmyContext,
  ) -> Result<(), LemmyError> {
    unimplemented!()
  }

  async fn get_follower_inboxes(&self, _pool: &DbPool) -> Result<Vec<Url>, LemmyError> {
    unimplemented!()
  }
}
