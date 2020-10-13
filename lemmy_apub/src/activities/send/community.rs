use crate::{
  activities::send::generate_activity_id,
  activity_queue::{send_activity_single_dest, send_to_community_followers},
  check_is_apub_id_valid,
  fetcher::get_or_fetch_and_upsert_user,
  ActorType,
  ToApub,
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
use lemmy_db::{community::Community, community_view::CommunityFollowerView, user::User_, DbPool};
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

  fn user_id(&self) -> i32 {
    self.creator_id
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
    let user = get_or_fetch_and_upsert_user(actor_uri, context).await?;

    let mut accept = Accept::new(self.actor_id.to_owned(), follow.into_any_base()?);
    accept
      .set_context(activitystreams::context())
      .set_id(generate_activity_id(AcceptType::Accept)?)
      .set_to(user.actor_id()?);

    send_activity_single_dest(accept, self, user.get_inbox_url()?, context).await?;
    Ok(())
  }

  async fn send_delete(&self, creator: &User_, context: &LemmyContext) -> Result<(), LemmyError> {
    let group = self.to_apub(context.pool()).await?;

    let mut delete = Delete::new(creator.actor_id.to_owned(), group.into_any_base()?);
    delete
      .set_context(activitystreams::context())
      .set_id(generate_activity_id(DeleteType::Delete)?)
      .set_to(public())
      .set_many_ccs(vec![self.get_followers_url()?]);

    send_to_community_followers(delete, self, context, None).await?;
    Ok(())
  }

  async fn send_undo_delete(
    &self,
    creator: &User_,
    context: &LemmyContext,
  ) -> Result<(), LemmyError> {
    let group = self.to_apub(context.pool()).await?;

    let mut delete = Delete::new(creator.actor_id.to_owned(), group.into_any_base()?);
    delete
      .set_context(activitystreams::context())
      .set_id(generate_activity_id(DeleteType::Delete)?)
      .set_to(public())
      .set_many_ccs(vec![self.get_followers_url()?]);

    let mut undo = Undo::new(creator.actor_id.to_owned(), delete.into_any_base()?);
    undo
      .set_context(activitystreams::context())
      .set_id(generate_activity_id(UndoType::Undo)?)
      .set_to(public())
      .set_many_ccs(vec![self.get_followers_url()?]);

    send_to_community_followers(undo, self, context, None).await?;
    Ok(())
  }

  async fn send_remove(&self, mod_: &User_, context: &LemmyContext) -> Result<(), LemmyError> {
    let mut remove = Remove::new(mod_.actor_id.to_owned(), self.actor_id()?);
    remove
      .set_context(activitystreams::context())
      .set_id(generate_activity_id(RemoveType::Remove)?)
      .set_to(public())
      .set_many_ccs(vec![self.get_followers_url()?]);

    send_to_community_followers(remove, self, context, None).await?;
    Ok(())
  }

  async fn send_undo_remove(&self, mod_: &User_, context: &LemmyContext) -> Result<(), LemmyError> {
    let mut remove = Remove::new(mod_.actor_id.to_owned(), self.actor_id()?);
    remove
      .set_context(activitystreams::context())
      .set_id(generate_activity_id(RemoveType::Remove)?)
      .set_to(public())
      .set_many_ccs(vec![self.get_followers_url()?]);

    // Undo that fake activity
    let mut undo = Undo::new(mod_.actor_id.to_owned(), remove.into_any_base()?);
    undo
      .set_context(activitystreams::context())
      .set_id(generate_activity_id(LikeType::Like)?)
      .set_to(public())
      .set_many_ccs(vec![self.get_followers_url()?]);

    send_to_community_followers(undo, self, context, None).await?;
    Ok(())
  }

  async fn send_announce(
    &self,
    activity: AnyBase,
    sender: &User_,
    context: &LemmyContext,
  ) -> Result<(), LemmyError> {
    let mut announce = Announce::new(self.actor_id.to_owned(), activity);
    announce
      .set_context(activitystreams::context())
      .set_id(generate_activity_id(AnnounceType::Announce)?)
      .set_to(public())
      .set_many_ccs(vec![self.get_followers_url()?]);

    send_to_community_followers(
      announce,
      self,
      context,
      Some(sender.get_shared_inbox_url()?),
    )
    .await?;

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
      .filter(|i| !i.user_local)
      .map(|u| -> Result<Url, LemmyError> {
        let url = Url::parse(&u.user_actor_id)?;
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
