use crate::{
  apub::{extensions::signatures::sign, is_apub_id_valid, ActorType},
  db::{activity::insert_activity, community::Community, user::User_},
};
use activitystreams::{context, object::properties::ObjectProperties, public, Activity, Base};
use diesel::PgConnection;
use failure::{Error, _core::fmt::Debug};
use isahc::prelude::*;
use log::debug;
use serde::Serialize;
use url::Url;

pub fn populate_object_props(
  props: &mut ObjectProperties,
  addressed_ccs: Vec<String>,
  object_id: &str,
) -> Result<(), Error> {
  props
    .set_context_xsd_any_uri(context())?
    // TODO: the activity needs a seperate id from the object
    .set_id(object_id)?
    // TODO: should to/cc go on the Create, or on the Post? or on both?
    // TODO: handle privacy on the receiving side (at least ignore anything thats not public)
    .set_to_xsd_any_uri(public())?
    .set_many_cc_xsd_any_uris(addressed_ccs)?;
  Ok(())
}

pub fn send_activity_to_community<A>(
  creator: &User_,
  conn: &PgConnection,
  community: &Community,
  to: Vec<String>,
  activity: A,
) -> Result<(), Error>
where
  A: Activity + Base + Serialize + Debug,
{
  insert_activity(&conn, creator.id, &activity, true)?;

  // if this is a local community, we need to do an announce from the community instead
  if community.local {
    Community::do_announce(activity, &community, creator, conn)?;
  } else {
    send_activity(&activity, creator, to)?;
  }
  Ok(())
}

/// Send an activity to a list of recipients, using the correct headers etc.
pub fn send_activity<A>(activity: &A, actor: &dyn ActorType, to: Vec<String>) -> Result<(), Error>
where
  A: Serialize + Debug,
{
  let json = serde_json::to_string(&activity)?;
  debug!("Sending activitypub activity {} to {:?}", json, to);
  for t in to {
    let to_url = Url::parse(&t)?;
    if !is_apub_id_valid(&to_url) {
      debug!("Not sending activity to {} (invalid or blocklisted)", t);
      continue;
    }
    let request = Request::post(t).header("Host", to_url.domain().unwrap());
    let signature = sign(&request, actor)?;
    let res = request
      .header("Signature", signature)
      .header("Content-Type", "application/json")
      .body(json.to_owned())?
      .send()?;
    debug!("Result for activity send: {:?}", res);
  }
  Ok(())
}
