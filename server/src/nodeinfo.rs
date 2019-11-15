use actix_web::web::Json;
use serde::Serialize;
use crate::db::establish_connection;
use crate::db::community_view::SiteView;
use actix_web::*;
use failure::Error;
use crate::version;

#[derive(Serialize)]
pub struct Software {
    name: String,
    version: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Usage {
    users: Users,
    local_posts: i64,
    local_comments: i64,
}

#[derive(Serialize)]
pub struct Users {
    total: i64,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NodeInfo {
    version: String,
    software: Software,
    protocols: [String; 0],
    usage: Usage,
    open_registrations: bool,
}

pub fn node_info() -> Result<Json<NodeInfo>, Error> {
    let conn = establish_connection();
    let site_view = match SiteView::read(&conn) {
        Ok(site_view) => site_view,
        Err(_e) => return Err(_e)?,
    };
    let json = Json(NodeInfo {
        version: String::from("2.0"),
        software: Software {
            name: String::from("lemmy"),
            version: String::from(version::VERSION),
        },
        protocols: [], // TODO: put 'activitypub' once that is implemented
        usage: Usage {
            users: Users {
                total: site_view.number_of_users,
            },
            local_posts: site_view.number_of_posts,
            local_comments: site_view.number_of_comments,
        },
        open_registrations: true });
    return Ok(json);
}