use crate::{
  apub::{
    activities::{populate_object_props, send_activity},
    create_apub_response,
    create_apub_tombstone_response,
    create_tombstone,
    extensions::group_extensions::GroupExtension,
    fetcher::get_or_fetch_and_upsert_remote_user,
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
use activitystreams::{
  activity::{Accept, Announce, Delete, Remove, Undo},
  Activity,
  Base,
  BaseBox,
};
use activitystreams_ext::Ext2;
use activitystreams_new::{
  activity::Follow,
  actor::{kind::GroupType, ApActor, Endpoints, Group},
  base::BaseExt,
  collection::UnorderedCollection,
  context,
  object::Tombstone,
  prelude::*,
  primitives::{XsdAnyUri, XsdDateTime},
};
use actix_web::{body::Body, client::Client, web, HttpResponse};
use itertools::Itertools;
use lemmy_db::{
  community::{Community, CommunityForm},
  community_view::{CommunityFollowerView, CommunityModeratorView},
  naive_now,
  user::User_,
};
use lemmy_utils::convert_datetime;
use serde::{Deserialize, Serialize};
use std::{fmt::Debug, str::FromStr};

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
      .set_id(XsdAnyUri::from_str(&self.actor_id)?)
      .set_name(self.name.to_owned())
      .set_published(XsdDateTime::from(convert_datetime(self.published)))
      .set_many_attributed_tos(moderators);

    if let Some(u) = self.updated.to_owned() {
      group.set_updated(XsdDateTime::from(convert_datetime(u)));
    }
    if let Some(d) = self.description.to_owned() {
      // TODO: this should be html, also add source field with raw markdown
      //       -> same for post.content and others
      group.set_content(d);
    }

    let mut ap_actor = ApActor::new(self.get_inbox_url().parse()?, group);
    ap_actor
      .set_preferred_username(self.title.to_owned())
      .set_outbox(self.get_outbox_url().parse()?)
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
  async fn from_apub(
    group: &mut GroupExt,
    client: &Client,
    pool: &DbPool,
  ) -> Result<Self, LemmyError> {
    // TODO: this is probably gonna cause problems cause fetcher:292 also calls take_attributed_to()
    let creator_and_moderator_uris = group.clone().take_attributed_to().unwrap();
    let creator_uri = creator_and_moderator_uris
      .as_many()
      .unwrap()
      .iter()
      .next()
      .unwrap()
      .as_xsd_any_uri()
      .unwrap();

    let creator = get_or_fetch_and_upsert_remote_user(creator_uri.as_str(), client, pool).await?;

    Ok(CommunityForm {
      name: group
        .take_name()
        .unwrap()
        .as_single_xsd_string()
        .unwrap()
        .into(),
      title: group.inner.take_preferred_username().unwrap(),
      // TODO: should be parsed as html and tags like <script> removed (or use markdown source)
      //       -> same for post.content etc
      description: group
        .take_content()
        .map(|s| s.as_single_xsd_string().unwrap().into()),
      category_id: group.ext_one.category.identifier.parse::<i32>()?,
      creator_id: creator.id,
      removed: None,
      published: group
        .take_published()
        .map(|u| u.as_ref().to_owned().naive_local()),
      updated: group
        .take_updated()
        .map(|u| u.as_ref().to_owned().naive_local()),
      deleted: None,
      nsfw: group.ext_one.sensitive,
      actor_id: group.id().unwrap().to_string(),
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
