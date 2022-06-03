use actix_web::{error::ErrorBadRequest, *};
use anyhow::anyhow;
use lemmy_api_common::utils::blocking;
use lemmy_db_views::structs::SiteView;
use lemmy_utils::{error::LemmyError, version};
use lemmy_websocket::LemmyContext;
use serde::{Deserialize, Serialize};
use url::Url;

pub fn config(cfg: &mut web::ServiceConfig) {
  cfg
    .route("/nodeinfo/2.0.json", web::get().to(node_info))
    .route("/.well-known/nodeinfo", web::get().to(node_info_well_known));
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
  let site_view = blocking(context.pool(), SiteView::read_local)
    .await?
    .map_err(|_| ErrorBadRequest(LemmyError::from(anyhow!("not_found"))))?;

  let protocols = if context.settings().federation.enabled {
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
        total: site_view.counts.users,
        active_halfyear: site_view.counts.users_active_half_year,
        active_month: site_view.counts.users_active_month,
      },
      local_posts: site_view.counts.posts,
      local_comments: site_view.counts.comments,
    },
    open_registrations: site_view.site.open_registration,
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

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct NodeInfo {
  pub version: String,
  pub software: NodeInfoSoftware,
  pub protocols: Vec<String>,
  pub usage: NodeInfoUsage,
  pub open_registrations: bool,
}

#[derive(Serialize, Deserialize, Debug)]
struct NodeInfoSoftware {
  pub name: String,
  pub version: String,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct NodeInfoUsage {
  pub users: NodeInfoUsers,
  pub local_posts: i64,
  pub local_comments: i64,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct NodeInfoUsers {
  pub total: i64,
  pub active_halfyear: i64,
  pub active_month: i64,
}
