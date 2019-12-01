extern crate rss;
extern crate htmlescape;

use super::*;
use crate::Settings;
use crate::db::{establish_connection, ListingType, SortType};
use crate::db::community_view::SiteView;
use crate::db::post_view::PostView;
use crate::db::user::User_;
use crate::db::community::Community;
use actix_web::{HttpResponse, web, Result};
use actix_web::body::Body;
use rss::{ChannelBuilder, Item, ItemBuilder};
use diesel::result::Error;
use std::str::FromStr;
use self::rss::Guid;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct Params {
  sort: Option<String>,
}

pub fn get_feed(path: web::Path<(char, String)>, info: web::Query<Params>) -> HttpResponse<Body> {
  let sort_query = info.sort.clone().unwrap_or(SortType::Hot.to_string());
  let sort_type: SortType = match SortType::from_str(&sort_query) {
    Ok(sort) => sort,
    Err(_) => return HttpResponse::BadRequest().finish(),
  };

  let result = get_feed_internal(path, &sort_type);
  if result.is_ok() {
    let rss = result.unwrap();
    return HttpResponse::Ok()
      .content_type("application/rss+xml")
      .body(rss);
  } else {
    let error = result.err().unwrap();
    return match error {
      Error::NotFound => HttpResponse::NotFound().finish(),
      _ => HttpResponse::InternalServerError().finish(),
    }
  }
}

fn get_feed_internal(info: web::Path<(char, String)>, sort_type: &SortType) -> Result<String, Error> {
  let conn = establish_connection();

  let mut community_id: Option<i32> = None;
  let mut creator_id: Option<i32> = None;
  match info.0 {
    'c' =>  community_id = Some(Community::read_from_name(&conn,info.1.clone())?.id),
    'u' => creator_id = Some(User_::find_by_email_or_username(&conn,&info.1)?.id),
    _ => return Err(Error::NotFound),
  }

  let post = PostView::list(&conn,
    ListingType::All,
    sort_type,
    community_id,
    creator_id,
    None,
    None,
    None,
    true,
    false,
    false,
    None,
    None,)?;

  let mut items: Vec<Item> = Vec::new();
  for p in post {
    let dt = DateTime::<Utc>::from_utc(p.published, Utc);
    let mut i = ItemBuilder::default();
    i.title(htmlescape::encode_minimal(&p.name));
    i.pub_date(htmlescape::encode_minimal(&dt.to_rfc2822()));

    let post_url = format!("https://{}/post/{}", Settings::get().hostname, p.id);
    let mut guid = Guid::default();
    guid.set_permalink(true);
    guid.set_value(&post_url);
    i.guid(guid);
    i.comments(post_url);

    if p.url.is_some() {
      i.link(p.url.unwrap());
    }
    if p.body.is_some() {
      i.content(p.body.unwrap());
    }
    items.push(i.build().unwrap());
  }

  let site_view = SiteView::read(&conn)?;
  let mut channel_builder = ChannelBuilder::default();
  channel_builder.title(htmlescape::encode_minimal(&site_view.name))
    .link(format!("https://{}", Settings::get().hostname))
    .items(items);
  if site_view.description.is_some() {
    channel_builder.description(htmlescape::encode_minimal(&site_view.description.unwrap()));
  }
  let channel = channel_builder.build().unwrap();
  channel.write_to(::std::io::sink()).unwrap();

  return Ok(channel.to_string());
}