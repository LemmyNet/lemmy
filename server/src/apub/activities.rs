use crate::apub::{get_apub_protocol_string, get_following_instances};
use crate::db::community::Community;
use crate::db::post::Post;
use crate::db::user::User_;
use crate::db::Crud;
use activitystreams::activity::Create;
use activitystreams::context;
use diesel::PgConnection;
use failure::Error;
use isahc::prelude::*;

pub fn post_create(post: &Post, creator: &User_, conn: &PgConnection) -> Result<(), Error> {
  let page = post.as_page(conn)?;
  let community = Community::read(conn, post.community_id)?;
  let mut create = Create::new();
  create.object_props.set_context_xsd_any_uri(context())?;
  create
    .object_props
    // TODO: seems like the create activity needs its own id (and be fetchable there)
    .set_id(page.object_props.get_id().unwrap().to_string())?
    // TODO: should to/cc go on the Create, or on the Post? or on both?
    // TODO: handle privacy on the receiving side (at least ignore anything thats not public)
    .set_to_xsd_any_uri("https://www.w3.org/ns/activitystreams#Public")?
    .set_cc_xsd_any_uri(format!("{}/followers", community.actor_id))?;
  create
    .create_props
    .set_actor_xsd_any_uri(creator.actor_id.to_owned())?;
  create.create_props.set_object_base_box(page)?;
  let json = serde_json::to_string(&create)?;
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
