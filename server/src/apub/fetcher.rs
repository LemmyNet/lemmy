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
use log::warn;
use serde::Deserialize;
use std::time::Duration;
use url::Url;

fn fetch_node_info(instance: &Instance) -> Result<NodeInfo, Error> {
  let well_known_uri = Url::parse(&format!(
    "{}://{}/.well-known/nodeinfo",
    get_apub_protocol_string(),
    instance.domain
  ))?;
  let well_known = fetch_remote_object::<NodeInfoWellKnown>(&well_known_uri)?;
  Ok(fetch_remote_object::<NodeInfo>(&well_known.links.href)?)
}

fn fetch_communities_from_instance(
  community_list: &Url,
  conn: &PgConnection,
) -> Result<Vec<Community>, Error> {
  fetch_remote_object::<UnorderedCollection>(community_list)?
    .collection_props
    .get_many_items_base_boxes()
    .unwrap()
    .map(|b| -> Result<CommunityForm, Error> {
      let group = b.to_owned().to_concrete::<GroupExt>()?;
      Ok(CommunityForm::from_group(&group, conn)?)
    })
    .map(
      |cf: Result<CommunityForm, Error>| -> Result<Community, Error> {
        let cf2 = cf?;
        let existing = Community::read_from_actor_id(conn, &cf2.actor_id);
        match existing {
          Err(NotFound {}) => Ok(Community::create(conn, &cf2)?),
          Ok(c) => Ok(Community::update(conn, c.id, &cf2)?),
          Err(e) => Err(Error::from(e)),
        }
      },
    )
    .collect()
}

// TODO: add an optional param last_updated and only fetch if its too old
pub fn fetch_remote_object<Response>(url: &Url) -> Result<Response, Error>
where
  Response: for<'de> Deserialize<'de>,
{
  if Settings::get().federation.tls_enabled && url.scheme() != "https" {
    return Err(format_err!("Activitypub uri is insecure: {}", url));
  }
  // TODO: this function should return a future
  let timeout = Duration::from_secs(60);
  let text = Request::get(url.as_str())
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
  community: &Community,
  conn: &PgConnection,
) -> Result<Vec<Post>, Error> {
  let outbox_url = Url::parse(&community.get_outbox_url())?;
  let outbox = fetch_remote_object::<OrderedCollection>(&outbox_url)?;
  let items = outbox.collection_props.get_many_items_base_boxes();

  Ok(
    items
      .unwrap()
      .map(|obox: &BaseBox| -> Result<PostForm, Error> {
        let page = obox.clone().to_concrete::<Page>()?;
        PostForm::from_page(&page, conn)
      })
      .map(|pf: Result<PostForm, Error>| -> Result<Post, Error> {
        let pf2 = pf?;
        let existing = Post::read_from_apub_id(conn, &pf2.ap_id);
        match existing {
          Err(NotFound {}) => Ok(Post::create(conn, &pf2)?),
          Ok(p) => Ok(Post::update(conn, p.id, &pf2)?),
          Err(e) => Err(Error::from(e)),
        }
      })
      .collect::<Result<Vec<Post>, Error>>()?,
  )
}

// TODO: can probably merge these two methods?
pub fn fetch_remote_user(apub_id: &Url, conn: &PgConnection) -> Result<User_, Error> {
  let person = fetch_remote_object::<PersonExt>(apub_id)?;
  let uf = UserForm::from_person(&person)?;
  let existing = User_::read_from_apub_id(conn, &uf.actor_id);
  Ok(match existing {
    Err(NotFound {}) => User_::create(conn, &uf)?,
    Ok(u) => User_::update(conn, u.id, &uf)?,
    Err(e) => return Err(Error::from(e)),
  })
}
pub fn fetch_remote_community(apub_id: &Url, conn: &PgConnection) -> Result<Community, Error> {
  let group = fetch_remote_object::<GroupExt>(apub_id)?;
  let cf = CommunityForm::from_group(&group, conn)?;
  let existing = Community::read_from_actor_id(conn, &cf.actor_id);
  Ok(match existing {
    Err(NotFound {}) => Community::create(conn, &cf)?,
    Ok(u) => Community::update(conn, u.id, &cf)?,
    Err(e) => return Err(Error::from(e)),
  })
}

// TODO: in the future, this should only be done when an instance is followed for the first time
//       after that, we should rely in the inbox, and fetch on demand when needed
pub fn fetch_all(conn: &PgConnection) -> Result<(), Error> {
  for instance in &get_following_instances() {
    let node_info = fetch_node_info(instance)?;
    if let Some(community_list) = node_info.metadata.community_list_url {
      let communities = fetch_communities_from_instance(&community_list, conn)?;
      for c in communities {
        fetch_remote_community_posts(&c, conn)?;
      }
    } else {
      warn!(
        "{} is not a Lemmy instance, federation is not supported",
        instance.domain
      );
    }
  }
  Ok(())
}
