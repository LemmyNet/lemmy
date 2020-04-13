use crate::apub::{get_apub_protocol_string, get_following_instances};
use crate::db::community::Community;
use crate::db::post::Post;
use crate::db::user::User_;
use crate::db::Crud;
use activitystreams::activity::{Create, Update};
use activitystreams::object::properties::ObjectProperties;
use activitystreams::{context, public};
use diesel::PgConnection;
use failure::Error;
use failure::_core::fmt::Debug;
use isahc::prelude::*;
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

fn send_activity<A>(activity: &A) -> Result<(), Error>
where
  A: Serialize + Debug,
{
  let json = serde_json::to_string(&activity)?;
  for i in get_following_instances() {
    // TODO: need to send this to the inbox of following users
    let inbox = format!(
      "{}://{}/federation/inbox",
      get_apub_protocol_string(),
      i.domain
    );
    let res = Request::post(inbox)
      .header("Content-Type", "application/json")
      .body(json.to_owned())?
      .send()?;
    dbg!(res);
  }
  Ok(())
}

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
  send_activity(&create)?;
  Ok(())
}

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
  send_activity(&update)?;
  Ok(())
}
