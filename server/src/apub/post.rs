use crate::apub::{create_apub_response, make_apub_endpoint, EndpointType};
use crate::db::post_view::PostView;
use crate::{convert_datetime, naive_now};
use activitystreams::{object::properties::ObjectProperties, object::Page};
use actix_web::body::Body;
use actix_web::web::Path;
use actix_web::{web, HttpResponse};
use diesel::r2d2::{ConnectionManager, Pool};
use diesel::PgConnection;
use failure::Error;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct PostQuery {
  post_id: String,
}

pub async fn get_apub_post(
  info: Path<PostQuery>,
  db: web::Data<Pool<ConnectionManager<PgConnection>>>,
) -> Result<HttpResponse<Body>, Error> {
  let id = info.post_id.parse::<i32>()?;
  // TODO: shows error: missing field `user_name`
  let post = PostView::read(&&db.get()?, id, None)?;
  Ok(create_apub_response(&post.as_page()?))
}

impl PostView {
  pub fn as_page(&self) -> Result<Page, Error> {
    let base_url = make_apub_endpoint(EndpointType::Post, &self.id.to_string());
    let mut page = Page::default();
    let oprops: &mut ObjectProperties = page.as_mut();

    oprops
      // Not needed when the Post is embedded in a collection (like for community outbox)
      //.set_context_xsd_any_uri(context())?
      .set_id(base_url)?
      .set_name_xsd_string(self.name.to_owned())?
      .set_published(convert_datetime(self.published))?
      .set_attributed_to_xsd_any_uri(make_apub_endpoint(
        EndpointType::User,
        &self.creator_id.to_string(),
      ))?;

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

  pub fn from_page(page: &Page) -> Result<PostView, Error> {
    let oprops = &page.object_props;
    Ok(PostView {
      id: -1,
      name: oprops.get_name_xsd_string().unwrap().to_string(),
      url: oprops.get_url_xsd_any_uri().map(|u| u.to_string()),
      body: oprops.get_content_xsd_string().map(|c| c.to_string()),
      creator_id: -1,
      community_id: -1,
      removed: false,
      locked: false,
      published: oprops
        .get_published()
        .unwrap()
        .as_ref()
        .naive_local()
        .to_owned(),
      updated: oprops
        .get_updated()
        .map(|u| u.as_ref().to_owned().naive_local()),
      deleted: false,
      nsfw: false,
      stickied: false,
      embed_title: None,
      embed_description: None,
      embed_html: None,
      thumbnail_url: None,
      banned: false,
      banned_from_community: false,
      creator_name: "".to_string(),
      creator_avatar: None,
      community_name: "".to_string(),
      community_removed: false,
      community_deleted: false,
      community_nsfw: false,
      number_of_comments: -1,
      score: -1,
      upvotes: -1,
      downvotes: -1,
      hot_rank: -1,
      newest_activity_time: naive_now(),
      user_id: None,
      my_vote: None,
      subscribed: None,
      read: None,
      saved: None,
    })
  }
}
