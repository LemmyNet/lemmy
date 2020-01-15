use crate::db::site_view::SiteView;
use crate::version;
use crate::Settings;
use actix_web::body::Body;
use actix_web::web;
use actix_web::HttpResponse;
use diesel::r2d2::{ConnectionManager, Pool};
use diesel::PgConnection;
use serde_json::json;

pub fn config(cfg: &mut web::ServiceConfig) {
  cfg
    .route("/nodeinfo/2.0.json", web::get().to(node_info))
    .route("/.well-known/nodeinfo", web::get().to(node_info_well_known));
}

async fn node_info_well_known() -> HttpResponse<Body> {
  let json = json!({
    "links": {
      "rel": "http://nodeinfo.diaspora.software/ns/schema/2.0",
      "href": format!("https://{}/nodeinfo/2.0.json", Settings::get().hostname),
    }
  });

  HttpResponse::Ok().json(json.to_string())
}

async fn node_info(
  db: web::Data<Pool<ConnectionManager<PgConnection>>>,
) -> Result<HttpResponse, actix_web::Error> {
  let res = web::block(move || {
    let conn = db.get()?;
    let site_view = match SiteView::read(&conn) {
      Ok(site_view) => site_view,
      Err(_) => return Err(format_err!("not_found")),
    };
    let protocols = if Settings::get().federation_enabled {
      vec!["activitypub"]
    } else {
      vec![]
    };
    Ok(json!({
      "version": "2.0",
      "software": {
        "name": "lemmy",
        "version": version::VERSION,
      },
      "protocols": protocols,
      "usage": {
        "users": {
          "total": site_view.number_of_users
        },
        "localPosts": site_view.number_of_posts,
        "localComments": site_view.number_of_comments,
        "openRegistrations": site_view.open_registration,
      }
    }))
  })
  .await
  .map(|json| HttpResponse::Ok().json(json))
  .map_err(|_| HttpResponse::InternalServerError())?;
  Ok(res)
}
