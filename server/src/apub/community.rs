use crate::{
  apub::{
    activities::generate_activity_id,
    activity_queue::send_activity,
    check_actor_domain,
    create_apub_response,
    create_apub_tombstone_response,
    create_tombstone,
    extensions::group_extensions::GroupExtension,
    fetcher::{get_or_fetch_and_upsert_actor, get_or_fetch_and_upsert_user},
    insert_activity,
    ActorType,
    FromApub,
    GroupExt,
    ToApub,
  },
  DbPool,
  LemmyContext,
};
use activitystreams::{
  activity::{
    kind::{AcceptType, AnnounceType, DeleteType, LikeType, RemoveType, UndoType},
    Accept,
    Announce,
    Delete,
    Follow,
    Remove,
    Undo,
  },
  actor::{kind::GroupType, ApActor, Endpoints, Group},
  base::{AnyBase, BaseExt},
  collection::{OrderedCollection, UnorderedCollection},
  object::{Image, Tombstone},
  prelude::*,
  public,
};
use activitystreams_ext::Ext2;
use actix_web::{body::Body, web, HttpResponse};
use anyhow::Context;
use itertools::Itertools;
use lemmy_api_structs::blocking;
use lemmy_db::{
  community::{Community, CommunityForm},
  community_view::{CommunityFollowerView, CommunityModeratorView},
  naive_now,
  post::Post,
  user::User_,
};
use lemmy_utils::{
  apub::get_apub_protocol_string,
  location_info,
  utils::{check_slurs, check_slurs_opt, convert_datetime},
  LemmyError,
};
use serde::Deserialize;
use url::Url;

#[derive(Deserialize)]
pub struct CommunityQuery {
  community_name: String,
}

#[async_trait::async_trait(?Send)]
impl ToApub for Community {
  type Response = GroupExt;

  // Turn a Lemmy Community into an ActivityPub group that can be sent out over the network.
  async fn to_apub(&self, pool: &DbPool) -> Result<GroupExt, LemmyError> {
    // The attributed to, is an ordered vector with the creator actor_ids first,
    // then the rest of the moderators
    // TODO Technically the instance admins can mod the community, but lets
    // ignore that for now
    let id = self.id;
    let moderators = blocking(pool, move |conn| {
      CommunityModeratorView::for_community(&conn, id)
    })
    .await??;
    let moderators: Vec<String> = moderators.into_iter().map(|m| m.user_actor_id).collect();

    let mut group = Group::new();
    group
      .set_context(activitystreams::context())
      .set_id(Url::parse(&self.actor_id)?)
      .set_name(self.name.to_owned())
      .set_published(convert_datetime(self.published))
      .set_many_attributed_tos(moderators);

    if let Some(u) = self.updated.to_owned() {
      group.set_updated(convert_datetime(u));
    }
    if let Some(d) = self.description.to_owned() {
      // TODO: this should be html, also add source field with raw markdown
      //       -> same for post.content and others
      group.set_content(d);
    }

    let mut ap_actor = ApActor::new(self.get_inbox_url()?, group);
    ap_actor
      .set_preferred_username(self.title.to_owned())
      .set_outbox(self.get_outbox_url()?)
      .set_followers(self.get_followers_url()?)
      .set_following(self.get_following_url().parse()?)
      .set_liked(self.get_liked_url().parse()?)
      .set_endpoints(Endpoints {
        shared_inbox: Some(self.get_shared_inbox_url()?),
        ..Default::default()
      });

    let nsfw = self.nsfw;
    let category_id = self.category_id;
    let group_extension = blocking(pool, move |conn| {
      GroupExtension::new(conn, category_id, nsfw)
    })
    .await??;

    Ok(Ext2::new(
      ap_actor,
      group_extension,
      self.get_public_key_ext()?,
    ))
  }

  fn to_tombstone(&self) -> Result<Tombstone, LemmyError> {
    create_tombstone(self.deleted, &self.actor_id, self.updated, GroupType::Group)
  }
}

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
    let actor = get_or_fetch_and_upsert_actor(actor_uri, context).await?;

    let mut accept = Accept::new(self.actor_id.to_owned(), follow.into_any_base()?);
    let to = actor.get_inbox_url()?;
    accept
      .set_context(activitystreams::context())
      .set_id(generate_activity_id(AcceptType::Accept)?)
      .set_to(to.clone());

    insert_activity(self.creator_id, accept.clone(), true, context.pool()).await?;

    send_activity(context.activity_queue(), accept, self, vec![to])?;
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

    insert_activity(self.creator_id, delete.clone(), true, context.pool()).await?;

    let inboxes = self.get_follower_inboxes(context.pool()).await?;

    // Note: For an accept, since it was automatic, no one pushed a button,
    // the community was the actor.
    // But for delete, the creator is the actor, and does the signing
    send_activity(context.activity_queue(), delete, creator, inboxes)?;
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

    insert_activity(self.creator_id, undo.clone(), true, context.pool()).await?;

    let inboxes = self.get_follower_inboxes(context.pool()).await?;

    // Note: For an accept, since it was automatic, no one pushed a button,
    // the community was the actor.
    // But for delete, the creator is the actor, and does the signing
    send_activity(context.activity_queue(), undo, creator, inboxes)?;
    Ok(())
  }

  async fn send_remove(&self, mod_: &User_, context: &LemmyContext) -> Result<(), LemmyError> {
    let group = self.to_apub(context.pool()).await?;

    let mut remove = Remove::new(mod_.actor_id.to_owned(), group.into_any_base()?);
    remove
      .set_context(activitystreams::context())
      .set_id(generate_activity_id(RemoveType::Remove)?)
      .set_to(public())
      .set_many_ccs(vec![self.get_followers_url()?]);

    insert_activity(mod_.id, remove.clone(), true, context.pool()).await?;

    let inboxes = self.get_follower_inboxes(context.pool()).await?;

    // Note: For an accept, since it was automatic, no one pushed a button,
    // the community was the actor.
    // But for delete, the creator is the actor, and does the signing
    send_activity(context.activity_queue(), remove, mod_, inboxes)?;
    Ok(())
  }

  async fn send_undo_remove(&self, mod_: &User_, context: &LemmyContext) -> Result<(), LemmyError> {
    let group = self.to_apub(context.pool()).await?;

    let mut remove = Remove::new(mod_.actor_id.to_owned(), group.into_any_base()?);
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

    insert_activity(mod_.id, undo.clone(), true, context.pool()).await?;

    let inboxes = self.get_follower_inboxes(context.pool()).await?;

    // Note: For an accept, since it was automatic, no one pushed a button,
    // the community was the actor.
    // But for remove , the creator is the actor, and does the signing
    send_activity(context.activity_queue(), undo, mod_, inboxes)?;
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
          get_apub_protocol_string(),
          domain,
          port,
        ))?)
      })
      .filter_map(Result::ok)
      .unique()
      .collect();

    Ok(inboxes)
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

  fn user_id(&self) -> i32 {
    self.creator_id
  }
}

#[async_trait::async_trait(?Send)]
impl FromApub for CommunityForm {
  type ApubType = GroupExt;

  /// Parse an ActivityPub group received from another instance into a Lemmy community.
  async fn from_apub(
    group: &GroupExt,
    context: &LemmyContext,
    expected_domain: Option<Url>,
  ) -> Result<Self, LemmyError> {
    let creator_and_moderator_uris = group.inner.attributed_to().context(location_info!())?;
    let creator_uri = creator_and_moderator_uris
      .as_many()
      .context(location_info!())?
      .iter()
      .next()
      .context(location_info!())?
      .as_xsd_any_uri()
      .context(location_info!())?;

    let creator = get_or_fetch_and_upsert_user(creator_uri, context).await?;
    let name = group
      .inner
      .name()
      .context(location_info!())?
      .as_one()
      .context(location_info!())?
      .as_xsd_string()
      .context(location_info!())?
      .to_string();
    let title = group
      .inner
      .preferred_username()
      .context(location_info!())?
      .to_string();
    // TODO: should be parsed as html and tags like <script> removed (or use markdown source)
    //       -> same for post.content etc
    let description = group
      .inner
      .content()
      .map(|s| s.as_single_xsd_string())
      .flatten()
      .map(|s| s.to_string());
    check_slurs(&name)?;
    check_slurs(&title)?;
    check_slurs_opt(&description)?;

    let icon = match group.icon() {
      Some(any_image) => Some(
        Image::from_any_base(any_image.as_one().context(location_info!())?.clone())
          .context(location_info!())?
          .context(location_info!())?
          .url()
          .context(location_info!())?
          .as_single_xsd_any_uri()
          .map(|u| u.to_string()),
      ),
      None => None,
    };

    let banner = match group.image() {
      Some(any_image) => Some(
        Image::from_any_base(any_image.as_one().context(location_info!())?.clone())
          .context(location_info!())?
          .context(location_info!())?
          .url()
          .context(location_info!())?
          .as_single_xsd_any_uri()
          .map(|u| u.to_string()),
      ),
      None => None,
    };

    Ok(CommunityForm {
      name,
      title,
      description,
      category_id: group.ext_one.category.identifier.parse::<i32>()?,
      creator_id: creator.id,
      removed: None,
      published: group.inner.published().map(|u| u.to_owned().naive_local()),
      updated: group.inner.updated().map(|u| u.to_owned().naive_local()),
      deleted: None,
      nsfw: group.ext_one.sensitive,
      actor_id: Some(check_actor_domain(group, expected_domain)?),
      local: false,
      private_key: None,
      public_key: Some(group.ext_two.to_owned().public_key.public_key_pem),
      last_refreshed_at: Some(naive_now()),
      icon,
      banner,
    })
  }
}

/// Return the community json over HTTP.
pub async fn get_apub_community_http(
  info: web::Path<CommunityQuery>,
  context: web::Data<LemmyContext>,
) -> Result<HttpResponse<Body>, LemmyError> {
  let community = blocking(context.pool(), move |conn| {
    Community::read_from_name(conn, &info.community_name)
  })
  .await??;

  if !community.deleted {
    let apub = community.to_apub(context.pool()).await?;

    Ok(create_apub_response(&apub))
  } else {
    Ok(create_apub_tombstone_response(&community.to_tombstone()?))
  }
}

/// Returns an empty followers collection, only populating the size (for privacy).
pub async fn get_apub_community_followers(
  info: web::Path<CommunityQuery>,
  context: web::Data<LemmyContext>,
) -> Result<HttpResponse<Body>, LemmyError> {
  let community = blocking(context.pool(), move |conn| {
    Community::read_from_name(&conn, &info.community_name)
  })
  .await??;

  let community_id = community.id;
  let community_followers = blocking(context.pool(), move |conn| {
    CommunityFollowerView::for_community(&conn, community_id)
  })
  .await??;

  let mut collection = UnorderedCollection::new();
  collection
    .set_context(activitystreams::context())
    .set_id(community.get_followers_url()?)
    .set_total_items(community_followers.len() as u64);
  Ok(create_apub_response(&collection))
}

pub async fn get_apub_community_outbox(
  info: web::Path<CommunityQuery>,
  context: web::Data<LemmyContext>,
) -> Result<HttpResponse<Body>, LemmyError> {
  let community = blocking(context.pool(), move |conn| {
    Community::read_from_name(&conn, &info.community_name)
  })
  .await??;

  let community_id = community.id;
  let posts = blocking(context.pool(), move |conn| {
    Post::list_for_community(conn, community_id)
  })
  .await??;

  let mut pages: Vec<AnyBase> = vec![];
  for p in posts {
    pages.push(p.to_apub(context.pool()).await?.into_any_base()?);
  }

  let len = pages.len();
  let mut collection = OrderedCollection::new();
  collection
    .set_many_items(pages)
    .set_context(activitystreams::context())
    .set_id(community.get_outbox_url()?)
    .set_total_items(len as u64);
  Ok(create_apub_response(&collection))
}

pub async fn do_announce(
  activity: AnyBase,
  community: &Community,
  sender: &User_,
  context: &LemmyContext,
) -> Result<(), LemmyError> {
  let mut announce = Announce::new(community.actor_id.to_owned(), activity);
  announce
    .set_context(activitystreams::context())
    .set_id(generate_activity_id(AnnounceType::Announce)?)
    .set_to(public())
    .set_many_ccs(vec![community.get_followers_url()?]);

  insert_activity(community.creator_id, announce.clone(), true, context.pool()).await?;

  let mut to: Vec<Url> = community.get_follower_inboxes(context.pool()).await?;

  // dont send to the local instance, nor to the instance where the activity originally came from,
  // because that would result in a database error (same data inserted twice)
  // this seems to be the "easiest" stable alternative for remove_item()
  let sender_shared_inbox = sender.get_shared_inbox_url()?;
  to.retain(|x| x != &sender_shared_inbox);
  let community_shared_inbox = community.get_shared_inbox_url()?;
  to.retain(|x| x != &community_shared_inbox);

  send_activity(context.activity_queue(), announce, community, to)?;

  Ok(())
}
