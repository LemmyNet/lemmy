use crate::apub::{create_apub_response, make_apub_endpoint, EndpointType};
use crate::convert_datetime;
use crate::db::post_view::PostView;
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
}
