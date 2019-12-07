extern crate htmlescape;
extern crate rss;

use super::*;
use crate::db::community::Community;
use crate::db::community_view::SiteView;
use crate::db::post_view::PostViewQuery;
use crate::db::user::User_;
use crate::db::{establish_connection, ListingType, SortType};
use crate::Settings;
use actix_web::body::Body;
use actix_web::{web, HttpResponse, Result};
use diesel::result::Error;
use rss::{CategoryBuilder, ChannelBuilder, GuidBuilder, Item, ItemBuilder};
use serde::Deserialize;
use std::str::FromStr;
use strum::ParseError;

#[derive(Deserialize)]
pub struct Params {
  sort: Option<String>,
}

enum RequestType {
  All,
  Community,
  User,
}

pub fn get_all_feed(info: web::Query<Params>) -> HttpResponse<Body> {
  let sort_type = match get_sort_type(info) {
    Ok(sort_type) => sort_type,
    Err(_) => return HttpResponse::BadRequest().finish(),
  };

  match get_feed_internal(&sort_type, RequestType::All, None) {
    Ok(rss) => HttpResponse::Ok()
      .content_type("application/rss+xml")
      .body(rss),
    Err(_) => HttpResponse::InternalServerError().finish(),
  }
}

pub fn get_feed(path: web::Path<(char, String)>, info: web::Query<Params>) -> HttpResponse<Body> {
  let sort_type = match get_sort_type(info) {
    Ok(sort_type) => sort_type,
    Err(_) => return HttpResponse::BadRequest().finish(),
  };

  let request_type = match path.0 {
    'u' => RequestType::User,
    'c' => RequestType::Community,
    _ => return HttpResponse::NotFound().finish(),
  };

  match get_feed_internal(&sort_type, request_type, Some(path.1.to_owned())) {
    Ok(rss) => HttpResponse::Ok()
      .content_type("application/rss+xml")
      .body(rss),
    Err(_) => HttpResponse::NotFound().finish(),
  }
}

fn get_sort_type(info: web::Query<Params>) -> Result<SortType, ParseError> {
  let sort_query = info.sort.to_owned().unwrap_or(SortType::Hot.to_string());
  SortType::from_str(&sort_query)
}

fn get_feed_internal(
  sort_type: &SortType,
  request_type: RequestType,
  name: Option<String>,
) -> Result<String, Error> {
  let conn = establish_connection();

  let mut community_id: Option<i32> = None;
  let mut creator_id: Option<i32> = None;

  let site_view = SiteView::read(&conn)?;

  let mut channel_builder = ChannelBuilder::default();

  // TODO do channel image, need to externalize

  match request_type {
    RequestType::All => {
      channel_builder
        .title(htmlescape::encode_minimal(&site_view.name))
        .link(format!("https://{}", Settings::get().hostname));

      if let Some(site_desc) = site_view.description {
        channel_builder.description(htmlescape::encode_minimal(&site_desc));
      }
    }
    RequestType::Community => {
      let community = Community::read_from_name(&conn, name.unwrap())?;
      community_id = Some(community.id);

      let community_url = format!("https://{}/c/{}", Settings::get().hostname, community.name);

      channel_builder
        .title(htmlescape::encode_minimal(&format!(
          "{} - {}",
          site_view.name, community.name
        )))
        .link(community_url);

      if let Some(community_desc) = community.description {
        channel_builder.description(htmlescape::encode_minimal(&community_desc));
      }
    }
    RequestType::User => {
      let creator = User_::find_by_email_or_username(&conn, &name.unwrap())?;
      creator_id = Some(creator.id);

      let creator_url = format!("https://{}/u/{}", Settings::get().hostname, creator.name);

      channel_builder
        .title(htmlescape::encode_minimal(&format!(
          "{} - {}",
          site_view.name, creator.name
        )))
        .link(creator_url);
    }
  }

  let posts = PostViewQuery::create(&conn, ListingType::All, sort_type, true, false, false)
    .for_community_id_optional(community_id)
    .for_creator_id_optional(creator_id)
    .list()?;

  let mut items: Vec<Item> = Vec::new();

  for p in posts {
    let mut i = ItemBuilder::default();

    i.title(htmlescape::encode_minimal(&p.name));

    let author_url = format!("https://{}/u/{}", Settings::get().hostname, p.creator_name);
    i.author(format!(
      "/u/{} <a href=\"{}\">(link)</a>",
      p.creator_name, author_url
    ));

    let dt = DateTime::<Utc>::from_utc(p.published, Utc);
    i.pub_date(htmlescape::encode_minimal(&dt.to_rfc2822()));

    let post_url = format!("https://{}/post/{}", Settings::get().hostname, p.id);
    i.comments(post_url.to_owned());
    let guid = GuidBuilder::default()
      .permalink(true)
      .value(&post_url)
      .build();
    i.guid(guid.unwrap());

    let community_url = format!(
      "https://{}/c/{}",
      Settings::get().hostname,
      p.community_name
    );

    let category = CategoryBuilder::default()
      .name(format!(
        "/c/{} <a href=\"{}\">(link)</a>",
        p.community_name, community_url
      ))
      .domain(Settings::get().hostname)
      .build();
    i.categories(vec![category.unwrap()]);

    if let Some(url) = p.url {
      i.link(url);
    }

    // TODO find a markdown to html parser here, do images, etc
    let mut description = format!("
    submitted by <a href=\"{}\">{}</a> to <a href=\"{}\">{}</a><br>{} points | <a href=\"{}\">{} comments</a>",
    author_url,
    p.creator_name,
    community_url,
    p.community_name,
    p.score,
    post_url,
    p.number_of_comments);

    if let Some(body) = p.body {
      description.push_str(&format!("<br><br>{}", body));
    }

    i.description(description);

    items.push(i.build().unwrap());
  }

  channel_builder.items(items);

  let channel = channel_builder.build().unwrap();
  channel.write_to(::std::io::sink()).unwrap();

  Ok(channel.to_string())
}
