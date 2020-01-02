extern crate rss;

use super::*;
use crate::db::comment_view::{ReplyQueryBuilder, ReplyView};
use crate::db::community::Community;
use crate::db::post_view::{PostQueryBuilder, PostView};
use crate::db::site_view::SiteView;
use crate::db::user::{Claims, User_};
use crate::db::user_mention_view::{UserMentionQueryBuilder, UserMentionView};
use crate::db::{establish_connection, ListingType, SortType};
use crate::Settings;
use actix_web::body::Body;
use actix_web::{web, HttpResponse, Result};
use chrono::{DateTime, Utc};
use failure::Error;
use rss::{CategoryBuilder, ChannelBuilder, GuidBuilder, Item, ItemBuilder};
use serde::Deserialize;
use std::str::FromStr;
use strum::ParseError;

#[derive(Deserialize)]
pub struct Params {
  sort: Option<String>,
}

enum RequestType {
  Community,
  User,
  Front,
  Inbox,
}

pub fn config(cfg: &mut web::ServiceConfig) {
  cfg
    .route("/feeds/{type}/{name}.xml", web::get().to(feeds::get_feed))
    .route("/feeds/all.xml", web::get().to(feeds::get_all_feed))
    .route("/feeds/all.xml", web::get().to(feeds::get_all_feed));
}

fn get_all_feed(info: web::Query<Params>) -> HttpResponse<Body> {
  let sort_type = match get_sort_type(info) {
    Ok(sort_type) => sort_type,
    Err(_) => return HttpResponse::BadRequest().finish(),
  };

  let feed_result = get_feed_all_data(&sort_type);

  match feed_result {
    Ok(rss) => HttpResponse::Ok()
      .content_type("application/rss+xml")
      .body(rss),
    Err(_) => HttpResponse::NotFound().finish(),
  }
}

fn get_feed(path: web::Path<(String, String)>, info: web::Query<Params>) -> HttpResponse<Body> {
  let sort_type = match get_sort_type(info) {
    Ok(sort_type) => sort_type,
    Err(_) => return HttpResponse::BadRequest().finish(),
  };

  let request_type = match path.0.as_ref() {
    "u" => RequestType::User,
    "c" => RequestType::Community,
    "front" => RequestType::Front,
    "inbox" => RequestType::Inbox,
    _ => return HttpResponse::NotFound().finish(),
  };

  let param = path.1.to_owned();

  let feed_result = match request_type {
    RequestType::User => get_feed_user(&sort_type, param),
    RequestType::Community => get_feed_community(&sort_type, param),
    RequestType::Front => get_feed_front(&sort_type, param),
    RequestType::Inbox => get_feed_inbox(param),
  };

  match feed_result {
    Ok(rss) => HttpResponse::Ok()
      .content_type("application/rss+xml")
      .body(rss),
    Err(_) => HttpResponse::NotFound().finish(),
  }
}

fn get_sort_type(info: web::Query<Params>) -> Result<SortType, ParseError> {
  let sort_query = info
    .sort
    .to_owned()
    .unwrap_or_else(|| SortType::Hot.to_string());
  SortType::from_str(&sort_query)
}

fn get_feed_all_data(sort_type: &SortType) -> Result<String, Error> {
  let conn = establish_connection();

  let site_view = SiteView::read(&conn)?;

  let posts = PostQueryBuilder::create(&conn)
    .listing_type(ListingType::All)
    .sort(sort_type)
    .list()?;

  let items = create_post_items(posts);

  let mut channel_builder = ChannelBuilder::default();
  channel_builder
    .title(&format!("{} - All", site_view.name))
    .link(format!("https://{}", Settings::get().hostname))
    .items(items);

  if let Some(site_desc) = site_view.description {
    channel_builder.description(&site_desc);
  }

  Ok(channel_builder.build().unwrap().to_string())
}

fn get_feed_user(sort_type: &SortType, user_name: String) -> Result<String, Error> {
  let conn = establish_connection();

  let site_view = SiteView::read(&conn)?;
  let user = User_::find_by_username(&conn, &user_name)?;
  let user_url = user.get_profile_url();

  let posts = PostQueryBuilder::create(&conn)
    .listing_type(ListingType::All)
    .sort(sort_type)
    .for_creator_id(user.id)
    .list()?;

  let items = create_post_items(posts);

  let mut channel_builder = ChannelBuilder::default();
  channel_builder
    .title(&format!("{} - {}", site_view.name, user.name))
    .link(user_url)
    .items(items);

  Ok(channel_builder.build().unwrap().to_string())
}

fn get_feed_community(sort_type: &SortType, community_name: String) -> Result<String, Error> {
  let conn = establish_connection();

  let site_view = SiteView::read(&conn)?;
  let community = Community::read_from_name(&conn, community_name)?;
  let community_url = community.get_url();

  let posts = PostQueryBuilder::create(&conn)
    .listing_type(ListingType::All)
    .sort(sort_type)
    .for_community_id(community.id)
    .list()?;

  let items = create_post_items(posts);

  let mut channel_builder = ChannelBuilder::default();
  channel_builder
    .title(&format!("{} - {}", site_view.name, community.name))
    .link(community_url)
    .items(items);

  if let Some(community_desc) = community.description {
    channel_builder.description(&community_desc);
  }

  Ok(channel_builder.build().unwrap().to_string())
}

fn get_feed_front(sort_type: &SortType, jwt: String) -> Result<String, Error> {
  let conn = establish_connection();

  let site_view = SiteView::read(&conn)?;
  let user_id = Claims::decode(&jwt)?.claims.id;

  let posts = PostQueryBuilder::create(&conn)
    .listing_type(ListingType::Subscribed)
    .sort(sort_type)
    .my_user_id(user_id)
    .list()?;

  let items = create_post_items(posts);

  let mut channel_builder = ChannelBuilder::default();
  channel_builder
    .title(&format!("{} - Subscribed", site_view.name))
    .link(format!("https://{}", Settings::get().hostname))
    .items(items);

  if let Some(site_desc) = site_view.description {
    channel_builder.description(&site_desc);
  }

  Ok(channel_builder.build().unwrap().to_string())
}

fn get_feed_inbox(jwt: String) -> Result<String, Error> {
  let conn = establish_connection();

  let site_view = SiteView::read(&conn)?;
  let user_id = Claims::decode(&jwt)?.claims.id;

  let sort = SortType::New;

  let replies = ReplyQueryBuilder::create(&conn, user_id)
    .sort(&sort)
    .list()?;

  let mentions = UserMentionQueryBuilder::create(&conn, user_id)
    .sort(&sort)
    .list()?;

  let items = create_reply_and_mention_items(replies, mentions);

  let mut channel_builder = ChannelBuilder::default();
  channel_builder
    .title(&format!("{} - Inbox", site_view.name))
    .link(format!("https://{}/inbox", Settings::get().hostname))
    .items(items);

  if let Some(site_desc) = site_view.description {
    channel_builder.description(&site_desc);
  }

  Ok(channel_builder.build().unwrap().to_string())
}

fn create_reply_and_mention_items(
  replies: Vec<ReplyView>,
  mentions: Vec<UserMentionView>,
) -> Vec<Item> {
  let mut items: Vec<Item> = Vec::new();

  for r in replies {
    let mut i = ItemBuilder::default();

    i.title(format!("Reply from {}", r.creator_name));

    let author_url = format!("https://{}/u/{}", Settings::get().hostname, r.creator_name);
    i.author(format!(
      "/u/{} <a href=\"{}\">(link)</a>",
      r.creator_name, author_url
    ));

    let dt = DateTime::<Utc>::from_utc(r.published, Utc);
    i.pub_date(dt.to_rfc2822());

    let reply_url = format!(
      "https://{}/post/{}/comment/{}",
      Settings::get().hostname,
      r.post_id,
      r.id
    );
    i.comments(reply_url.to_owned());
    let guid = GuidBuilder::default()
      .permalink(true)
      .value(&reply_url)
      .build();
    i.guid(guid.unwrap());

    i.link(reply_url);

    // TODO find a markdown to html parser here, do images, etc
    i.description(r.content);

    items.push(i.build().unwrap());
  }

  for m in mentions {
    let mut i = ItemBuilder::default();

    i.title(format!("Mention from {}", m.creator_name));

    let author_url = format!("https://{}/u/{}", Settings::get().hostname, m.creator_name);
    i.author(format!(
      "/u/{} <a href=\"{}\">(link)</a>",
      m.creator_name, author_url
    ));

    let dt = DateTime::<Utc>::from_utc(m.published, Utc);
    i.pub_date(dt.to_rfc2822());

    let mention_url = format!(
      "https://{}/post/{}/comment/{}",
      Settings::get().hostname,
      m.post_id,
      m.id
    );
    i.comments(mention_url.to_owned());
    let guid = GuidBuilder::default()
      .permalink(true)
      .value(&mention_url)
      .build();
    i.guid(guid.unwrap());

    i.link(mention_url);

    // TODO find a markdown to html parser here, do images, etc
    i.description(m.content);

    items.push(i.build().unwrap());
  }

  items
}

fn create_post_items(posts: Vec<PostView>) -> Vec<Item> {
  let mut items: Vec<Item> = Vec::new();

  for p in posts {
    let mut i = ItemBuilder::default();

    i.title(p.name);

    let author_url = format!("https://{}/u/{}", Settings::get().hostname, p.creator_name);
    i.author(format!(
      "/u/{} <a href=\"{}\">(link)</a>",
      p.creator_name, author_url
    ));

    let dt = DateTime::<Utc>::from_utc(p.published, Utc);
    i.pub_date(dt.to_rfc2822());

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
      .domain(Settings::get().hostname.to_owned())
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

  items
}
