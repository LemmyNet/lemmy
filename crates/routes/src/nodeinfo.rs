use actix_web::{error::ErrorBadRequest, web, Error, HttpResponse, Result};
use anyhow::anyhow;
use lemmy_api_common::context::LemmyContext;
use lemmy_db_schema::RegistrationMode;
use lemmy_db_views::structs::SiteView;
use lemmy_utils::{
  cache_header::{cache_1hour, cache_3days},
  error::LemmyError,
  VERSION,
};
use serde::{Deserialize, Serialize};
use url::Url;

pub fn config(cfg: &mut web::ServiceConfig) {
  cfg
    .route(
      "/nodeinfo/2.0.json",
      web::get().to(node_info).wrap(cache_1hour()),
    )
    .service(web::redirect("/version", "/nodeinfo/2.0.json"))
    .route(
      "/.well-known/nodeinfo",
      web::get().to(node_info_well_known).wrap(cache_3days()),
    );
}

async fn node_info_well_known(
  context: web::Data<LemmyContext>,
) -> Result<HttpResponse, LemmyError> {
  let node_info = NodeInfoWellKnown {
    links: vec![NodeInfoWellKnownLinks {
      rel: Url::parse("http://nodeinfo.diaspora.software/ns/schema/2.0")?,
      href: Url::parse(&format!(
        "{}/nodeinfo/2.0.json",
        &context.settings().get_protocol_and_hostname(),
      ))?,
    }],
  };
  Ok(HttpResponse::Ok().json(node_info))
}

async fn node_info(context: web::Data<LemmyContext>) -> Result<HttpResponse, Error> {
  let site_view = SiteView::read_local(&mut context.pool())
    .await
    .map_err(|_| ErrorBadRequest(LemmyError::from(anyhow!("not_found"))))?;

  let protocols = if site_view.local_site.federation_enabled {
    Some(vec!["activitypub".to_string()])
  } else {
    None
  };
  // Since there are 3 registration options,
  // we need to set open_registrations as true if RegistrationMode is not Closed.
  let open_registrations = Some(site_view.local_site.registration_mode != RegistrationMode::Closed);
  let json = NodeInfo {
    version: Some("2.0".to_string()),
    software: Some(NodeInfoSoftware {
      name: Some("lemmy".to_string()),
      version: Some(VERSION.to_string()),
    }),
    protocols,
    usage: Some(NodeInfoUsage {
      users: Some(NodeInfoUsers {
        total: Some(site_view.counts.users),
        active_halfyear: Some(site_view.counts.users_active_half_year),
        active_month: Some(site_view.counts.users_active_month),
      }),
      local_posts: Some(site_view.counts.posts),
      local_comments: Some(site_view.counts.comments),
    }),
    open_registrations,
  };

  Ok(HttpResponse::Ok().json(json))
}

#[derive(Serialize, Deserialize, Debug)]
struct NodeInfoWellKnown {
  pub links: Vec<NodeInfoWellKnownLinks>,
}

#[derive(Serialize, Deserialize, Debug)]
struct NodeInfoWellKnownLinks {
  pub rel: Url,
  pub href: Url,
}

#[derive(Serialize, Deserialize, Debug, Default)]
#[serde(rename_all = "camelCase", default)]
pub struct NodeInfo {
  pub version: Option<String>,
  pub software: Option<NodeInfoSoftware>,
  pub protocols: Option<Vec<String>>,
  pub usage: Option<NodeInfoUsage>,
  pub open_registrations: Option<bool>,
}

#[derive(Serialize, Deserialize, Debug, Default)]
#[serde(default)]
pub struct NodeInfoSoftware {
  pub name: Option<String>,
  pub version: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Default)]
#[serde(rename_all = "camelCase", default)]
pub struct NodeInfoUsage {
  pub users: Option<NodeInfoUsers>,
  pub local_posts: Option<i64>,
  pub local_comments: Option<i64>,
}

#[derive(Serialize, Deserialize, Debug, Default)]
#[serde(rename_all = "camelCase", default)]
pub struct NodeInfoUsers {
  pub total: Option<i64>,
  pub active_halfyear: Option<i64>,
  pub active_month: Option<i64>,
}
