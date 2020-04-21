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
use activitystreams::collection::OrderedCollection;
use activitystreams::object::Page;
use activitystreams::BaseBox;
use diesel::result::Error::NotFound;
use diesel::PgConnection;
use failure::Error;
use isahc::prelude::*;
use serde::Deserialize;
use std::time::Duration;
use url::Url;

// Fetch nodeinfo metadata from a remote instance.
fn _fetch_node_info(domain: &str) -> Result<NodeInfo, Error> {
  let well_known_uri = Url::parse(&format!(
    "{}://{}/.well-known/nodeinfo",
    get_apub_protocol_string(),
    domain
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

/// Fetch any type of ActivityPub object, handling things like HTTP headers, deserialisation,
/// timeouts etc.
/// TODO: add an optional param last_updated and only fetch if its too old
pub fn fetch_remote_object<Response>(url: &Url) -> Result<Response, Error>
where
  Response: for<'de> Deserialize<'de>,
{
  if !is_apub_id_valid(&url) {
    return Err(format_err!("Activitypub uri invalid or blocked: {}", url));
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

/// The types of ActivityPub objects that can be fetched directly by searching for their ID.
#[serde(untagged)]
#[derive(serde::Deserialize, Debug)]
pub enum SearchAcceptedObjects {
  Person(Box<PersonExt>),
  Group(Box<GroupExt>),
  Page(Box<Page>),
}

/// Attempt to parse the query as URL, and fetch an ActivityPub object from it.
///
/// Some working examples for use with the docker/federation/ setup:
/// http://lemmy_alpha:8540/federation/c/main
/// http://lemmy_alpha:8540/federation/u/lemmy_alpha
/// http://lemmy_alpha:8540/federation/p/3
pub fn search_by_apub_id(query: &str, conn: &PgConnection) -> Result<SearchResponse, Error> {
  let query_url = Url::parse(&query)?;
  let mut response = SearchResponse {
    type_: SearchType::All.to_string(),
    comments: vec![],
    posts: vec![],
    communities: vec![],
    users: vec![],
  };
  match fetch_remote_object::<SearchAcceptedObjects>(&query_url)? {
    SearchAcceptedObjects::Person(p) => {
      let u = upsert_user(&UserForm::from_person(&p)?, conn)?;
      response.users = vec![UserView::read(conn, u.id)?];
    }
    SearchAcceptedObjects::Group(g) => {
      let c = upsert_community(&CommunityForm::from_group(&g, conn)?, conn)?;
      fetch_community_outbox(&c, conn)?;
      response.communities = vec![CommunityView::read(conn, c.id, None)?];
    }
    SearchAcceptedObjects::Page(p) => {
      let p = upsert_post(&PostForm::from_page(&p, conn)?, conn)?;
      response.posts = vec![PostView::read(conn, p.id, None)?];
    }
  }
  Ok(response)
}

/// Fetch all posts in the outbox of the given user, and insert them into the database.
fn fetch_community_outbox(community: &Community, conn: &PgConnection) -> Result<Vec<Post>, Error> {
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

/// Fetch a user, insert/update it in the database and return the user.
pub fn fetch_remote_user(apub_id: &Url, conn: &PgConnection) -> Result<User_, Error> {
  let person = fetch_remote_object::<PersonExt>(apub_id)?;
  let uf = UserForm::from_person(&person)?;
  upsert_user(&uf, conn)
}

/// Fetch a community, insert/update it in the database and return the community.
pub fn fetch_remote_community(apub_id: &Url, conn: &PgConnection) -> Result<Community, Error> {
  let group = fetch_remote_object::<GroupExt>(apub_id)?;
  let cf = CommunityForm::from_group(&group, conn)?;
  upsert_community(&cf, conn)
}
