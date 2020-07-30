use crate::{
  apub::{
    activities::{generate_activity_id, send_activity},
    create_apub_response,
    create_apub_tombstone_response,
    create_tombstone,
    extensions::group_extensions::GroupExtension,
    fetcher::get_or_fetch_and_upsert_user,
    get_shared_inbox,
    insert_activity,
    ActorType,
    FromApub,
    GroupExt,
    ToApub,
  },
  blocking,
  routes::DbPoolParam,
  DbPool,
  LemmyError,
};
use activitystreams_ext::Ext2;
use activitystreams_new::{
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
  context,
  object::Tombstone,
  prelude::*,
  public,
};
use actix_web::{body::Body, client::Client, web, HttpResponse};
use itertools::Itertools;
use lemmy_db::{
  community::{Community, CommunityForm},
  community_view::{CommunityFollowerView, CommunityModeratorView},
  naive_now,
  post::Post,
  user::User_,
};
use lemmy_utils::convert_datetime;
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
      .set_context(context())
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
      .set_followers(self.get_followers_url().parse()?)
      .set_following(self.get_following_url().parse()?)
      .set_liked(self.get_liked_url().parse()?)
      .set_endpoints(Endpoints {
        shared_inbox: Some(self.get_shared_inbox_url().parse()?),
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
      self.get_public_key_ext(),
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

  fn public_key(&self) -> String {
    self.public_key.to_owned().unwrap()
  }
  fn private_key(&self) -> String {
    self.private_key.to_owned().unwrap()
  }

  /// As a local community, accept the follow request from a remote user.
  async fn send_accept_follow(
    &self,
    follow: Follow,
    client: &Client,
    pool: &DbPool,
  ) -> Result<(), LemmyError> {
    let actor_uri = follow.actor()?.as_single_xsd_any_uri().unwrap().to_string();

    let mut accept = Accept::new(self.actor_id.to_owned(), follow.into_any_base()?);
    let to = format!("{}/inbox", actor_uri);
    accept
      .set_context(context())
      .set_id(generate_activity_id(AcceptType::Accept)?)
      .set_to(to.clone());

    insert_activity(self.creator_id, accept.clone(), true, pool).await?;

    send_activity(client, &accept.into_any_base()?, self, vec![to]).await?;
    Ok(())
  }

  async fn send_delete(
    &self,
    creator: &User_,
    client: &Client,
    pool: &DbPool,
  ) -> Result<(), LemmyError> {
    let group = self.to_apub(pool).await?;

    let mut delete = Delete::new(creator.actor_id.to_owned(), group.into_any_base()?);
    delete
      .set_context(context())
      .set_id(generate_activity_id(DeleteType::Delete)?)
      .set_to(public())
      .set_many_ccs(vec![self.get_followers_url()]);

    insert_activity(self.creator_id, delete.clone(), true, pool).await?;

    let inboxes = self.get_follower_inboxes(pool).await?;

    // Note: For an accept, since it was automatic, no one pushed a button,
    // the community was the actor.
    // But for delete, the creator is the actor, and does the signing
    send_activity(client, &delete.into_any_base()?, creator, inboxes).await?;
    Ok(())
  }

  async fn send_undo_delete(
    &self,
    creator: &User_,
    client: &Client,
    pool: &DbPool,
  ) -> Result<(), LemmyError> {
    let group = self.to_apub(pool).await?;

    let mut delete = Delete::new(creator.actor_id.to_owned(), group.into_any_base()?);
    delete
      .set_context(context())
      .set_id(generate_activity_id(DeleteType::Delete)?)
      .set_to(public())
      .set_many_ccs(vec![self.get_followers_url()]);

    // TODO
    // Undo that fake activity
    let mut undo = Undo::new(creator.actor_id.to_owned(), delete.into_any_base()?);
    undo
      .set_context(context())
      .set_id(generate_activity_id(UndoType::Undo)?)
      .set_to(public())
      .set_many_ccs(vec![self.get_followers_url()]);

    insert_activity(self.creator_id, undo.clone(), true, pool).await?;

    let inboxes = self.get_follower_inboxes(pool).await?;

    // Note: For an accept, since it was automatic, no one pushed a button,
    // the community was the actor.
    // But for delete, the creator is the actor, and does the signing
    send_activity(client, &undo.into_any_base()?, creator, inboxes).await?;
    Ok(())
  }

  async fn send_remove(
    &self,
    mod_: &User_,
    client: &Client,
    pool: &DbPool,
  ) -> Result<(), LemmyError> {
    let group = self.to_apub(pool).await?;

    let mut remove = Remove::new(mod_.actor_id.to_owned(), group.into_any_base()?);
    remove
      .set_context(context())
      .set_id(generate_activity_id(RemoveType::Remove)?)
      .set_to(public())
      .set_many_ccs(vec![self.get_followers_url()]);

    insert_activity(mod_.id, remove.clone(), true, pool).await?;

    let inboxes = self.get_follower_inboxes(pool).await?;

    // Note: For an accept, since it was automatic, no one pushed a button,
    // the community was the actor.
    // But for delete, the creator is the actor, and does the signing
    send_activity(client, &remove.into_any_base()?, mod_, inboxes).await?;
    Ok(())
  }

  async fn send_undo_remove(
    &self,
    mod_: &User_,
    client: &Client,
    pool: &DbPool,
  ) -> Result<(), LemmyError> {
    let group = self.to_apub(pool).await?;

    let mut remove = Remove::new(mod_.actor_id.to_owned(), group.into_any_base()?);
    remove
      .set_context(context())
      .set_id(generate_activity_id(RemoveType::Remove)?)
      .set_to(public())
      .set_many_ccs(vec![self.get_followers_url()]);

    // Undo that fake activity
    let mut undo = Undo::new(mod_.actor_id.to_owned(), remove.into_any_base()?);
    undo
      .set_context(context())
      .set_id(generate_activity_id(LikeType::Like)?)
      .set_to(public())
      .set_many_ccs(vec![self.get_followers_url()]);

    insert_activity(mod_.id, undo.clone(), true, pool).await?;

    let inboxes = self.get_follower_inboxes(pool).await?;

    // Note: For an accept, since it was automatic, no one pushed a button,
    // the community was the actor.
    // But for remove , the creator is the actor, and does the signing
    send_activity(client, &undo.into_any_base()?, mod_, inboxes).await?;
    Ok(())
  }

  /// For a given community, returns the inboxes of all followers.
  async fn get_follower_inboxes(&self, pool: &DbPool) -> Result<Vec<String>, LemmyError> {
    let id = self.id;

    let inboxes = blocking(pool, move |conn| {
      CommunityFollowerView::for_community(conn, id)
    })
    .await??;
    let inboxes = inboxes
      .into_iter()
      .map(|c| get_shared_inbox(&Url::parse(&c.user_actor_id).unwrap()))
      .filter(|s| !s.is_empty())
      .unique()
      .collect();

    Ok(inboxes)
  }

  async fn send_follow(
    &self,
    _follow_actor_id: &str,
    _client: &Client,
    _pool: &DbPool,
  ) -> Result<(), LemmyError> {
    unimplemented!()
  }

  async fn send_unfollow(
    &self,
    _follow_actor_id: &str,
    _client: &Client,
    _pool: &DbPool,
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
  async fn from_apub(group: &GroupExt, client: &Client, pool: &DbPool) -> Result<Self, LemmyError> {
    let creator_and_moderator_uris = group.inner.attributed_to().unwrap();
    let creator_uri = creator_and_moderator_uris
      .as_many()
      .unwrap()
      .iter()
      .next()
      .unwrap()
      .as_xsd_any_uri()
      .unwrap();

    let creator = get_or_fetch_and_upsert_user(creator_uri, client, pool).await?;

    Ok(CommunityForm {
      name: group
        .inner
        .name()
        .unwrap()
        .as_one()
        .unwrap()
        .as_xsd_string()
        .unwrap()
        .into(),
      title: group.inner.preferred_username().unwrap().to_string(),
      // TODO: should be parsed as html and tags like <script> removed (or use markdown source)
      //       -> same for post.content etc
      description: group
        .inner
        .content()
        .map(|s| s.as_single_xsd_string().unwrap().into()),
      category_id: group.ext_one.category.identifier.parse::<i32>()?,
      creator_id: creator.id,
      removed: None,
      published: group.inner.published().map(|u| u.to_owned().naive_local()),
      updated: group.inner.updated().map(|u| u.to_owned().naive_local()),
      deleted: None,
      nsfw: group.ext_one.sensitive,
      actor_id: group.inner.id_unchecked().unwrap().to_string(),
      local: false,
      private_key: None,
      public_key: Some(group.ext_two.to_owned().public_key.public_key_pem),
      last_refreshed_at: Some(naive_now()),
    })
  }
}

/// Return the community json over HTTP.
pub async fn get_apub_community_http(
  info: web::Path<CommunityQuery>,
  db: DbPoolParam,
) -> Result<HttpResponse<Body>, LemmyError> {
  let community = blocking(&db, move |conn| {
    Community::read_from_name(conn, &info.community_name)
  })
  .await??;

  if !community.deleted {
    let apub = community.to_apub(&db).await?;

    Ok(create_apub_response(&apub))
  } else {
    Ok(create_apub_tombstone_response(&community.to_tombstone()?))
  }
}

/// Returns an empty followers collection, only populating the size (for privacy).
pub async fn get_apub_community_followers(
  info: web::Path<CommunityQuery>,
  db: DbPoolParam,
) -> Result<HttpResponse<Body>, LemmyError> {
  let community = blocking(&db, move |conn| {
    Community::read_from_name(&conn, &info.community_name)
  })
  .await??;

  let community_id = community.id;
  let community_followers = blocking(&db, move |conn| {
    CommunityFollowerView::for_community(&conn, community_id)
  })
  .await??;

  let mut collection = UnorderedCollection::new(vec![]);
  collection
    .set_context(context())
    // TODO: this needs its own ID
    .set_id(community.actor_id.parse()?)
    .set_total_items(community_followers.len() as u64);
  Ok(create_apub_response(&collection))
}

pub async fn get_apub_community_outbox(
  info: web::Path<CommunityQuery>,
  db: DbPoolParam,
) -> Result<HttpResponse<Body>, LemmyError> {
  let community = blocking(&db, move |conn| {
    Community::read_from_name(&conn, &info.community_name)
  })
  .await??;

  let community_id = community.id;
  let posts = blocking(&db, move |conn| {
    Post::list_for_community(conn, community_id)
  })
  .await??;

  let mut pages: Vec<AnyBase> = vec![];
  for p in posts {
    pages.push(p.to_apub(&db).await?.into_any_base()?);
  }

  let len = pages.len();
  let mut collection = OrderedCollection::new(pages);
  collection
    .set_context(context())
    .set_id(community.get_outbox_url()?)
    .set_total_items(len as u64);
  Ok(create_apub_response(&collection))
}

pub async fn do_announce(
  activity: AnyBase,
  community: &Community,
  sender: &User_,
  client: &Client,
  pool: &DbPool,
) -> Result<(), LemmyError> {
  let mut announce = Announce::new(community.actor_id.to_owned(), activity);
  announce
    .set_context(context())
    .set_id(generate_activity_id(AnnounceType::Announce)?)
    .set_to(public())
    .set_many_ccs(vec![community.get_followers_url()]);

  insert_activity(community.creator_id, announce.clone(), true, pool).await?;

  // dont send to the instance where the activity originally came from, because that would result
  // in a database error (same data inserted twice)
  let mut to = community.get_follower_inboxes(pool).await?;

  // this seems to be the "easiest" stable alternative for remove_item()
  to.retain(|x| *x != sender.get_shared_inbox_url());

  send_activity(client, &announce.into_any_base()?, community, to).await?;

  Ok(())
}
