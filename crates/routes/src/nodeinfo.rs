use actix_web::{error::ErrorBadRequest, web, Error, HttpResponse, Result};
use anyhow::anyhow;
use lemmy_api_common::context::LemmyContext;
use lemmy_db_schema::RegistrationMode;
use lemmy_db_views::structs::SiteView;
use lemmy_utils::{
  cache_header::{cache_1hour, cache_3days},
  error::{LemmyError, LemmyResult},
  VERSION,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use url::Url;

/// A description of the nodeinfo endpoint is here:
/// https://github.com/jhass/nodeinfo/blob/main/PROTOCOL.md
pub fn config(cfg: &mut web::ServiceConfig) {
  cfg
    .route(
      "/nodeinfo/2.1",
      web::get().to(node_info).wrap(cache_1hour()),
    )
    .service(web::redirect("/version", "/nodeinfo/2.1"))
    // For backwards compatibility, can be removed after Lemmy 0.20
    .service(web::redirect("/nodeinfo/2.0.json", "/nodeinfo/2.1"))
    .service(web::redirect("/nodeinfo/2.1.json", "/nodeinfo/2.1"))
    .route(
      "/.well-known/nodeinfo",
      web::get().to(node_info_well_known).wrap(cache_3days()),
    );
}

async fn node_info_well_known(context: web::Data<LemmyContext>) -> LemmyResult<HttpResponse> {
  let node_info = NodeInfoWellKnown {
    links: vec![NodeInfoWellKnownLinks {
      rel: Url::parse("http://nodeinfo.diaspora.software/ns/schema/2.1")?,
      href: Url::parse(&format!(
        "{}/nodeinfo/2.1",
        &context.settings().get_protocol_and_hostname(),
      ))?,
    }],
  };
  Ok(HttpResponse::Ok().json(node_info))
}

async fn node_info(context: web::Data<LemmyContext>) -> Result<HttpResponse, Error> {
  let site_view = SiteView::read_local(&mut context.pool())
    .await
    .map_err(|_| ErrorBadRequest(LemmyError::from(anyhow!("not_found"))))?
    .ok_or(ErrorBadRequest(LemmyError::from(anyhow!("not_found"))))?;

  // Since there are 3 registration options,
  // we need to set open_registrations as true if RegistrationMode is not Closed.
  let open_registrations = Some(site_view.local_site.registration_mode != RegistrationMode::Closed);
  let json = NodeInfo {
    version: Some("2.1".to_string()),
    software: Some(NodeInfoSoftware {
      name: Some("lemmy".to_string()),
      version: Some(VERSION.to_string()),
      repository: Some("https://github.com/LemmyNet/lemmy".to_string()),
      homepage: Some("https://join-lemmy.org/".to_string()),
    }),
    protocols: Some(vec!["activitypub".to_string()]),
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
    services: Some(NodeInfoServices {
      inbound: Some(vec![]),
      outbound: Some(vec![]),
    }),
    metadata: Some(HashMap::new()),
  };

  Ok(HttpResponse::Ok().json(json))
}

#[derive(Serialize, Deserialize, Debug)]
pub struct NodeInfoWellKnown {
  pub links: Vec<NodeInfoWellKnownLinks>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct NodeInfoWellKnownLinks {
  pub rel: Url,
  pub href: Url,
}

/// Nodeinfo spec: http://nodeinfo.diaspora.software/docson/index.html#/ns/schema/2.1
#[derive(Serialize, Deserialize, Debug, Default)]
#[serde(rename_all = "camelCase", default)]
pub struct NodeInfo {
  pub version: Option<String>,
  pub software: Option<NodeInfoSoftware>,
  pub protocols: Option<Vec<String>>,
  pub usage: Option<NodeInfoUsage>,
  pub open_registrations: Option<bool>,
  /// These fields are required by the spec for no reason
  pub services: Option<NodeInfoServices>,
  pub metadata: Option<HashMap<String, String>>,
}

#[derive(Serialize, Deserialize, Debug, Default)]
#[serde(default)]
pub struct NodeInfoSoftware {
  pub name: Option<String>,
  pub version: Option<String>,
  pub repository: Option<String>,
  pub homepage: Option<String>,
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

#[derive(Serialize, Deserialize, Debug, Default)]
#[serde(rename_all = "camelCase", default)]
pub struct NodeInfoServices {
  pub inbound: Option<Vec<String>>,
  pub outbound: Option<Vec<String>>,
}
