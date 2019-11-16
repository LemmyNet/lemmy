extern crate rss;
extern crate htmlescape;

use rss::ChannelBuilder;
use rss::ItemBuilder;
use actix_web::HttpResponse;
use actix_web::body::Body;
use crate::Settings;
use crate::db::{establish_connection, ListingType, SortType};
use crate::db::community_view::SiteView;
use crate::db::post_view::PostView;
use self::rss::Item;

pub fn get_feed() -> HttpResponse<Body> {
  let conn = establish_connection();
  let site_view = match SiteView::read(&conn) {
    Ok(site_view) => site_view,
    Err(_e) => return HttpResponse::InternalServerError().finish(),
  };

  let post = match PostView::list(&conn,
    ListingType::All,
    &SortType::New,
    None,
    None,
    None,
    None,
    None,
    true,
    false,
    false,
    None,
    None,) {
    Ok(post) => post,
    Err(_e) => return HttpResponse::InternalServerError().finish(),
  };

  let mut items: Vec<Item> = Vec::new();
  for p in post {
    let i = ItemBuilder::default()
      .title(p.name)
      .link(p.url)
      .content(p.body)
      .author(p.creator_id)
      .pub_date(p.published)
      .build()
      .unwrap();
    items.append(&i);
  }

  let channel = ChannelBuilder::default()
    .title(htmlescape::encode_minimal(&site_view.name))
    .link(format!("https://{}", Settings::get().hostname))
    .description(htmlescape::encode_minimal(&site_view.description.unwrap()))
    .pub_date("asd")
    .items(items)
    .build()
    .unwrap();
  channel.write_to(::std::io::sink()).unwrap();

  return HttpResponse::Ok()
    .content_type("application/rss+xml")
    .body(channel.to_string());
}