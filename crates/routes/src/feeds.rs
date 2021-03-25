use actix_web::{error::ErrorBadRequest, *};
use anyhow::anyhow;
use chrono::{DateTime, NaiveDateTime, Utc};
use diesel::PgConnection;
use lemmy_api_common::blocking;
use lemmy_db_queries::{
  source::{community::Community_, person::Person_},
  Crud,
  ListingType,
  SortType,
};
use lemmy_db_schema::{
  source::{community::Community, local_user::LocalUser, person::Person},
  LocalUserId,
};
use lemmy_db_views::{
  comment_view::{CommentQueryBuilder, CommentView},
  post_view::{PostQueryBuilder, PostView},
  site_view::SiteView,
};
use lemmy_db_views_actor::person_mention_view::{PersonMentionQueryBuilder, PersonMentionView};
use lemmy_utils::{
  claims::Claims,
  settings::structs::Settings,
  utils::markdown_to_html,
  LemmyError,
};
use lemmy_websocket::LemmyContext;
use rss::{
  extension::dublincore::DublinCoreExtensionBuilder,
  ChannelBuilder,
  GuidBuilder,
  Item,
  ItemBuilder,
};
use serde::Deserialize;
use std::{collections::HashMap, str::FromStr};
use strum::ParseError;

#[derive(Deserialize)]
struct Params {
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
    .route("/feeds/all.xml", web::get().to(get_all_feed))
    .route("/feeds/local.xml", web::get().to(get_local_feed));
}

lazy_static! {
  static ref RSS_NAMESPACE: HashMap<String, String> = {
    let mut h = HashMap::new();
    h.insert(
      "dc".to_string(),
      rss::extension::dublincore::NAMESPACE.to_string(),
    );
    h
  };
}

async fn get_all_feed(
  info: web::Query<Params>,
  context: web::Data<LemmyContext>,
) -> Result<HttpResponse, Error> {
  let sort_type = get_sort_type(info).map_err(ErrorBadRequest)?;
  Ok(get_feed_data(&context, ListingType::All, sort_type).await?)
}

async fn get_local_feed(
  info: web::Query<Params>,
  context: web::Data<LemmyContext>,
) -> Result<HttpResponse, Error> {
  let sort_type = get_sort_type(info).map_err(ErrorBadRequest)?;
  Ok(get_feed_data(&context, ListingType::Local, sort_type).await?)
}

async fn get_feed_data(
  context: &LemmyContext,
  listing_type: ListingType,
  sort_type: SortType,
) -> Result<HttpResponse, LemmyError> {
  let site_view = blocking(context.pool(), move |conn| SiteView::read(&conn)).await??;

  let listing_type_ = listing_type.clone();
  let posts = blocking(context.pool(), move |conn| {
    PostQueryBuilder::create(&conn)
      .listing_type(&listing_type_)
      .sort(&sort_type)
      .list()
  })
  .await??;

  let items = create_post_items(posts)?;

  let mut channel_builder = ChannelBuilder::default();
  channel_builder
    .namespaces(RSS_NAMESPACE.to_owned())
    .title(&format!(
      "{} - {}",
      site_view.site.name,
      listing_type.to_string()
    ))
    .link(Settings::get().get_protocol_and_hostname())
    .items(items);

  if let Some(site_desc) = site_view.site.description {
    channel_builder.description(&site_desc);
  }

  let rss = channel_builder.build().map_err(|e| anyhow!(e))?.to_string();
  Ok(
    HttpResponse::Ok()
      .content_type("application/rss+xml")
      .body(rss),
  )
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
  let person = Person::find_by_name(&conn, &user_name)?;

  let posts = PostQueryBuilder::create(&conn)
    .listing_type(&ListingType::All)
    .sort(sort_type)
    .creator_id(person.id)
    .list()?;

  let items = create_post_items(posts)?;

  let mut channel_builder = ChannelBuilder::default();
  channel_builder
    .namespaces(RSS_NAMESPACE.to_owned())
    .title(&format!("{} - {}", site_view.site.name, person.name))
    .link(person.actor_id.to_string())
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
    .listing_type(&ListingType::All)
    .sort(sort_type)
    .community_id(community.id)
    .list()?;

  let items = create_post_items(posts)?;

  let mut channel_builder = ChannelBuilder::default();
  channel_builder
    .namespaces(RSS_NAMESPACE.to_owned())
    .title(&format!("{} - {}", site_view.site.name, community.name))
    .link(community.actor_id.to_string())
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
  let local_user_id = LocalUserId(Claims::decode(&jwt)?.claims.sub);
  let person_id = LocalUser::read(&conn, local_user_id)?.person_id;

  let posts = PostQueryBuilder::create(&conn)
    .listing_type(&ListingType::Subscribed)
    .my_person_id(person_id)
    .sort(sort_type)
    .list()?;

  let items = create_post_items(posts)?;

  let mut channel_builder = ChannelBuilder::default();
  channel_builder
    .namespaces(RSS_NAMESPACE.to_owned())
    .title(&format!("{} - Subscribed", site_view.site.name))
    .link(Settings::get().get_protocol_and_hostname())
    .items(items);

  if let Some(site_desc) = site_view.site.description {
    channel_builder.description(&site_desc);
  }

  Ok(channel_builder)
}

fn get_feed_inbox(conn: &PgConnection, jwt: String) -> Result<ChannelBuilder, LemmyError> {
  let site_view = SiteView::read(&conn)?;
  let local_user_id = LocalUserId(Claims::decode(&jwt)?.claims.sub);
  let person_id = LocalUser::read(&conn, local_user_id)?.person_id;

  let sort = SortType::New;

  let replies = CommentQueryBuilder::create(&conn)
    .recipient_id(person_id)
    .my_person_id(person_id)
    .sort(&sort)
    .list()?;

  let mentions = PersonMentionQueryBuilder::create(&conn)
    .recipient_id(person_id)
    .my_person_id(person_id)
    .sort(&sort)
    .list()?;

  let items = create_reply_and_mention_items(replies, mentions)?;

  let mut channel_builder = ChannelBuilder::default();
  channel_builder
    .namespaces(RSS_NAMESPACE.to_owned())
    .title(&format!("{} - Inbox", site_view.site.name))
    .link(format!(
      "{}/inbox",
      Settings::get().get_protocol_and_hostname()
    ))
    .items(items);

  if let Some(site_desc) = site_view.site.description {
    channel_builder.description(&site_desc);
  }

  Ok(channel_builder)
}

fn create_reply_and_mention_items(
  replies: Vec<CommentView>,
  mentions: Vec<PersonMentionView>,
) -> Result<Vec<Item>, LemmyError> {
  let mut reply_items: Vec<Item> = replies
    .iter()
    .map(|r| {
      let reply_url = format!(
        "{}/post/{}/comment/{}",
        Settings::get().get_protocol_and_hostname(),
        r.post.id,
        r.comment.id
      );
      build_item(
        &r.creator.name,
        &r.comment.published,
        &reply_url,
        &r.comment.content,
      )
    })
    .collect::<Result<Vec<Item>, LemmyError>>()?;

  let mut mention_items: Vec<Item> = mentions
    .iter()
    .map(|m| {
      let mention_url = format!(
        "{}/post/{}/comment/{}",
        Settings::get().get_protocol_and_hostname(),
        m.post.id,
        m.comment.id
      );
      build_item(
        &m.creator.name,
        &m.comment.published,
        &mention_url,
        &m.comment.content,
      )
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
    let mut dc_extension = DublinCoreExtensionBuilder::default();

    i.title(p.post.name);

    dc_extension.creators(vec![p.creator.actor_id.to_string()]);

    let dt = DateTime::<Utc>::from_utc(p.post.published, Utc);
    i.pub_date(dt.to_rfc2822());

    let post_url = format!(
      "{}/post/{}",
      Settings::get().get_protocol_and_hostname(),
      p.post.id
    );
    i.link(post_url.to_owned());
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
      p.community.name
    );

    // TODO add images
    let mut description = format!("submitted by <a href=\"{}\">{}</a> to <a href=\"{}\">{}</a><br>{} points | <a href=\"{}\">{} comments</a>",
    p.creator.actor_id,
    p.creator.name,
    community_url,
    p.community.name,
    p.counts.score,
    post_url,
    p.counts.comments);

    // If its a url post, add it to the description
    if let Some(url) = p.post.url {
      let link_html = format!("<br><a href=\"{url}\">{url}</a>", url = url);
      description.push_str(&link_html);
    }

    if let Some(body) = p.post.body {
      let html = markdown_to_html(&body);
      description.push_str(&html);
    }

    i.description(description);

    i.dublin_core_ext(dc_extension.build().map_err(|e| anyhow!(e))?);
    items.push(i.build().map_err(|e| anyhow!(e))?);
  }

  Ok(items)
}
