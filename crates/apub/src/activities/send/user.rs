use crate::{
  activities::send::generate_activity_id,
  activity_queue::send_activity_single_dest,
  extensions::context::lemmy_context,
  ActorType,
};
use activitystreams::{
  activity::{
    kind::{FollowType, UndoType},
    Follow,
    Undo,
  },
  base::{AnyBase, BaseExt, ExtendsExt},
  object::ObjectExt,
};
use lemmy_db_queries::{ApubObject, DbPool, Followable};
use lemmy_db_schema::source::{
  community::{Community, CommunityFollower, CommunityFollowerForm},
  user::User_,
};
use lemmy_structs::blocking;
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

  /// As a given local user, send out a follow request to a remote community.
  async fn send_follow(
    &self,
    follow_actor_id: &Url,
    context: &LemmyContext,
  ) -> Result<(), LemmyError> {
    let follow_actor_id = follow_actor_id.to_string();
    let community = blocking(context.pool(), move |conn| {
      Community::read_from_apub_id(conn, &follow_actor_id)
    })
    .await??;

    let community_follower_form = CommunityFollowerForm {
      community_id: community.id,
      user_id: self.id,
      pending: true,
    };
    blocking(&context.pool(), move |conn| {
      CommunityFollower::follow(conn, &community_follower_form).ok()
    })
    .await?;

    let mut follow = Follow::new(self.actor_id.to_owned(), community.actor_id()?);
    follow
      .set_many_contexts(lemmy_context()?)
      .set_id(generate_activity_id(FollowType::Follow)?)
      .set_to(community.actor_id()?);

    send_activity_single_dest(follow, self, community.get_inbox_url()?, context).await?;
    Ok(())
  }

  async fn send_unfollow(
    &self,
    follow_actor_id: &Url,
    context: &LemmyContext,
  ) -> Result<(), LemmyError> {
    let follow_actor_id = follow_actor_id.to_string();
    let community = blocking(context.pool(), move |conn| {
      Community::read_from_apub_id(conn, &follow_actor_id)
    })
    .await??;

    let mut follow = Follow::new(self.actor_id.to_owned(), community.actor_id()?);
    follow
      .set_many_contexts(lemmy_context()?)
      .set_id(generate_activity_id(FollowType::Follow)?)
      .set_to(community.actor_id()?);

    // Undo that fake activity
    let mut undo = Undo::new(Url::parse(&self.actor_id)?, follow.into_any_base()?);
    undo
      .set_many_contexts(lemmy_context()?)
      .set_id(generate_activity_id(UndoType::Undo)?)
      .set_to(community.actor_id()?);

    send_activity_single_dest(undo, self, community.get_inbox_url()?, context).await?;
    Ok(())
  }

  async fn send_accept_follow(
    &self,
    _follow: Follow,
    _context: &LemmyContext,
  ) -> Result<(), LemmyError> {
    unimplemented!()
  }

  async fn send_delete(&self, _context: &LemmyContext) -> Result<(), LemmyError> {
    unimplemented!()
  }

  async fn send_undo_delete(&self, _context: &LemmyContext) -> Result<(), LemmyError> {
    unimplemented!()
  }

  async fn send_remove(&self, _context: &LemmyContext) -> Result<(), LemmyError> {
    unimplemented!()
  }

  async fn send_undo_remove(&self, _context: &LemmyContext) -> Result<(), LemmyError> {
    unimplemented!()
  }

  async fn send_announce(
    &self,
    _activity: AnyBase,
    _context: &LemmyContext,
  ) -> Result<(), LemmyError> {
    unimplemented!()
  }

  async fn get_follower_inboxes(&self, _pool: &DbPool) -> Result<Vec<Url>, LemmyError> {
    unimplemented!()
  }
}
