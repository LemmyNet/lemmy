use actix_web::{error::ErrorBadRequest, *};
use anyhow::anyhow;
use chrono::{DateTime, NaiveDateTime, Utc};
use diesel::PgConnection;
use lemmy_api::claims::Claims;
use lemmy_db::{
  comment_view::{ReplyQueryBuilder, ReplyView},
  community::Community,
  post_view::{PostQueryBuilder, PostView},
  site_view::SiteView,
  user::User_,
  user_mention_view::{UserMentionQueryBuilder, UserMentionView},
  ListingType,
  SortType,
};
use lemmy_structs::blocking;
use lemmy_utils::{settings::Settings, utils::markdown_to_html, LemmyError};
use lemmy_websocket::LemmyContext;
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
    .route("/feeds/{type}/{name}.xml", web::get().to(get_feed))
    .route("/feeds/all.xml", web::get().to(get_all_feed));
}

async fn get_all_feed(
  info: web::Query<Params>,
  context: web::Data<LemmyContext>,
) -> Result<HttpResponse, Error> {
  let sort_type = get_sort_type(info).map_err(ErrorBadRequest)?;

  let rss = blocking(context.pool(), move |conn| {
    get_feed_all_data(conn, &sort_type)
  })
  .await?
  .map_err(ErrorBadRequest)?;

  Ok(
    HttpResponse::Ok()
      .content_type("application/rss+xml")
      .body(rss),
  )
}

fn get_feed_all_data(conn: &PgConnection, sort_type: &SortType) -> Result<String, LemmyError> {
  let site_view = SiteView::read(&conn)?;

  let posts = PostQueryBuilder::create(&conn)
    .listing_type(ListingType::All)
    .sort(sort_type)
    .list()?;

  let items = create_post_items(posts)?;

  let mut channel_builder = ChannelBuilder::default();
  channel_builder
    .title(&format!("{} - All", site_view.name))
    .link(Settings::get().get_protocol_and_hostname())
    .items(items);

  if let Some(site_desc) = site_view.description {
    channel_builder.description(&site_desc);
  }

  Ok(channel_builder.build().map_err(|e| anyhow!(e))?.to_string())
}

async fn get_feed(
  web::Path((req_type, param)): web::Path<(String, String)>,
  info: web::Query<Params>,
  context: web::Data<LemmyContext>,
) -> Result<HttpResponse, Error> {
  let sort_type = get_sort_type(info).map_err(ErrorBadRequest)?;

  let request_type = match req_type.as_str() {
    "u" => RequestType::User,
    "c" => RequestType::Community,
    "front" => RequestType::Front,
    "inbox" => RequestType::Inbox,
    _ => return Err(ErrorBadRequest(LemmyError::from(anyhow!("wrong_type")))),
  };

  let builder = blocking(context.pool(), move |conn| match request_type {
    RequestType::User => get_feed_user(conn, &sort_type, param),
    RequestType::Community => get_feed_community(conn, &sort_type, param),
    RequestType::Front => get_feed_front(conn, &sort_type, param),
    RequestType::Inbox => get_feed_inbox(conn, param),
  })
  .await?
  .map_err(ErrorBadRequest)?;

  let rss = builder.build().map_err(ErrorBadRequest)?.to_string();

  Ok(
    HttpResponse::Ok()
      .content_type("application/rss+xml")
      .body(rss),
  )
}

fn get_sort_type(info: web::Query<Params>) -> Result<SortType, ParseError> {
  let sort_query = info
    .sort
    .to_owned()
    .unwrap_or_else(|| SortType::Hot.to_string());
  SortType::from_str(&sort_query)
}

fn get_feed_user(
  conn: &PgConnection,
  sort_type: &SortType,
  user_name: String,
) -> Result<ChannelBuilder, LemmyError> {
  let site_view = SiteView::read(&conn)?;
  let user = User_::find_by_username(&conn, &user_name)?;
  let user_url = user.get_profile_url(&Settings::get().hostname);

  let posts = PostQueryBuilder::create(&conn)
    .listing_type(ListingType::All)
    .sort(sort_type)
    .for_creator_id(user.id)
    .list()?;

  let items = create_post_items(posts)?;

  let mut channel_builder = ChannelBuilder::default();
  channel_builder
    .title(&format!("{} - {}", site_view.name, user.name))
    .link(user_url)
    .items(items);

  Ok(channel_builder)
}

fn get_feed_community(
  conn: &PgConnection,
  sort_type: &SortType,
  community_name: String,
) -> Result<ChannelBuilder, LemmyError> {
  let site_view = SiteView::read(&conn)?;
  let community = Community::read_from_name(&conn, &community_name)?;

  let posts = PostQueryBuilder::create(&conn)
    .listing_type(ListingType::All)
    .sort(sort_type)
    .for_community_id(community.id)
    .list()?;

  let items = create_post_items(posts)?;

  let mut channel_builder = ChannelBuilder::default();
  channel_builder
    .title(&format!("{} - {}", site_view.name, community.name))
    .link(community.actor_id)
    .items(items);

  if let Some(community_desc) = community.description {
    channel_builder.description(&community_desc);
  }

  Ok(channel_builder)
}

fn get_feed_front(
  conn: &PgConnection,
  sort_type: &SortType,
  jwt: String,
) -> Result<ChannelBuilder, LemmyError> {
  let site_view = SiteView::read(&conn)?;
  let user_id = Claims::decode(&jwt)?.claims.id;

  let posts = PostQueryBuilder::create(&conn)
    .listing_type(ListingType::Subscribed)
    .sort(sort_type)
    .my_user_id(user_id)
    .list()?;

  let items = create_post_items(posts)?;

  let mut channel_builder = ChannelBuilder::default();
  channel_builder
    .title(&format!("{} - Subscribed", site_view.name))
    .link(Settings::get().get_protocol_and_hostname())
    .items(items);

  if let Some(site_desc) = site_view.description {
    channel_builder.description(&site_desc);
  }

  Ok(channel_builder)
}

fn get_feed_inbox(conn: &PgConnection, jwt: String) -> Result<ChannelBuilder, LemmyError> {
  let site_view = SiteView::read(&conn)?;
  let user_id = Claims::decode(&jwt)?.claims.id;

  let sort = SortType::New;

  let replies = ReplyQueryBuilder::create(&conn, user_id)
    .sort(&sort)
    .list()?;

  let mentions = UserMentionQueryBuilder::create(&conn, user_id)
    .sort(&sort)
    .list()?;

  let items = create_reply_and_mention_items(replies, mentions)?;

  let mut channel_builder = ChannelBuilder::default();
  channel_builder
    .title(&format!("{} - Inbox", site_view.name))
    .link(format!(
      "{}/inbox",
      Settings::get().get_protocol_and_hostname()
    ))
    .items(items);

  if let Some(site_desc) = site_view.description {
    channel_builder.description(&site_desc);
  }

  Ok(channel_builder)
}

fn create_reply_and_mention_items(
  replies: Vec<ReplyView>,
  mentions: Vec<UserMentionView>,
) -> Result<Vec<Item>, LemmyError> {
  let mut reply_items: Vec<Item> = replies
    .iter()
    .map(|r| {
      let reply_url = format!(
        "{}/post/{}/comment/{}",
        Settings::get().get_protocol_and_hostname(),
        r.post_id,
        r.id
      );
      build_item(&r.creator_name, &r.published, &reply_url, &r.content)
    })
    .collect::<Result<Vec<Item>, LemmyError>>()?;

  let mut mention_items: Vec<Item> = mentions
    .iter()
    .map(|m| {
      let mention_url = format!(
        "{}/post/{}/comment/{}",
        Settings::get().get_protocol_and_hostname(),
        m.post_id,
        m.id
      );
      build_item(&m.creator_name, &m.published, &mention_url, &m.content)
    })
    .collect::<Result<Vec<Item>, LemmyError>>()?;

  reply_items.append(&mut mention_items);
  Ok(reply_items)
}

fn build_item(
  creator_name: &str,
  published: &NaiveDateTime,
  url: &str,
  content: &str,
) -> Result<Item, LemmyError> {
  let mut i = ItemBuilder::default();
  i.title(format!("Reply from {}", creator_name));
  let author_url = format!(
    "{}/u/{}",
    Settings::get().get_protocol_and_hostname(),
    creator_name
  );
  i.author(format!(
    "/u/{} <a href=\"{}\">(link)</a>",
    creator_name, author_url
  ));
  let dt = DateTime::<Utc>::from_utc(*published, Utc);
  i.pub_date(dt.to_rfc2822());
  i.comments(url.to_owned());
  let guid = GuidBuilder::default()
    .permalink(true)
    .value(url)
    .build()
    .map_err(|e| anyhow!(e))?;
  i.guid(guid);
  i.link(url.to_owned());
  // TODO add images
  let html = markdown_to_html(&content.to_string());
  i.description(html);
  Ok(i.build().map_err(|e| anyhow!(e))?)
}

fn create_post_items(posts: Vec<PostView>) -> Result<Vec<Item>, LemmyError> {
  let mut items: Vec<Item> = Vec::new();

  for p in posts {
    let mut i = ItemBuilder::default();

    i.title(p.name);

    let author_url = format!(
      "{}/u/{}",
      Settings::get().get_protocol_and_hostname(),
      p.creator_name
    );
    i.author(format!(
      "/u/{} <a href=\"{}\">(link)</a>",
      p.creator_name, author_url
    ));

    let dt = DateTime::<Utc>::from_utc(p.published, Utc);
    i.pub_date(dt.to_rfc2822());

    let post_url = format!(
      "{}/post/{}",
      Settings::get().get_protocol_and_hostname(),
      p.id
    );
    i.comments(post_url.to_owned());
    let guid = GuidBuilder::default()
      .permalink(true)
      .value(&post_url)
      .build()
      .map_err(|e| anyhow!(e))?;
    i.guid(guid);

    let community_url = format!(
      "{}/c/{}",
      Settings::get().get_protocol_and_hostname(),
      p.community_name
    );

    let category = CategoryBuilder::default()
      .name(format!(
        "/c/{} <a href=\"{}\">(link)</a>",
        p.community_name, community_url
      ))
      .domain(Settings::get().hostname.to_owned())
      .build()
      .map_err(|e| anyhow!(e))?;

    i.categories(vec![category]);

    if let Some(url) = p.url {
      i.link(url);
    }

    // TODO add images
    let mut description = format!("submitted by <a href=\"{}\">{}</a> to <a href=\"{}\">{}</a><br>{} points | <a href=\"{}\">{} comments</a>",
    author_url,
    p.creator_name,
    community_url,
    p.community_name,
    p.score,
    post_url,
    p.number_of_comments);

    if let Some(body) = p.body {
      let html = markdown_to_html(&body);
      description.push_str(&html);
    }

    i.description(description);

    items.push(i.build().map_err(|e| anyhow!(e))?);
  }

  Ok(items)
}
