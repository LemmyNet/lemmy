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
  convert_datetime,
  db::{
    activity::insert_activity,
    community::{Community, CommunityForm},
    community_view::{CommunityFollowerView, CommunityModeratorView},
    user::User_,
  },
  naive_now,
  routes::DbPoolParam,
};
use activitystreams::{
  activity::{Accept, Announce, Delete, Follow, Remove, Undo},
  actor::{kind::GroupType, properties::ApActorProperties, Group},
  collection::UnorderedCollection,
  context,
  endpoint::EndpointProperties,
  object::{properties::ObjectProperties, Tombstone},
  Activity,
  Base,
  BaseBox,
};
use activitystreams_ext::Ext3;
use actix_web::{body::Body, web::Path, HttpResponse, Result};
use diesel::PgConnection;
use failure::{Error, _core::fmt::Debug};
use itertools::Itertools;
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct CommunityQuery {
  community_name: String,
}

impl ToApub for Community {
  type Response = GroupExt;

  // Turn a Lemmy Community into an ActivityPub group that can be sent out over the network.
  fn to_apub(&self, conn: &PgConnection) -> Result<GroupExt, Error> {
    let mut group = Group::default();
    let oprops: &mut ObjectProperties = group.as_mut();

    // The attributed to, is an ordered vector with the creator actor_ids first,
    // then the rest of the moderators
    // TODO Technically the instance admins can mod the community, but lets
    // ignore that for now
    let moderators = CommunityModeratorView::for_community(&conn, self.id)?
      .into_iter()
      .map(|m| m.user_actor_id)
      .collect();

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
      oprops.set_summary_xsd_string(d)?;
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

    let group_extension = GroupExtension::new(conn, self.category_id, self.nsfw)?;

    Ok(Ext3::new(
      group,
      group_extension,
      actor_props,
      self.get_public_key_ext(),
    ))
  }

  fn to_tombstone(&self) -> Result<Tombstone, Error> {
    create_tombstone(
      self.deleted,
      &self.actor_id,
      self.updated,
      GroupType.to_string(),
    )
  }
}

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
  fn send_accept_follow(&self, follow: &Follow, conn: &PgConnection) -> Result<(), Error> {
    let actor_uri = follow
      .follow_props
      .get_actor_xsd_any_uri()
      .unwrap()
      .to_string();
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

    insert_activity(&conn, self.creator_id, &accept, true)?;

    send_activity(&accept, self, vec![to])?;
    Ok(())
  }

  fn send_delete(&self, creator: &User_, conn: &PgConnection) -> Result<(), Error> {
    let group = self.to_apub(conn)?;
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

    insert_activity(&conn, self.creator_id, &delete, true)?;

    // Note: For an accept, since it was automatic, no one pushed a button,
    // the community was the actor.
    // But for delete, the creator is the actor, and does the signing
    send_activity(&delete, creator, self.get_follower_inboxes(&conn)?)?;
    Ok(())
  }

  fn send_undo_delete(&self, creator: &User_, conn: &PgConnection) -> Result<(), Error> {
    let group = self.to_apub(conn)?;
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

    insert_activity(&conn, self.creator_id, &undo, true)?;

    // Note: For an accept, since it was automatic, no one pushed a button,
    // the community was the actor.
    // But for delete, the creator is the actor, and does the signing
    send_activity(&undo, creator, self.get_follower_inboxes(&conn)?)?;
    Ok(())
  }

  fn send_remove(&self, mod_: &User_, conn: &PgConnection) -> Result<(), Error> {
    let group = self.to_apub(conn)?;
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

    insert_activity(&conn, mod_.id, &remove, true)?;

    // Note: For an accept, since it was automatic, no one pushed a button,
    // the community was the actor.
    // But for delete, the creator is the actor, and does the signing
    send_activity(&remove, mod_, self.get_follower_inboxes(&conn)?)?;
    Ok(())
  }

  fn send_undo_remove(&self, mod_: &User_, conn: &PgConnection) -> Result<(), Error> {
    let group = self.to_apub(conn)?;
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

    insert_activity(&conn, mod_.id, &undo, true)?;

    // Note: For an accept, since it was automatic, no one pushed a button,
    // the community was the actor.
    // But for remove , the creator is the actor, and does the signing
    send_activity(&undo, mod_, self.get_follower_inboxes(&conn)?)?;
    Ok(())
  }

  /// For a given community, returns the inboxes of all followers.
  fn get_follower_inboxes(&self, conn: &PgConnection) -> Result<Vec<String>, Error> {
    Ok(
      CommunityFollowerView::for_community(conn, self.id)?
        .into_iter()
        .map(|c| get_shared_inbox(&c.user_actor_id))
        .filter(|s| !s.is_empty())
        .unique()
        .collect(),
    )
  }

  fn send_follow(&self, _follow_actor_id: &str, _conn: &PgConnection) -> Result<(), Error> {
    unimplemented!()
  }

  fn send_unfollow(&self, _follow_actor_id: &str, _conn: &PgConnection) -> Result<(), Error> {
    unimplemented!()
  }
}

impl FromApub for CommunityForm {
  type ApubType = GroupExt;

  /// Parse an ActivityPub group received from another instance into a Lemmy community.
  fn from_apub(group: &GroupExt, conn: &PgConnection) -> Result<Self, Error> {
    let group_extensions: &GroupExtension = &group.ext_one;
    let oprops = &group.inner.object_props;
    let aprops = &group.ext_two;
    let public_key: &PublicKey = &group.ext_three.public_key;

    let mut creator_and_moderator_uris = oprops.get_many_attributed_to_xsd_any_uris().unwrap();
    let creator = creator_and_moderator_uris
      .next()
      .map(|c| get_or_fetch_and_upsert_remote_user(&c.to_string(), &conn).unwrap())
      .unwrap();

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
  info: Path<CommunityQuery>,
  db: DbPoolParam,
) -> Result<HttpResponse<Body>, Error> {
  let community = Community::read_from_name(&&db.get()?, &info.community_name)?;
  if !community.deleted {
    Ok(create_apub_response(
      &community.to_apub(&db.get().unwrap())?,
    ))
  } else {
    Ok(create_apub_tombstone_response(&community.to_tombstone()?))
  }
}

/// Returns an empty followers collection, only populating the size (for privacy).
pub async fn get_apub_community_followers(
  info: Path<CommunityQuery>,
  db: DbPoolParam,
) -> Result<HttpResponse<Body>, Error> {
  let community = Community::read_from_name(&&db.get()?, &info.community_name)?;

  let conn = db.get()?;

  //As we are an object, we validated that the community id was valid
  let community_followers = CommunityFollowerView::for_community(&conn, community.id).unwrap();

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
  pub fn do_announce<A>(
    activity: A,
    // TODO: maybe pass in the community object
    community_uri: &str,
    sender: &str,
    conn: &PgConnection,
    is_local_activity: bool,
  ) -> Result<HttpResponse, Error>
  where
    A: Activity + Base + Serialize + Debug,
  {
    let community = Community::read_from_actor_id(conn, &community_uri)?;

    insert_activity(&conn, -1, &activity, is_local_activity)?;

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

    insert_activity(&conn, -1, &announce, true)?;

    // dont send to the instance where the activity originally came from, because that would result
    // in a database error (same data inserted twice)
    let mut to = community.get_follower_inboxes(&conn)?;
    let sending_user = get_or_fetch_and_upsert_remote_user(&sender, conn)?;
    // this seems to be the "easiest" stable alternative for remove_item()
    to.retain(|x| *x != sending_user.get_shared_inbox_url());

    send_activity(&announce, &community, to)?;

    Ok(HttpResponse::Ok().finish())
  }
}
