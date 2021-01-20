use crate::{
  activities::send::generate_activity_id,
  activity_queue::{send_activity_single_dest, send_to_community_followers},
  check_is_apub_id_valid,
  extensions::context::lemmy_context,
  fetcher::user::get_or_fetch_and_upsert_user,
  ActorType,
};
use activitystreams::{
  activity::{
    kind::{AcceptType, AnnounceType, DeleteType, LikeType, RemoveType, UndoType},
    Accept,
    ActorAndObjectRefExt,
    Announce,
    Delete,
    Follow,
    Remove,
    Undo,
  },
  base::{AnyBase, BaseExt, ExtendsExt},
  object::ObjectExt,
  public,
};
use anyhow::Context;
use itertools::Itertools;
use lemmy_db_queries::DbPool;
use lemmy_db_schema::source::community::Community;
use lemmy_db_views_actor::community_follower_view::CommunityFollowerView;
use lemmy_structs::blocking;
use lemmy_utils::{location_info, settings::Settings, LemmyError};
use lemmy_websocket::LemmyContext;
use url::Url;

#[async_trait::async_trait(?Send)]
impl ActorType for Community {
  fn actor_id_str(&self) -> String {
    self.actor_id.to_owned()
  }

  fn public_key(&self) -> Option<String> {
    self.public_key.to_owned()
  }
  fn private_key(&self) -> Option<String> {
    self.private_key.to_owned()
  }

  async fn send_follow(
    &self,
    _follow_actor_id: &Url,
    _context: &LemmyContext,
  ) -> Result<(), LemmyError> {
    unimplemented!()
  }

  async fn send_unfollow(
    &self,
    _follow_actor_id: &Url,
    _context: &LemmyContext,
  ) -> Result<(), LemmyError> {
    unimplemented!()
  }

  /// As a local community, accept the follow request from a remote user.
  async fn send_accept_follow(
    &self,
    follow: Follow,
    context: &LemmyContext,
  ) -> Result<(), LemmyError> {
    let actor_uri = follow
      .actor()?
      .as_single_xsd_any_uri()
      .context(location_info!())?;
    let user = get_or_fetch_and_upsert_user(actor_uri, context, &mut 0).await?;

    let mut accept = Accept::new(self.actor_id.to_owned(), follow.into_any_base()?);
    accept
      .set_many_contexts(lemmy_context()?)
      .set_id(generate_activity_id(AcceptType::Accept)?)
      .set_to(user.actor_id()?);

    send_activity_single_dest(accept, self, user.get_inbox_url()?, context).await?;
    Ok(())
  }

  /// If the creator of a community deletes the community, send this to all followers.
  async fn send_delete(&self, context: &LemmyContext) -> Result<(), LemmyError> {
    let mut delete = Delete::new(self.actor_id()?, self.actor_id()?);
    delete
      .set_many_contexts(lemmy_context()?)
      .set_id(generate_activity_id(DeleteType::Delete)?)
      .set_to(public())
      .set_many_ccs(vec![self.get_followers_url()?]);

    send_to_community_followers(delete, self, context).await?;
    Ok(())
  }

  /// If the creator of a community reverts the deletion of a community, send this to all followers.
  async fn send_undo_delete(&self, context: &LemmyContext) -> Result<(), LemmyError> {
    let mut delete = Delete::new(self.actor_id()?, self.actor_id()?);
    delete
      .set_many_contexts(lemmy_context()?)
      .set_id(generate_activity_id(DeleteType::Delete)?)
      .set_to(public())
      .set_many_ccs(vec![self.get_followers_url()?]);

    let mut undo = Undo::new(self.actor_id()?, delete.into_any_base()?);
    undo
      .set_many_contexts(lemmy_context()?)
      .set_id(generate_activity_id(UndoType::Undo)?)
      .set_to(public())
      .set_many_ccs(vec![self.get_followers_url()?]);

    send_to_community_followers(undo, self, context).await?;
    Ok(())
  }

  /// If an admin removes a community, send this to all followers.
  async fn send_remove(&self, context: &LemmyContext) -> Result<(), LemmyError> {
    let mut remove = Remove::new(self.actor_id()?, self.actor_id()?);
    remove
      .set_many_contexts(lemmy_context()?)
      .set_id(generate_activity_id(RemoveType::Remove)?)
      .set_to(public())
      .set_many_ccs(vec![self.get_followers_url()?]);

    send_to_community_followers(remove, self, context).await?;
    Ok(())
  }

  /// If an admin reverts the removal of a community, send this to all followers.
  async fn send_undo_remove(&self, context: &LemmyContext) -> Result<(), LemmyError> {
    let mut remove = Remove::new(self.actor_id()?, self.actor_id()?);
    remove
      .set_many_contexts(lemmy_context()?)
      .set_id(generate_activity_id(RemoveType::Remove)?)
      .set_to(public())
      .set_many_ccs(vec![self.get_followers_url()?]);

    // Undo that fake activity
    let mut undo = Undo::new(self.actor_id()?, remove.into_any_base()?);
    undo
      .set_many_contexts(lemmy_context()?)
      .set_id(generate_activity_id(LikeType::Like)?)
      .set_to(public())
      .set_many_ccs(vec![self.get_followers_url()?]);

    send_to_community_followers(undo, self, context).await?;
    Ok(())
  }

  /// Wraps an activity sent to the community in an announce, and then sends the announce to all
  /// community followers.
  async fn send_announce(
    &self,
    activity: AnyBase,
    context: &LemmyContext,
  ) -> Result<(), LemmyError> {
    let mut announce = Announce::new(self.actor_id.to_owned(), activity);
    announce
      .set_many_contexts(lemmy_context()?)
      .set_id(generate_activity_id(AnnounceType::Announce)?)
      .set_to(public())
      .set_many_ccs(vec![self.get_followers_url()?]);

    send_to_community_followers(announce, self, context).await?;

    Ok(())
  }

  /// For a given community, returns the inboxes of all followers.
  ///
  /// TODO: this function is very badly implemented, we should just store shared_inbox_url in
  ///       CommunityFollowerView
  async fn get_follower_inboxes(&self, pool: &DbPool) -> Result<Vec<Url>, LemmyError> {
    let id = self.id;

    let inboxes = blocking(pool, move |conn| {
      CommunityFollowerView::for_community(conn, id)
    })
    .await??;
    let inboxes = inboxes
      .into_iter()
      .filter(|i| !i.follower.local)
      .map(|u| -> Result<Url, LemmyError> {
        let url = Url::parse(&u.follower.actor_id)?;
        let domain = url.domain().context(location_info!())?;
        let port = if let Some(port) = url.port() {
          format!(":{}", port)
        } else {
          "".to_string()
        };
        Ok(Url::parse(&format!(
          "{}://{}{}/inbox",
          Settings::get().get_protocol_string(),
          domain,
          port,
        ))?)
      })
      .filter_map(Result::ok)
      // Don't send to blocked instances
      .filter(|inbox| check_is_apub_id_valid(inbox).is_ok())
      .unique()
      .collect();

    Ok(inboxes)
  }
}
