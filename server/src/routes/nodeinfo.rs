use crate::{blocking, routes::DbPoolParam, version, LemmyError};
use actix_web::{body::Body, error::ErrorBadRequest, *};
use anyhow::anyhow;
use lemmy_db::site_view::SiteView;
use lemmy_utils::{get_apub_protocol_string, settings::Settings};
use serde::{Deserialize, Serialize};
use url::Url;

pub fn config(cfg: &mut web::ServiceConfig) {
  cfg
    .route("/nodeinfo/2.0.json", web::get().to(node_info))
    .route("/.well-known/nodeinfo", web::get().to(node_info_well_known));
}

async fn node_info_well_known() -> Result<HttpResponse<Body>, LemmyError> {
  let node_info = NodeInfoWellKnown {
    links: NodeInfoWellKnownLinks {
      rel: Url::parse("http://nodeinfo.diaspora.software/ns/schema/2.0")?,
      href: Url::parse(&format!(
        "{}://{}/nodeinfo/2.0.json",
        get_apub_protocol_string(),
        Settings::get().hostname
      ))?,
    },
  };
  Ok(HttpResponse::Ok().json(node_info))
}

async fn node_info(db: DbPoolParam) -> Result<HttpResponse, Error> {
  let site_view = blocking(&db, SiteView::read)
    .await?
    .map_err(|_| ErrorBadRequest(LemmyError::from(anyhow!("not_found"))))?;

  let protocols = if Settings::get().federation.enabled {
    vec!["activitypub".to_string()]
  } else {
    vec![]
  };

  let json = NodeInfo {
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
  };

  Ok(HttpResponse::Ok().json(json))
}

#[derive(Serialize, Deserialize, Debug)]
pub struct NodeInfoWellKnown {
  pub links: NodeInfoWellKnownLinks,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct NodeInfoWellKnownLinks {
  pub rel: Url,
  pub href: Url,
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
