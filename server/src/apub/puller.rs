use crate::apub::*;
use crate::db::community::{Community, CommunityForm};
use crate::db::post::{Post, PostForm};
use crate::db::user::{UserForm, User_};
use crate::db::Crud;
use crate::routes::nodeinfo::{NodeInfo, NodeInfoWellKnown};
use crate::settings::Settings;
use activitystreams::collection::{OrderedCollection, UnorderedCollection};
use activitystreams::object::Page;
use activitystreams::BaseBox;
use diesel::result::Error::NotFound;
use diesel::PgConnection;
use failure::Error;
use isahc::prelude::*;
use serde::Deserialize;
use std::time::Duration;

fn fetch_node_info(domain: &str) -> Result<NodeInfo, Error> {
  let well_known_uri = format!(
    "{}://{}/.well-known/nodeinfo",
    get_apub_protocol_string(),
    domain
  );
  let well_known = fetch_remote_object::<NodeInfoWellKnown>(&well_known_uri)?;
  Ok(fetch_remote_object::<NodeInfo>(&well_known.links.href)?)
}

fn fetch_communities_from_instance(
  domain: &str,
  conn: &PgConnection,
) -> Result<Vec<CommunityForm>, Error> {
  let node_info = fetch_node_info(domain)?;

  if let Some(community_list_url) = node_info.metadata.community_list_url {
    let collection = fetch_remote_object::<UnorderedCollection>(&community_list_url)?;
    let object_boxes = collection
      .collection_props
      .get_many_items_base_boxes()
      .unwrap();
    let communities: Result<Vec<CommunityForm>, Error> = object_boxes
      .map(|c| {
        let group = c.to_owned().to_concrete::<GroupExt>()?;
        CommunityForm::from_group(&group, conn)
      })
      .collect();
    Ok(communities?)
  } else {
    Err(format_err!(
      "{} is not a Lemmy instance, federation is not supported",
      domain
    ))
  }
}

// TODO: add an optional param last_updated and only fetch if its too old
pub fn fetch_remote_object<Response>(uri: &str) -> Result<Response, Error>
where
  Response: for<'de> Deserialize<'de>,
{
  if Settings::get().federation.tls_enabled && !uri.starts_with("https://") {
    return Err(format_err!("Activitypub uri is insecure: {}", uri));
  }
  // TODO: should cache responses here when we are in production
  // TODO: this function should return a future
  let timeout = Duration::from_secs(60);
  let text = Request::get(uri)
    .header("Accept", APUB_JSON_CONTENT_TYPE)
    .connect_timeout(timeout)
    .timeout(timeout)
    .body(())?
    .send()?
    .text()?;
  let res: Response = serde_json::from_str(&text)?;
  Ok(res)
}

fn fetch_remote_community_posts(
  instance: &str,
  community: &str,
  conn: &PgConnection,
) -> Result<Vec<PostForm>, Error> {
  let endpoint = format!("http://{}/federation/c/{}", instance, community);
  let community = fetch_remote_object::<GroupExt>(&endpoint)?;
  let outbox_uri = &community.extension.get_outbox().to_string();
  let outbox = fetch_remote_object::<OrderedCollection>(outbox_uri)?;
  let items = outbox.collection_props.get_many_items_base_boxes();

  let posts = items
    .unwrap()
    .map(|obox: &BaseBox| {
      let page = obox.clone().to_concrete::<Page>().unwrap();
      PostForm::from_page(&page, conn)
    })
    .collect::<Result<Vec<PostForm>, Error>>()?;
  Ok(posts)
}

pub fn fetch_remote_user(apub_id: &str, conn: &PgConnection) -> Result<User_, Error> {
  let person = fetch_remote_object::<PersonExt>(apub_id)?;
  let uf = UserForm::from_person(&person)?;
  let existing = User_::read_from_apub_id(conn, &uf.actor_id);
  Ok(match existing {
    Err(NotFound {}) => User_::create(conn, &uf)?,
    Ok(u) => User_::update(conn, u.id, &uf)?,
    Err(e) => return Err(Error::from(e)),
  })
}

// TODO: in the future, this should only be done when an instance is followed for the first time
//       after that, we should rely in the inbox, and fetch on demand when needed
pub fn fetch_all(conn: &PgConnection) -> Result<(), Error> {
  for instance in &get_following_instances() {
    let communities = fetch_communities_from_instance(instance, conn)?;

    for community in &communities {
      let existing = Community::read_from_actor_id(conn, &community.actor_id);
      let community_id = match existing {
        Err(NotFound {}) => Community::create(conn, community)?.id,
        Ok(c) => Community::update(conn, c.id, community)?.id,
        Err(e) => return Err(Error::from(e)),
      };
      let mut posts = fetch_remote_community_posts(instance, &community.name, conn)?;
      for post_ in &mut posts {
        post_.community_id = community_id;
        let existing = Post::read_from_apub_id(conn, &post_.ap_id);
        match existing {
          Err(NotFound {}) => Post::create(conn, post_)?,
          Ok(p) => Post::update(conn, p.id, post_)?,
          Err(e) => return Err(Error::from(e)),
        };
      }
    }
  }
  Ok(())
}
