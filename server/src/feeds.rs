extern crate rss;
extern crate htmlescape;

use super::*;
use crate::Settings;
use crate::db::{establish_connection, ListingType, SortType, Crud};
use crate::db::community_view::SiteView;
use crate::db::post_view::PostView;
use crate::db::user::User_;
use crate::db::community::Community;
use actix_web::{HttpResponse, web, Result};
use actix_web::body::Body;
use rss::{ChannelBuilder, Item, ItemBuilder};
use diesel::result::Error;

pub fn get_feed(info: web::Path<(char, String)>) -> HttpResponse<Body> {
  return match  get_feed_internal(info) {
    Ok(body) => HttpResponse::Ok()
      .content_type("application/rss+xml")
      .body(body),
    // TODO: handle the specific type of error (403, 500, etc)
    Err(e) => HttpResponse::InternalServerError().finish(),
  }

}

fn get_feed_internal(info: web::Path<(char, String)>) -> Result<String, Error> {
  let conn = establish_connection();

  let mut community_id: Option<i32> = None;
  let mut creator_id: Option<i32> = None;
  // TODO: add a feed for /type/all
  match info.0 {
    'c' =>  community_id = Some(Community::read_from_name(&conn,info.1.clone())?.id),
    'u' => creator_id = Some(User_::find_by_email_or_username(&conn,&info.1)?.id),
    _ => return Err(Error::NotFound),
  }

  let post = PostView::list(&conn,
    ListingType::All,
    &SortType::New,
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
    // TODO: this may cause a lot of db queries
    let user = User_::read(&conn, p.creator_id)?;
    let dt = DateTime::<Utc>::from_utc(p.published, Utc);
    let mut i = ItemBuilder::default();
    i.title(htmlescape::encode_minimal(&p.name));
    i.author(htmlescape::encode_minimal(&user.name));
    i.pub_date(htmlescape::encode_minimal(&dt.to_rfc2822()));
    if p.url.is_some() {
      i.link(p.url.unwrap());
    }
    if p.body.is_some() {
      i.content(p.body.unwrap());
    }
    // TODO: any other fields?
    // https://rust-syndication.github.io/rss/rss/struct.ItemBuilder.html
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
  // TODO: any other fields?
  // https://rust-syndication.github.io/rss/rss/struct.ChannelBuilder.html
  let channel = channel_builder.build().unwrap();
  channel.write_to(::std::io::sink()).unwrap();

  return Ok(channel.to_string());
}