use crate::{
  apub::{
    extensions::signatures::verify,
    fetcher::{get_or_fetch_and_upsert_remote_community, get_or_fetch_and_upsert_remote_user},
    ActorType,
  },
  db::{
    activity::insert_activity,
    community::{Community, CommunityFollower, CommunityFollowerForm},
    user::User_,
    Followable,
  },
  routes::{ChatServerParam, DbPoolParam},
};
use activitystreams::activity::{Follow, Undo, Update, Create, Delete, Remove};
use actix_web::{web, HttpRequest, HttpResponse, Result};
use diesel::PgConnection;
use failure::{Error, _core::fmt::Debug};
use log::debug;
use serde::{Deserialize, Serialize};
use activitystreams::activity::{Activity, Announce};
use activitystreams::Base;
use crate::apub::activities::{populate_object_props, send_activity};
use activitystreams::BaseBox;

#[serde(untagged)]
#[derive(Deserialize, Debug)]
pub enum CommunityAcceptedObjects {
  Follow(Follow),
  Undo(Undo),
  Create(Create),
  Update(Update),
  Delete(Delete),
  Remove(Remove),
}

impl CommunityAcceptedObjects {
  fn follow(&self) -> Result<Follow, Error> {
    match self {
      CommunityAcceptedObjects::Follow(f) => Ok(f.to_owned()),
      CommunityAcceptedObjects::Undo(u) => Ok(
        u.undo_props
          .get_object_base_box()
          .to_owned()
          .unwrap()
          .to_owned()
          .into_concrete::<Follow>()?,
      ),
      _ => todo!()
    }
  }
}

// TODO Consolidate community and user inboxes into a single shared one
/// Handler for all incoming activities to community inboxes.
pub async fn community_inbox(
  request: HttpRequest,
  input: web::Json<CommunityAcceptedObjects>,
  path: web::Path<String>,
  db: DbPoolParam,
  chat_server: ChatServerParam,
) -> Result<HttpResponse, Error> {
  let input = input.into_inner();
  let conn = db.get()?;
  let community = Community::read_from_name(&conn, &path.into_inner())?;
  if !community.local {
    return Err(format_err!(
      "Received activity is addressed to remote community {}",
      &community.actor_id
    ));
  }
  debug!(
    "Community {} received activity {:?}",
    &community.name, &input
  );
  let follow = input.follow()?;
  let user_uri = follow
    .follow_props
    .get_actor_xsd_any_uri()
    .unwrap()
    .to_string();
  let community_uri = follow
    .follow_props
    .get_object_xsd_any_uri()
    .unwrap()
    .to_string();

  let conn = db.get()?;

  let user = get_or_fetch_and_upsert_remote_user(&user_uri, &conn)?;
  let community = get_or_fetch_and_upsert_remote_community(&community_uri, &conn)?;

  verify(&request, &user)?;

  match input {
    CommunityAcceptedObjects::Follow(f) => {
      handle_follow(&f, &user,  &community, &conn)
    }
    CommunityAcceptedObjects::Undo(u) => {
      // TODO: if this is an undo<remove> or undo<delete>, we need to announce it instead
      handle_undo_follow(&u, &user,  &community, &conn)
    }
    // TODO: we should be able to handle all this with a single wildcard match, but i dont see how
    //       to get the value from that
    CommunityAcceptedObjects::Create(c) => {
      do_announce(c, &request, &community, &conn, chat_server)
    }
    CommunityAcceptedObjects::Update(u) => {
      do_announce(u, &request, &community, &conn, chat_server)
    }
    CommunityAcceptedObjects::Delete(d) => {
      do_announce(d, &request, &community, &conn, chat_server)
    }
    CommunityAcceptedObjects::Remove(r) => {
      do_announce(r, &request, &community, &conn, chat_server)
    }
  }
}

/// Handle a follow request from a remote user, adding it to the local database and returning an
/// Accept activity.
fn handle_follow(
  follow: &Follow,
  user: &User_,
  community: &Community,
  conn: &PgConnection,
) -> Result<HttpResponse, Error> {
  insert_activity(&conn, user.id, &follow, false)?;

  let community_follower_form = CommunityFollowerForm {
    community_id: community.id,
    user_id: user.id,
  };

  // This will fail if they're already a follower, but ignore the error.
  CommunityFollower::follow(&conn, &community_follower_form).ok();

  community.send_accept_follow(&follow, &conn)?;

  Ok(HttpResponse::Ok().finish())
}

fn handle_undo_follow(
  undo: &Undo,
  user: &User_,
  community: &Community,
  conn: &PgConnection,
) -> Result<HttpResponse, Error> {
  insert_activity(&conn, user.id, &undo, false)?;

  let community_follower_form = CommunityFollowerForm {
    community_id: community.id,
    user_id: user.id,
  };

  CommunityFollower::unfollow(&conn, &community_follower_form).ok();

  Ok(HttpResponse::Ok().finish())
}

fn do_announce<A>(
  activity: A,
  _request: &HttpRequest,
  community: &Community,
  conn: &PgConnection,
  _chat_server: ChatServerParam,
) -> Result<HttpResponse, Error>
where
  A: Activity + Base + Serialize,
{
  // TODO: checking the signature needs a lot of boilerplate, unless this gets implemented
  // https://git.asonix.dog/Aardwolf/activitystreams/issues/4
  /*
  let user_uri = activity
      .follow_props
      .get_actor_xsd_any_uri()
      .unwrap()
      .to_string();
  let user = get_or_fetch_and_upsert_remote_user(&user_uri, &conn)?;
  verify(&request, &user.public_key.unwrap())?;
  */

  insert_activity(&conn, -1, &activity, false)?;

  // TODO: handle the sending in community.rs
  let mut announce = Announce::default();
  populate_object_props(
    &mut announce.object_props,
    vec!(community.get_followers_url()),
    &format!("{}/announce/{}", community.actor_id, uuid::Uuid::new_v4()),
  )?;
  announce
    .announce_props
    .set_actor_xsd_any_uri(community.actor_id.to_owned())?
    .set_object_base_box(BaseBox::from_concrete(activity)?)?;

  insert_activity(&conn, -1, &announce, true)?;

  send_activity(
    &announce,
    community,
    community.get_follower_inboxes(&conn)?,
  )?;

  Ok(HttpResponse::Ok().finish())
}
