use super::*;

pub fn config(cfg: &mut web::ServiceConfig) {
  cfg
    .route("/nodeinfo/2.0.json", web::get().to(node_info))
    .route("/.well-known/nodeinfo", web::get().to(node_info_well_known));
}

async fn node_info_well_known() -> HttpResponse<Body> {
  let node_info = NodeInfoWellKnown {
    links: NodeInfoWellKnownLinks {
      rel: "http://nodeinfo.diaspora.software/ns/schema/2.0".to_string(),
      href: format!("https://{}/nodeinfo/2.0.json", Settings::get().hostname),
    },
  };
  HttpResponse::Ok().json(node_info)
}

async fn node_info(
  db: web::Data<Pool<ConnectionManager<PgConnection>>>,
) -> Result<HttpResponse, Error> {
  let res = web::block(move || {
    let conn = db.get()?;
    let site_view = match SiteView::read(&conn) {
      Ok(site_view) => site_view,
      Err(_) => return Err(format_err!("not_found")),
    };
    let protocols = vec![];
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
  .map_err(ErrorBadRequest)?;
  Ok(res)
}

#[derive(Serialize)]
struct NodeInfoWellKnown {
  links: NodeInfoWellKnownLinks,
}

#[derive(Serialize)]
struct NodeInfoWellKnownLinks {
  rel: String,
  href: String,
}

#[derive(Serialize)]
struct NodeInfo {
  version: String,
  software: NodeInfoSoftware,
  protocols: Vec<String>,
  usage: NodeInfoUsage,
}

#[derive(Serialize)]
struct NodeInfoSoftware {
  name: String,
  version: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct NodeInfoUsage {
  users: NodeInfoUsers,
  local_posts: i64,
  local_comments: i64,
  open_registrations: bool,
}

#[derive(Serialize)]
struct NodeInfoUsers {
  total: i64,
}
