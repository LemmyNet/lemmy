use crate::apub::is_apub_id_valid;
use crate::db::community::Community;
use crate::db::community_view::CommunityFollowerView;
use crate::db::post::Post;
use crate::db::user::User_;
use crate::db::Crud;
use activitystreams::activity::{Accept, Create, Follow, Update};
use activitystreams::object::properties::ObjectProperties;
use activitystreams::BaseBox;
use activitystreams::{context, public};
use diesel::PgConnection;
use failure::Error;
use failure::_core::fmt::Debug;
use isahc::prelude::*;
use log::debug;
use serde::Serialize;

fn populate_object_props(
  props: &mut ObjectProperties,
  addressed_to: &str,
  object_id: &str,
) -> Result<(), Error> {
  props
    .set_context_xsd_any_uri(context())?
    // TODO: the activity needs a seperate id from the object
    .set_id(object_id)?
    // TODO: should to/cc go on the Create, or on the Post? or on both?
    // TODO: handle privacy on the receiving side (at least ignore anything thats not public)
    .set_to_xsd_any_uri(public())?
    .set_cc_xsd_any_uri(addressed_to)?;
  Ok(())
}

/// Send an activity to a list of recipients, using the correct headers etc.
fn send_activity<A>(activity: &A, to: Vec<String>) -> Result<(), Error>
where
  A: Serialize + Debug,
{
  let json = serde_json::to_string(&activity)?;
  debug!("Sending activitypub activity {} to {:?}", json, to);
  for t in to {
    if is_apub_id_valid(&t) {
      debug!("Not sending activity to {} (invalid or blacklisted)", t);
      continue;
    }
    let res = Request::post(t)
      .header("Content-Type", "application/json")
      .body(json.to_owned())?
      .send()?;
    debug!("Result for activity send: {:?}", res);
  }
  Ok(())
}

/// For a given community, returns the inboxes of all followers.
fn get_follower_inboxes(conn: &PgConnection, community: &Community) -> Result<Vec<String>, Error> {
  Ok(
    CommunityFollowerView::for_community(conn, community.id)?
      .iter()
      .filter(|c| !c.user_local)
      .map(|c| format!("{}/inbox", c.user_actor_id.to_owned()))
      .collect(),
  )
}

/// Send out information about a newly created post, to the followers of the community.
pub fn post_create(post: &Post, creator: &User_, conn: &PgConnection) -> Result<(), Error> {
  let page = post.as_page(conn)?;
  let community = Community::read(conn, post.community_id)?;
  let mut create = Create::new();
  populate_object_props(
    &mut create.object_props,
    &community.get_followers_url(),
    &post.ap_id,
  )?;
  create
    .create_props
    .set_actor_xsd_any_uri(creator.actor_id.to_owned())?
    .set_object_base_box(page)?;
  send_activity(&create, get_follower_inboxes(conn, &community)?)?;
  Ok(())
}

/// Send out information about an edited post, to the followers of the community.
pub fn post_update(post: &Post, creator: &User_, conn: &PgConnection) -> Result<(), Error> {
  let page = post.as_page(conn)?;
  let community = Community::read(conn, post.community_id)?;
  let mut update = Update::new();
  populate_object_props(
    &mut update.object_props,
    &community.get_followers_url(),
    &post.ap_id,
  )?;
  update
    .update_props
    .set_actor_xsd_any_uri(creator.actor_id.to_owned())?
    .set_object_base_box(page)?;
  send_activity(&update, get_follower_inboxes(conn, &community)?)?;
  Ok(())
}

/// As a given local user, send out a follow request to a remote community.
pub fn follow_community(
  community: &Community,
  user: &User_,
  _conn: &PgConnection,
) -> Result<(), Error> {
  let mut follow = Follow::new();
  follow
    .object_props
    .set_context_xsd_any_uri(context())?
    // TODO: needs proper id
    .set_id(user.actor_id.clone())?;
  follow
    .follow_props
    .set_actor_xsd_any_uri(user.actor_id.clone())?
    .set_object_xsd_any_uri(community.actor_id.clone())?;
  let to = format!("{}/inbox", community.actor_id);
  send_activity(&follow, vec![to])?;
  Ok(())
}

/// As a local community, accept the follow request from a remote user.
pub fn accept_follow(follow: &Follow) -> Result<(), Error> {
  let mut accept = Accept::new();
  accept
    .object_props
    .set_context_xsd_any_uri(context())?
    // TODO: needs proper id
    .set_id(
      follow
        .follow_props
        .get_actor_xsd_any_uri()
        .unwrap()
        .to_string(),
    )?;
  accept
    .accept_props
    .set_object_base_box(BaseBox::from_concrete(follow.clone())?)?;
  let to = format!(
    "{}/inbox",
    follow
      .follow_props
      .get_actor_xsd_any_uri()
      .unwrap()
      .to_string()
  );
  send_activity(&accept, vec![to])?;
  Ok(())
}
