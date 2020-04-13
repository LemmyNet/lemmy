use crate::apub::fetcher::{fetch_remote_community, fetch_remote_user};
use crate::apub::{create_apub_response, make_apub_endpoint, EndpointType};
use crate::convert_datetime;
use crate::db::community::Community;
use crate::db::post::{Post, PostForm};
use crate::db::user::User_;
use crate::db::Crud;
use activitystreams::{context, object::properties::ObjectProperties, object::Page};
use actix_web::body::Body;
use actix_web::web::Path;
use actix_web::{web, HttpResponse};
use diesel::r2d2::{ConnectionManager, Pool};
use diesel::PgConnection;
use failure::Error;
use serde::Deserialize;
use url::Url;

#[derive(Deserialize)]
pub struct PostQuery {
  post_id: String,
}

pub async fn get_apub_post(
  info: Path<PostQuery>,
  db: web::Data<Pool<ConnectionManager<PgConnection>>>,
) -> Result<HttpResponse<Body>, Error> {
  let id = info.post_id.parse::<i32>()?;
  let post = Post::read(&&db.get()?, id)?;
  Ok(create_apub_response(&post.as_page(&db.get().unwrap())?))
}

impl Post {
  pub fn as_page(&self, conn: &PgConnection) -> Result<Page, Error> {
    let base_url = make_apub_endpoint(EndpointType::Post, &self.id.to_string());
    let mut page = Page::default();
    let oprops: &mut ObjectProperties = page.as_mut();
    let creator = User_::read(conn, self.creator_id)?;
    let community = Community::read(conn, self.community_id)?;

    oprops
      // Not needed when the Post is embedded in a collection (like for community outbox)
      .set_context_xsd_any_uri(context())?
      .set_id(base_url)?
      // Use summary field to be consistent with mastodon content warning.
      // https://mastodon.xyz/@Louisa/103987265222901387.json
      .set_summary_xsd_string(self.name.to_owned())?
      .set_published(convert_datetime(self.published))?
      .set_to_xsd_any_uri(community.actor_id)?
      .set_attributed_to_xsd_any_uri(make_apub_endpoint(EndpointType::User, &creator.name))?;

    if let Some(body) = &self.body {
      oprops.set_content_xsd_string(body.to_owned())?;
    }

    // TODO: hacky code because we get self.url == Some("")
    // https://github.com/dessalines/lemmy/issues/602
    let url = self.url.as_ref().filter(|u| !u.is_empty());
    if let Some(u) = url {
      oprops.set_url_xsd_any_uri(u.to_owned())?;
    }

    if let Some(u) = self.updated {
      oprops.set_updated(convert_datetime(u))?;
    }

    Ok(page)
  }
}

impl PostForm {
  pub fn from_page(page: &Page, conn: &PgConnection) -> Result<PostForm, Error> {
    let oprops = &page.object_props;
    let creator_id = Url::parse(&oprops.get_attributed_to_xsd_any_uri().unwrap().to_string())?;
    let creator = fetch_remote_user(&creator_id, conn)?;
    let community_id = Url::parse(&oprops.get_to_xsd_any_uri().unwrap().to_string())?;
    let community = fetch_remote_community(&community_id, conn)?;

    Ok(PostForm {
      name: oprops.get_summary_xsd_string().unwrap().to_string(),
      url: oprops.get_url_xsd_any_uri().map(|u| u.to_string()),
      body: oprops.get_content_xsd_string().map(|c| c.to_string()),
      creator_id: creator.id,
      community_id: community.id,
      removed: None, // -> Delete activity / tombstone
      locked: None,  // -> commentsEnabled
      published: oprops
        .get_published()
        .map(|u| u.as_ref().to_owned().naive_local()),
      updated: oprops
        .get_updated()
        .map(|u| u.as_ref().to_owned().naive_local()),
      deleted: None,     // -> Delete activity / tombstone
      nsfw: false,       // -> sensitive
      stickied: None,    // -> put it in "featured" collection of the community
      embed_title: None, // -> attachment?
      embed_description: None,
      embed_html: None,
      thumbnail_url: None,
      ap_id: oprops.get_id().unwrap().to_string(),
      local: false,
    })
  }
}
