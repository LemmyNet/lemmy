use crate::{
  apub::{
    activities::{populate_object_props, send_activity},
    create_apub_response,
    create_apub_tombstone_response,
    create_tombstone,
    extensions::{group_extensions::GroupExtension, signatures::PublicKey},
    fetcher::get_or_fetch_and_upsert_remote_user,
    get_shared_inbox,
    ActorType,
    FromApub,
    GroupExt,
    ToApub,
  },
  blocking,
  convert_datetime,
  db::{
    activity::insert_activity,
    community::{Community, CommunityForm},
    community_view::{CommunityFollowerView, CommunityModeratorView},
    user::User_,
  },
  naive_now,
  routes::DbPoolParam,
  DbPool,
  LemmyError,
};
use activitystreams::{
  activity::{Accept, Announce, Delete, Remove, Undo},
  actor::{kind::GroupType, properties::ApActorProperties, Group},
  collection::UnorderedCollection,
  context,
  endpoint::EndpointProperties,
  object::properties::ObjectProperties,
  Activity,
  Base,
  BaseBox,
};
use activitystreams_ext::Ext3;
use activitystreams_new::{activity::Follow, object::Tombstone};
use actix_web::{body::Body, client::Client, web, HttpResponse};
use itertools::Itertools;
use serde::{Deserialize, Serialize};
use std::fmt::Debug;

#[derive(Deserialize)]
pub struct CommunityQuery {
  community_name: String,
}

#[async_trait::async_trait(?Send)]
impl ToApub for Community {
  type Response = GroupExt;

  // Turn a Lemmy Community into an ActivityPub group that can be sent out over the network.
  async fn to_apub(&self, pool: &DbPool) -> Result<GroupExt, LemmyError> {
    let mut group = Group::default();
    let oprops: &mut ObjectProperties = group.as_mut();

    // The attributed to, is an ordered vector with the creator actor_ids first,
    // then the rest of the moderators
    // TODO Technically the instance admins can mod the community, but lets
    // ignore that for now
    let id = self.id;
    let moderators = blocking(pool, move |conn| {
      CommunityModeratorView::for_community(&conn, id)
    })
    .await??;
    let moderators = moderators.into_iter().map(|m| m.user_actor_id).collect();

    oprops
      .set_context_xsd_any_uri(context())?
      .set_id(self.actor_id.to_owned())?
      .set_name_xsd_string(self.name.to_owned())?
      .set_published(convert_datetime(self.published))?
      .set_many_attributed_to_xsd_any_uris(moderators)?;

    if let Some(u) = self.updated.to_owned() {
      oprops.set_updated(convert_datetime(u))?;
    }
    if let Some(d) = self.description.to_owned() {
      // TODO: this should be html, also add source field with raw markdown
      //       -> same for post.content and others
      oprops.set_content_xsd_string(d)?;
    }

    let mut endpoint_props = EndpointProperties::default();

    endpoint_props.set_shared_inbox(self.get_shared_inbox_url())?;

    let mut actor_props = ApActorProperties::default();

    actor_props
      .set_preferred_username(self.title.to_owned())?
      .set_inbox(self.get_inbox_url())?
      .set_outbox(self.get_outbox_url())?
      .set_endpoints(endpoint_props)?
      .set_followers(self.get_followers_url())?;

    let nsfw = self.nsfw;
    let category_id = self.category_id;
    let group_extension = blocking(pool, move |conn| {
      GroupExtension::new(conn, category_id, nsfw)
    })
    .await??;

    Ok(Ext3::new(
      group,
      group_extension,
      actor_props,
      self.get_public_key_ext(),
    ))
  }

  fn to_tombstone(&self) -> Result<Tombstone, LemmyError> {
    create_tombstone(
      self.deleted,
      &self.actor_id,
      self.updated,
      GroupType.to_string(),
    )
  }
}

#[async_trait::async_trait(?Send)]
impl ActorType for Community {
  fn actor_id(&self) -> String {
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
    follow: &Follow,
    client: &Client,
    pool: &DbPool,
  ) -> Result<(), LemmyError> {
    let actor_uri = follow.actor.as_single_xsd_any_uri().unwrap().to_string();
    let id = format!("{}/accept/{}", self.actor_id, uuid::Uuid::new_v4());

    let mut accept = Accept::new();
    accept
      .object_props
      .set_context_xsd_any_uri(context())?
      .set_id(id)?;
    accept
      .accept_props
      .set_actor_xsd_any_uri(self.actor_id.to_owned())?
      .set_object_base_box(BaseBox::from_concrete(follow.clone())?)?;
    let to = format!("{}/inbox", actor_uri);

    insert_activity(self.creator_id, accept.clone(), true, pool).await?;

    send_activity(client, &accept, self, vec![to]).await?;
    Ok(())
  }

  async fn send_delete(
    &self,
    creator: &User_,
    client: &Client,
    pool: &DbPool,
  ) -> Result<(), LemmyError> {
    let group = self.to_apub(pool).await?;

    let id = format!("{}/delete/{}", self.actor_id, uuid::Uuid::new_v4());

    let mut delete = Delete::default();
    populate_object_props(
      &mut delete.object_props,
      vec![self.get_followers_url()],
      &id,
    )?;

    delete
      .delete_props
      .set_actor_xsd_any_uri(creator.actor_id.to_owned())?
      .set_object_base_box(BaseBox::from_concrete(group)?)?;

    insert_activity(self.creator_id, delete.clone(), true, pool).await?;

    let inboxes = self.get_follower_inboxes(pool).await?;

    // Note: For an accept, since it was automatic, no one pushed a button,
    // the community was the actor.
    // But for delete, the creator is the actor, and does the signing
    send_activity(client, &delete, creator, inboxes).await?;
    Ok(())
  }

  async fn send_undo_delete(
    &self,
    creator: &User_,
    client: &Client,
    pool: &DbPool,
  ) -> Result<(), LemmyError> {
    let group = self.to_apub(pool).await?;

    let id = format!("{}/delete/{}", self.actor_id, uuid::Uuid::new_v4());

    let mut delete = Delete::default();
    populate_object_props(
      &mut delete.object_props,
      vec![self.get_followers_url()],
      &id,
    )?;

    delete
      .delete_props
      .set_actor_xsd_any_uri(creator.actor_id.to_owned())?
      .set_object_base_box(BaseBox::from_concrete(group)?)?;

    // TODO
    // Undo that fake activity
    let undo_id = format!("{}/undo/delete/{}", self.actor_id, uuid::Uuid::new_v4());
    let mut undo = Undo::default();

    populate_object_props(
      &mut undo.object_props,
      vec![self.get_followers_url()],
      &undo_id,
    )?;

    undo
      .undo_props
      .set_actor_xsd_any_uri(creator.actor_id.to_owned())?
      .set_object_base_box(delete)?;

    insert_activity(self.creator_id, undo.clone(), true, pool).await?;

    let inboxes = self.get_follower_inboxes(pool).await?;

    // Note: For an accept, since it was automatic, no one pushed a button,
    // the community was the actor.
    // But for delete, the creator is the actor, and does the signing
    send_activity(client, &undo, creator, inboxes).await?;
    Ok(())
  }

  async fn send_remove(
    &self,
    mod_: &User_,
    client: &Client,
    pool: &DbPool,
  ) -> Result<(), LemmyError> {
    let group = self.to_apub(pool).await?;

    let id = format!("{}/remove/{}", self.actor_id, uuid::Uuid::new_v4());

    let mut remove = Remove::default();
    populate_object_props(
      &mut remove.object_props,
      vec![self.get_followers_url()],
      &id,
    )?;

    remove
      .remove_props
      .set_actor_xsd_any_uri(mod_.actor_id.to_owned())?
      .set_object_base_box(BaseBox::from_concrete(group)?)?;

    insert_activity(mod_.id, remove.clone(), true, pool).await?;

    let inboxes = self.get_follower_inboxes(pool).await?;

    // Note: For an accept, since it was automatic, no one pushed a button,
    // the community was the actor.
    // But for delete, the creator is the actor, and does the signing
    send_activity(client, &remove, mod_, inboxes).await?;
    Ok(())
  }

  async fn send_undo_remove(
    &self,
    mod_: &User_,
    client: &Client,
    pool: &DbPool,
  ) -> Result<(), LemmyError> {
    let group = self.to_apub(pool).await?;

    let id = format!("{}/remove/{}", self.actor_id, uuid::Uuid::new_v4());

    let mut remove = Remove::default();
    populate_object_props(
      &mut remove.object_props,
      vec![self.get_followers_url()],
      &id,
    )?;

    remove
      .remove_props
      .set_actor_xsd_any_uri(mod_.actor_id.to_owned())?
      .set_object_base_box(BaseBox::from_concrete(group)?)?;

    // Undo that fake activity
    let undo_id = format!("{}/undo/remove/{}", self.actor_id, uuid::Uuid::new_v4());
    let mut undo = Undo::default();

    populate_object_props(
      &mut undo.object_props,
      vec![self.get_followers_url()],
      &undo_id,
    )?;

    undo
      .undo_props
      .set_actor_xsd_any_uri(mod_.actor_id.to_owned())?
      .set_object_base_box(remove)?;

    insert_activity(mod_.id, undo.clone(), true, pool).await?;

    let inboxes = self.get_follower_inboxes(pool).await?;

    // Note: For an accept, since it was automatic, no one pushed a button,
    // the community was the actor.
    // But for remove , the creator is the actor, and does the signing
    send_activity(client, &undo, mod_, inboxes).await?;
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
      .map(|c| get_shared_inbox(&c.user_actor_id))
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
}

#[async_trait::async_trait(?Send)]
impl FromApub for CommunityForm {
  type ApubType = GroupExt;

  /// Parse an ActivityPub group received from another instance into a Lemmy community.
  async fn from_apub(group: &GroupExt, client: &Client, pool: &DbPool) -> Result<Self, LemmyError> {
    let group_extensions: &GroupExtension = &group.ext_one;
    let oprops = &group.inner.object_props;
    let aprops = &group.ext_two;
    let public_key: &PublicKey = &group.ext_three.public_key;

    let mut creator_and_moderator_uris = oprops.get_many_attributed_to_xsd_any_uris().unwrap();
    let creator_uri = creator_and_moderator_uris.next().unwrap();

    let creator = get_or_fetch_and_upsert_remote_user(creator_uri.as_str(), client, pool).await?;

    Ok(CommunityForm {
      name: oprops.get_name_xsd_string().unwrap().to_string(),
      title: aprops.get_preferred_username().unwrap().to_string(),
      // TODO: should be parsed as html and tags like <script> removed (or use markdown source)
      //       -> same for post.content etc
      description: oprops.get_content_xsd_string().map(|s| s.to_string()),
      category_id: group_extensions.category.identifier.parse::<i32>()?,
      creator_id: creator.id,
      removed: None,
      published: oprops
        .get_published()
        .map(|u| u.as_ref().to_owned().naive_local()),
      updated: oprops
        .get_updated()
        .map(|u| u.as_ref().to_owned().naive_local()),
      deleted: None,
      nsfw: group_extensions.sensitive,
      actor_id: oprops.get_id().unwrap().to_string(),
      local: false,
      private_key: None,
      public_key: Some(public_key.to_owned().public_key_pem),
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

  let mut collection = UnorderedCollection::default();
  let oprops: &mut ObjectProperties = collection.as_mut();
  oprops
    .set_context_xsd_any_uri(context())?
    .set_id(community.actor_id)?;
  collection
    .collection_props
    .set_total_items(community_followers.len() as u64)?;
  Ok(create_apub_response(&collection))
}

impl Community {
  pub async fn do_announce<A>(
    activity: A,
    community: &Community,
    sender: &dyn ActorType,
    client: &Client,
    pool: &DbPool,
  ) -> Result<HttpResponse, LemmyError>
  where
    A: Activity + Base + Serialize + Debug,
  {
    let mut announce = Announce::default();
    populate_object_props(
      &mut announce.object_props,
      vec![community.get_followers_url()],
      &format!("{}/announce/{}", community.actor_id, uuid::Uuid::new_v4()),
    )?;
    announce
      .announce_props
      .set_actor_xsd_any_uri(community.actor_id.to_owned())?
      .set_object_base_box(BaseBox::from_concrete(activity)?)?;

    insert_activity(community.creator_id, announce.clone(), true, pool).await?;

    // dont send to the instance where the activity originally came from, because that would result
    // in a database error (same data inserted twice)
    let mut to = community.get_follower_inboxes(pool).await?;

    // this seems to be the "easiest" stable alternative for remove_item()
    to.retain(|x| *x != sender.get_shared_inbox_url());

    send_activity(client, &announce, community, to).await?;

    Ok(HttpResponse::Ok().finish())
  }
}
