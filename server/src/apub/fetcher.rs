use crate::api::site::SearchResponse;
use crate::apub::*;
use crate::db::community::{Community, CommunityForm};
use crate::db::community_view::CommunityView;
use crate::db::post::{Post, PostForm};
use crate::db::post_view::PostView;
use crate::db::user::{UserForm, User_};
use crate::db::user_view::UserView;
use crate::db::{Crud, SearchType};
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

// TODO: move these to db
fn upsert_community(
  community_form: &CommunityForm,
  conn: &PgConnection,
) -> Result<Community, Error> {
  let existing = Community::read_from_actor_id(conn, &community_form.actor_id);
  match existing {
    Err(NotFound {}) => Ok(Community::create(conn, &community_form)?),
    Ok(c) => Ok(Community::update(conn, c.id, &community_form)?),
    Err(e) => Err(Error::from(e)),
  }
}
fn upsert_user(user_form: &UserForm, conn: &PgConnection) -> Result<User_, Error> {
  let existing = User_::read_from_apub_id(conn, &user_form.actor_id);
  Ok(match existing {
    Err(NotFound {}) => User_::create(conn, &user_form)?,
    Ok(u) => User_::update(conn, u.id, &user_form)?,
    Err(e) => return Err(Error::from(e)),
  })
}

fn upsert_post(post_form: &PostForm, conn: &PgConnection) -> Result<Post, Error> {
  let existing = Post::read_from_apub_id(conn, &post_form.ap_id);
  match existing {
    Err(NotFound {}) => Ok(Post::create(conn, &post_form)?),
    Ok(p) => Ok(Post::update(conn, p.id, &post_form)?),
    Err(e) => Err(Error::from(e)),
  }
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
    .map(|cf| upsert_community(&cf?, conn))
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

#[serde(untagged)]
#[derive(serde::Deserialize)]
pub enum SearchAcceptedObjects {
  Person(Box<PersonExt>),
  Group(Box<GroupExt>),
  Page(Box<Page>),
}

pub fn search_by_apub_id(query: &str, conn: &PgConnection) -> Result<SearchResponse, Error> {
  let query_url = Url::parse(&query)?;
  let mut response = SearchResponse {
    type_: SearchType::All.to_string(),
    comments: vec![],
    posts: vec![],
    communities: vec![],
    users: vec![],
  };
  // test with:
  // http://lemmy_alpha:8540/federation/c/main
  // http://lemmy_alpha:8540/federation/u/lemmy_alpha
  // http://lemmy_alpha:8540/federation/p/3
  match fetch_remote_object::<SearchAcceptedObjects>(&query_url)? {
    SearchAcceptedObjects::Person(p) => {
      let u = upsert_user(&UserForm::from_person(&p)?, conn)?;
      response.users = vec![UserView::read(conn, u.id)?];
    }
    SearchAcceptedObjects::Group(g) => {
      let c = upsert_community(&CommunityForm::from_group(&g, conn)?, conn)?;
      response.communities = vec![CommunityView::read(conn, c.id, None)?];
    }
    SearchAcceptedObjects::Page(p) => {
      let p = upsert_post(&PostForm::from_page(&p, conn)?, conn)?;
      response.posts = vec![PostView::read(conn, p.id, None)?];
    }
  }
  dbg!(&response);
  Ok(response)
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
      .map(|pf| upsert_post(&pf?, conn))
      .collect::<Result<Vec<Post>, Error>>()?,
  )
}

pub fn fetch_remote_user(apub_id: &Url, conn: &PgConnection) -> Result<User_, Error> {
  let person = fetch_remote_object::<PersonExt>(apub_id)?;
  let uf = UserForm::from_person(&person)?;
  upsert_user(&uf, conn)
}

pub fn fetch_remote_community(apub_id: &Url, conn: &PgConnection) -> Result<Community, Error> {
  let group = fetch_remote_object::<GroupExt>(apub_id)?;
  let cf = CommunityForm::from_group(&group, conn)?;
  upsert_community(&cf, conn)
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
