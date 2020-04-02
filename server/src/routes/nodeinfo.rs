use crate::apub::get_apub_protocol_string;
use crate::db::site_view::SiteView;
use crate::version;
use crate::Settings;
use actix_web::body::Body;
use actix_web::web;
use actix_web::HttpResponse;
use diesel::r2d2::{ConnectionManager, Pool};
use diesel::PgConnection;
use serde::{Deserialize, Serialize};
use std::fmt::Debug;

pub fn config(cfg: &mut web::ServiceConfig) {
  cfg
    .route("/nodeinfo/2.0.json", web::get().to(node_info))
    .route("/.well-known/nodeinfo", web::get().to(node_info_well_known));
}

async fn node_info_well_known() -> HttpResponse<Body> {
  let node_info = NodeInfoWellKnown {
    links: NodeInfoWellKnownLinks {
      rel: "http://nodeinfo.diaspora.software/ns/schema/2.0".to_string(),
      href: format!(
        "{}://{}/nodeinfo/2.0.json",
        get_apub_protocol_string(),
        Settings::get().hostname
      ),
    },
  };
  HttpResponse::Ok().json(node_info)
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
    let protocols = if Settings::get().federation.enabled {
      vec!["activitypub".to_string()]
    } else {
      vec![]
    };
    Ok(NodeInfo {
      version: "2.0".to_string(),
      software: NodeInfoSoftware {
        name: "lemmy".to_string(),
        version: version::VERSION.to_string(),
      },
      protocols,
      usage: NodeInfoUsage {
        users: NodeInfoUsers {
          total: site_view.number_of_users,
        },
        local_posts: site_view.number_of_posts,
        local_comments: site_view.number_of_comments,
        open_registrations: site_view.open_registration,
      },
    })
  })
  .await
  .map(|json| HttpResponse::Ok().json(json))
  .map_err(|_| HttpResponse::InternalServerError())?;
  Ok(res)
}

#[derive(Serialize, Deserialize, Debug)]
pub struct NodeInfoWellKnown {
  pub links: NodeInfoWellKnownLinks,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct NodeInfoWellKnownLinks {
  pub rel: String,
  pub href: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct NodeInfo {
  pub version: String,
  pub software: NodeInfoSoftware,
  pub protocols: Vec<String>,
  pub usage: NodeInfoUsage,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct NodeInfoSoftware {
  pub name: String,
  pub version: String,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct NodeInfoUsage {
  pub users: NodeInfoUsers,
  pub local_posts: i64,
  pub local_comments: i64,
  pub open_registrations: bool,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct NodeInfoUsers {
  pub total: i64,
}
