use actix_web::{error::ErrorBadRequest, *};
use anyhow::anyhow;
use chrono::{DateTime, NaiveDateTime, Utc};
use diesel::PgConnection;
use lemmy_api_common::utils::blocking;
use lemmy_db_schema::{
  newtypes::LocalUserId,
  source::{community::Community, local_user::LocalUser, person::Person},
  traits::{ApubActor, Crud},
  CommentSortType,
  ListingType,
  SortType,
};
use lemmy_db_views::{
  post_view::PostQuery,
  structs::{PostView, SiteView},
};
use lemmy_db_views_actor::{
  comment_reply_view::CommentReplyQuery,
  person_mention_view::PersonMentionQuery,
  structs::{CommentReplyView, PersonMentionView},
};
use lemmy_utils::{claims::Claims, error::LemmyError, utils::markdown_to_html};
use lemmy_websocket::LemmyContext;
use once_cell::sync::Lazy;
use rss::{
  extension::dublincore::DublinCoreExtensionBuilder,
  ChannelBuilder,
  GuidBuilder,
  Item,
  ItemBuilder,
};
use serde::Deserialize;
use std::{collections::BTreeMap, str::FromStr};
use strum::ParseError;

const RSS_FETCH_LIMIT: i64 = 20;

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

static RSS_NAMESPACE: Lazy<BTreeMap<String, String>> = Lazy::new(|| {
  let mut h = BTreeMap::new();
  h.insert(
    "dc".to_string(),
    rss::extension::dublincore::NAMESPACE.to_string(),
  );
  h
});

#[tracing::instrument(skip_all)]
async fn get_all_feed(
  info: web::Query<Params>,
  context: web::Data<LemmyContext>,
) -> Result<HttpResponse, Error> {
  let sort_type = get_sort_type(info).map_err(ErrorBadRequest)?;
  Ok(get_feed_data(&context, ListingType::All, sort_type).await?)
}

#[tracing::instrument(skip_all)]
async fn get_local_feed(
  info: web::Query<Params>,
  context: web::Data<LemmyContext>,
) -> Result<HttpResponse, Error> {
  let sort_type = get_sort_type(info).map_err(ErrorBadRequest)?;
  Ok(get_feed_data(&context, ListingType::Local, sort_type).await?)
}

#[tracing::instrument(skip_all)]
async fn get_feed_data(
  context: &LemmyContext,
  listing_type: ListingType,
  sort_type: SortType,
) -> Result<HttpResponse, LemmyError> {
  let site_view = blocking(context.pool(), SiteView::read_local).await??;

  let posts = blocking(context.pool(), move |conn| {
    PostQuery::builder()
      .conn(conn)
      .listing_type(Some(listing_type))
      .sort(Some(sort_type))
      .limit(Some(RSS_FETCH_LIMIT))
      .build()
      .list()
  })
  .await??;

  let items = create_post_items(posts, &context.settings().get_protocol_and_hostname())?;

  let mut channel_builder = ChannelBuilder::default();
  channel_builder
    .namespaces(RSS_NAMESPACE.to_owned())
    .title(&format!("{} - {}", site_view.site.name, listing_type))
    .link(context.settings().get_protocol_and_hostname())
    .items(items);

  if let Some(site_desc) = site_view.site.description {
    channel_builder.description(&site_desc);
  }

  let rss = channel_builder.build().to_string();
  Ok(
    HttpResponse::Ok()
      .content_type("application/rss+xml")
      .body(rss),
  )
}

#[tracing::instrument(skip_all)]
async fn get_feed(
  req: HttpRequest,
  info: web::Query<Params>,
  context: web::Data<LemmyContext>,
) -> Result<HttpResponse, Error> {
  let sort_type = get_sort_type(info).map_err(ErrorBadRequest)?;

  let req_type: String = req.match_info().get("type").unwrap_or("none").parse()?;
  let param: String = req.match_info().get("name").unwrap_or("none").parse()?;

  let request_type = match req_type.as_str() {
    "u" => RequestType::User,
    "c" => RequestType::Community,
    "front" => RequestType::Front,
    "inbox" => RequestType::Inbox,
    _ => return Err(ErrorBadRequest(LemmyError::from(anyhow!("wrong_type")))),
  };

  let jwt_secret = context.secret().jwt_secret.to_owned();
  let protocol_and_hostname = context.settings().get_protocol_and_hostname();

  let builder = blocking(context.pool(), move |conn| match request_type {
    RequestType::User => get_feed_user(conn, &sort_type, &param, &protocol_and_hostname),
    RequestType::Community => get_feed_community(conn, &sort_type, &param, &protocol_and_hostname),
    RequestType::Front => get_feed_front(
      conn,
      &jwt_secret,
      &sort_type,
      &param,
      &protocol_and_hostname,
    ),
    RequestType::Inbox => get_feed_inbox(conn, &jwt_secret, &param, &protocol_and_hostname),
  })
  .await?
  .map_err(ErrorBadRequest)?;

  let rss = builder.build().to_string();

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

#[tracing::instrument(skip_all)]
fn get_feed_user(
  conn: &PgConnection,
  sort_type: &SortType,
  user_name: &str,
  protocol_and_hostname: &str,
) -> Result<ChannelBuilder, LemmyError> {
  let site_view = SiteView::read_local(conn)?;
  let person = Person::read_from_name(conn, user_name, false)?;

  let posts = PostQuery::builder()
    .conn(conn)
    .listing_type(Some(ListingType::All))
    .sort(Some(*sort_type))
    .creator_id(Some(person.id))
    .limit(Some(RSS_FETCH_LIMIT))
    .build()
    .list()?;

  let items = create_post_items(posts, protocol_and_hostname)?;

  let mut channel_builder = ChannelBuilder::default();
  channel_builder
    .namespaces(RSS_NAMESPACE.to_owned())
    .title(&format!("{} - {}", site_view.site.name, person.name))
    .link(person.actor_id.to_string())
    .items(items);

  Ok(channel_builder)
}

#[tracing::instrument(skip_all)]
fn get_feed_community(
  conn: &PgConnection,
  sort_type: &SortType,
  community_name: &str,
  protocol_and_hostname: &str,
) -> Result<ChannelBuilder, LemmyError> {
  let site_view = SiteView::read_local(conn)?;
  let community = Community::read_from_name(conn, community_name, false)?;

  let posts = PostQuery::builder()
    .conn(conn)
    .sort(Some(*sort_type))
    .community_id(Some(community.id))
    .limit(Some(RSS_FETCH_LIMIT))
    .build()
    .list()?;

  let items = create_post_items(posts, protocol_and_hostname)?;

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

#[tracing::instrument(skip_all)]
fn get_feed_front(
  conn: &PgConnection,
  jwt_secret: &str,
  sort_type: &SortType,
  jwt: &str,
  protocol_and_hostname: &str,
) -> Result<ChannelBuilder, LemmyError> {
  let site_view = SiteView::read_local(conn)?;
  let local_user_id = LocalUserId(Claims::decode(jwt, jwt_secret)?.claims.sub);
  let local_user = LocalUser::read(conn, local_user_id)?;

  let posts = PostQuery::builder()
    .conn(conn)
    .listing_type(Some(ListingType::Subscribed))
    .my_person_id(Some(local_user.person_id))
    .show_bot_accounts(Some(local_user.show_bot_accounts))
    .show_read_posts(Some(local_user.show_read_posts))
    .sort(Some(*sort_type))
    .limit(Some(RSS_FETCH_LIMIT))
    .build()
    .list()?;

  let items = create_post_items(posts, protocol_and_hostname)?;

  let mut channel_builder = ChannelBuilder::default();
  channel_builder
    .namespaces(RSS_NAMESPACE.to_owned())
    .title(&format!("{} - Subscribed", site_view.site.name))
    .link(protocol_and_hostname)
    .items(items);

  if let Some(site_desc) = site_view.site.description {
    channel_builder.description(&site_desc);
  }

  Ok(channel_builder)
}

#[tracing::instrument(skip_all)]
fn get_feed_inbox(
  conn: &PgConnection,
  jwt_secret: &str,
  jwt: &str,
  protocol_and_hostname: &str,
) -> Result<ChannelBuilder, LemmyError> {
  let site_view = SiteView::read_local(conn)?;
  let local_user_id = LocalUserId(Claims::decode(jwt, jwt_secret)?.claims.sub);
  let local_user = LocalUser::read(conn, local_user_id)?;
  let person_id = local_user.person_id;
  let show_bot_accounts = local_user.show_bot_accounts;

  let sort = CommentSortType::New;

  let replies = CommentReplyQuery::builder()
    .conn(conn)
    .recipient_id(Some(person_id))
    .my_person_id(Some(person_id))
    .show_bot_accounts(Some(show_bot_accounts))
    .sort(Some(sort))
    .limit(Some(RSS_FETCH_LIMIT))
    .build()
    .list()?;

  let mentions = PersonMentionQuery::builder()
    .conn(conn)
    .recipient_id(Some(person_id))
    .my_person_id(Some(person_id))
    .show_bot_accounts(Some(show_bot_accounts))
    .sort(Some(sort))
    .limit(Some(RSS_FETCH_LIMIT))
    .build()
    .list()?;

  let items = create_reply_and_mention_items(replies, mentions, protocol_and_hostname)?;

  let mut channel_builder = ChannelBuilder::default();
  channel_builder
    .namespaces(RSS_NAMESPACE.to_owned())
    .title(&format!("{} - Inbox", site_view.site.name))
    .link(format!("{}/inbox", protocol_and_hostname,))
    .items(items);

  if let Some(site_desc) = site_view.site.description {
    channel_builder.description(&site_desc);
  }

  Ok(channel_builder)
}

#[tracing::instrument(skip_all)]
fn create_reply_and_mention_items(
  replies: Vec<CommentReplyView>,
  mentions: Vec<PersonMentionView>,
  protocol_and_hostname: &str,
) -> Result<Vec<Item>, LemmyError> {
  let mut reply_items: Vec<Item> = replies
    .iter()
    .map(|r| {
      let reply_url = format!(
        "{}/post/{}/comment/{}",
        protocol_and_hostname, r.post.id, r.comment.id
      );
      build_item(
        &r.creator.name,
        &r.comment.published,
        &reply_url,
        &r.comment.content,
        protocol_and_hostname,
      )
    })
    .collect::<Result<Vec<Item>, LemmyError>>()?;

  let mut mention_items: Vec<Item> = mentions
    .iter()
    .map(|m| {
      let mention_url = format!(
        "{}/post/{}/comment/{}",
        protocol_and_hostname, m.post.id, m.comment.id
      );
      build_item(
        &m.creator.name,
        &m.comment.published,
        &mention_url,
        &m.comment.content,
        protocol_and_hostname,
      )
    })
    .collect::<Result<Vec<Item>, LemmyError>>()?;

  reply_items.append(&mut mention_items);
  Ok(reply_items)
}

#[tracing::instrument(skip_all)]
fn build_item(
  creator_name: &str,
  published: &NaiveDateTime,
  url: &str,
  content: &str,
  protocol_and_hostname: &str,
) -> Result<Item, LemmyError> {
  let mut i = ItemBuilder::default();
  i.title(format!("Reply from {}", creator_name));
  let author_url = format!("{}/u/{}", protocol_and_hostname, creator_name);
  i.author(format!(
    "/u/{} <a href=\"{}\">(link)</a>",
    creator_name, author_url
  ));
  let dt = DateTime::<Utc>::from_utc(*published, Utc);
  i.pub_date(dt.to_rfc2822());
  i.comments(url.to_owned());
  let guid = GuidBuilder::default().permalink(true).value(url).build();
  i.guid(guid);
  i.link(url.to_owned());
  // TODO add images
  let html = markdown_to_html(content);
  i.description(html);
  Ok(i.build())
}

#[tracing::instrument(skip_all)]
fn create_post_items(
  posts: Vec<PostView>,
  protocol_and_hostname: &str,
) -> Result<Vec<Item>, LemmyError> {
  let mut items: Vec<Item> = Vec::new();

  for p in posts {
    let mut i = ItemBuilder::default();
    let mut dc_extension = DublinCoreExtensionBuilder::default();

    i.title(p.post.name);

    dc_extension.creators(vec![p.creator.actor_id.to_string()]);

    let dt = DateTime::<Utc>::from_utc(p.post.published, Utc);
    i.pub_date(dt.to_rfc2822());

    let post_url = format!("{}/post/{}", protocol_and_hostname, p.post.id);
    i.link(post_url.to_owned());
    i.comments(post_url.to_owned());
    let guid = GuidBuilder::default()
      .permalink(true)
      .value(&post_url)
      .build();
    i.guid(guid);

    let community_url = format!("{}/c/{}", protocol_and_hostname, p.community.name);

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

    i.dublin_core_ext(dc_extension.build());
    items.push(i.build());
  }

  Ok(items)
}
