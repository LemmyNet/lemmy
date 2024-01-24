use crate::local_user_view_from_jwt;
use actix_web::{error::ErrorBadRequest, web, Error, HttpRequest, HttpResponse, Result};
use anyhow::anyhow;
use chrono::{DateTime, Utc};
use lemmy_api_common::{context::LemmyContext, utils::check_private_instance};
use lemmy_db_schema::{
  source::{community::Community, person::Person},
  traits::ApubActor,
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
use lemmy_utils::{
  cache_header::cache_1hour,
  error::LemmyError,
  utils::markdown::{markdown_to_html, sanitize_html},
};
use once_cell::sync::Lazy;
use rss::{extension::dublincore::DublinCoreExtension, Channel, Guid, Item};
use serde::Deserialize;
use std::{collections::BTreeMap, str::FromStr};

const RSS_FETCH_LIMIT: i64 = 20;

#[derive(Deserialize)]
struct Params {
  sort: Option<String>,
  limit: Option<i64>,
  page: Option<i64>,
}

impl Params {
  fn sort_type(&self) -> Result<SortType, Error> {
    let sort_query = self
      .sort
      .clone()
      .unwrap_or_else(|| SortType::Hot.to_string());
    SortType::from_str(&sort_query).map_err(ErrorBadRequest)
  }
  fn get_limit(&self) -> i64 {
    self.limit.unwrap_or(RSS_FETCH_LIMIT)
  }
  fn get_page(&self) -> i64 {
    self.page.unwrap_or(1)
  }
}

enum RequestType {
  Community,
  User,
  Front,
  Inbox,
}

pub fn config(cfg: &mut web::ServiceConfig) {
  cfg.service(
    web::scope("/feeds")
      .route("/{type}/{name}.xml", web::get().to(get_feed))
      .route("/all.xml", web::get().to(get_all_feed).wrap(cache_1hour()))
      .route(
        "/local.xml",
        web::get().to(get_local_feed).wrap(cache_1hour()),
      ),
  );
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
  Ok(
    get_feed_data(
      &context,
      ListingType::All,
      info.sort_type()?,
      info.get_limit(),
      info.get_page(),
    )
    .await?,
  )
}

#[tracing::instrument(skip_all)]
async fn get_local_feed(
  info: web::Query<Params>,
  context: web::Data<LemmyContext>,
) -> Result<HttpResponse, Error> {
  Ok(
    get_feed_data(
      &context,
      ListingType::Local,
      info.sort_type()?,
      info.get_limit(),
      info.get_page(),
    )
    .await?,
  )
}

#[tracing::instrument(skip_all)]
async fn get_feed_data(
  context: &LemmyContext,
  listing_type: ListingType,
  sort_type: SortType,
  limit: i64,
  page: i64,
) -> Result<HttpResponse, LemmyError> {
  let site_view = SiteView::read_local(&mut context.pool()).await?;

  check_private_instance(&None, &site_view.local_site)?;

  let posts = PostQuery {
    listing_type: (Some(listing_type)),
    sort: (Some(sort_type)),
    limit: (Some(limit)),
    page: (Some(page)),
    ..Default::default()
  }
  .list(&site_view.site, &mut context.pool())
  .await?;

  let items = create_post_items(posts, &context.settings().get_protocol_and_hostname())?;

  let mut channel = Channel {
    namespaces: RSS_NAMESPACE.clone(),
    title: format!("{} - {}", site_view.site.name, listing_type),
    link: context.settings().get_protocol_and_hostname(),
    items,
    ..Default::default()
  };

  if let Some(site_desc) = site_view.site.description {
    channel.set_description(&site_desc);
  }

  let rss = channel.to_string();
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
  let req_type: String = req.match_info().get("type").unwrap_or("none").parse()?;
  let param: String = req.match_info().get("name").unwrap_or("none").parse()?;

  let request_type = match req_type.as_str() {
    "u" => RequestType::User,
    "c" => RequestType::Community,
    "front" => RequestType::Front,
    "inbox" => RequestType::Inbox,
    _ => return Err(ErrorBadRequest(LemmyError::from(anyhow!("wrong_type")))),
  };

  let builder = match request_type {
    RequestType::User => {
      get_feed_user(
        &context,
        &info.sort_type()?,
        &info.get_limit(),
        &info.get_page(),
        &param,
      )
      .await
    }
    RequestType::Community => {
      get_feed_community(
        &context,
        &info.sort_type()?,
        &info.get_limit(),
        &info.get_page(),
        &param,
      )
      .await
    }
    RequestType::Front => {
      get_feed_front(
        &context,
        &info.sort_type()?,
        &info.get_limit(),
        &info.get_page(),
        &param,
      )
      .await
    }
    RequestType::Inbox => get_feed_inbox(&context, &param).await,
  }
  .map_err(ErrorBadRequest)?;

  let rss = builder.to_string();

  Ok(
    HttpResponse::Ok()
      .content_type("application/rss+xml")
      .body(rss),
  )
}

#[tracing::instrument(skip_all)]
async fn get_feed_user(
  context: &LemmyContext,
  sort_type: &SortType,
  limit: &i64,
  page: &i64,
  user_name: &str,
) -> Result<Channel, LemmyError> {
  let site_view = SiteView::read_local(&mut context.pool()).await?;
  let person = Person::read_from_name(&mut context.pool(), user_name, false).await?;

  check_private_instance(&None, &site_view.local_site)?;

  let posts = PostQuery {
    listing_type: (Some(ListingType::All)),
    sort: (Some(*sort_type)),
    creator_id: (Some(person.id)),
    limit: (Some(*limit)),
    page: (Some(*page)),
    ..Default::default()
  }
  .list(&site_view.site, &mut context.pool())
  .await?;

  let items = create_post_items(posts, &context.settings().get_protocol_and_hostname())?;

  let channel = Channel {
    namespaces: RSS_NAMESPACE.clone(),
    title: format!("{} - {}", site_view.site.name, person.name),
    link: person.actor_id.to_string(),
    items,
    ..Default::default()
  };

  Ok(channel)
}

#[tracing::instrument(skip_all)]
async fn get_feed_community(
  context: &LemmyContext,
  sort_type: &SortType,
  limit: &i64,
  page: &i64,
  community_name: &str,
) -> Result<Channel, LemmyError> {
  let site_view = SiteView::read_local(&mut context.pool()).await?;
  let community = Community::read_from_name(&mut context.pool(), community_name, false).await?;

  check_private_instance(&None, &site_view.local_site)?;

  let posts = PostQuery {
    sort: (Some(*sort_type)),
    community_id: (Some(community.id)),
    limit: (Some(*limit)),
    page: (Some(*page)),
    ..Default::default()
  }
  .list(&site_view.site, &mut context.pool())
  .await?;

  let items = create_post_items(posts, &context.settings().get_protocol_and_hostname())?;

  let mut channel = Channel {
    namespaces: RSS_NAMESPACE.clone(),
    title: format!("{} - {}", site_view.site.name, community.name),
    link: community.actor_id.to_string(),
    items,
    ..Default::default()
  };

  if let Some(community_desc) = community.description {
    channel.set_description(markdown_to_html(&community_desc));
  }

  Ok(channel)
}

#[tracing::instrument(skip_all)]
async fn get_feed_front(
  context: &LemmyContext,
  sort_type: &SortType,
  limit: &i64,
  page: &i64,
  jwt: &str,
) -> Result<Channel, LemmyError> {
  let site_view = SiteView::read_local(&mut context.pool()).await?;
  let local_user = local_user_view_from_jwt(jwt, context).await?;

  check_private_instance(&Some(local_user.clone()), &site_view.local_site)?;

  let posts = PostQuery {
    listing_type: (Some(ListingType::Subscribed)),
    local_user: (Some(&local_user)),
    sort: (Some(*sort_type)),
    limit: (Some(*limit)),
    page: (Some(*page)),
    ..Default::default()
  }
  .list(&site_view.site, &mut context.pool())
  .await?;

  let protocol_and_hostname = context.settings().get_protocol_and_hostname();
  let items = create_post_items(posts, &protocol_and_hostname)?;

  let mut channel = Channel {
    namespaces: RSS_NAMESPACE.clone(),
    title: format!("{} - Subscribed", site_view.site.name),
    link: protocol_and_hostname,
    items,
    ..Default::default()
  };

  if let Some(site_desc) = site_view.site.description {
    channel.set_description(markdown_to_html(&site_desc));
  }

  Ok(channel)
}

#[tracing::instrument(skip_all)]
async fn get_feed_inbox(context: &LemmyContext, jwt: &str) -> Result<Channel, LemmyError> {
  let site_view = SiteView::read_local(&mut context.pool()).await?;
  let local_user = local_user_view_from_jwt(jwt, context).await?;
  let person_id = local_user.local_user.person_id;
  let show_bot_accounts = local_user.local_user.show_bot_accounts;

  let sort = CommentSortType::New;

  check_private_instance(&Some(local_user.clone()), &site_view.local_site)?;

  let replies = CommentReplyQuery {
    recipient_id: (Some(person_id)),
    my_person_id: (Some(person_id)),
    show_bot_accounts: (show_bot_accounts),
    sort: (Some(sort)),
    limit: (Some(RSS_FETCH_LIMIT)),
    ..Default::default()
  }
  .list(&mut context.pool())
  .await?;

  let mentions = PersonMentionQuery {
    recipient_id: (Some(person_id)),
    my_person_id: (Some(person_id)),
    show_bot_accounts: (show_bot_accounts),
    sort: (Some(sort)),
    limit: (Some(RSS_FETCH_LIMIT)),
    ..Default::default()
  }
  .list(&mut context.pool())
  .await?;

  let protocol_and_hostname = context.settings().get_protocol_and_hostname();
  let items = create_reply_and_mention_items(replies, mentions, &protocol_and_hostname)?;

  let mut channel = Channel {
    namespaces: RSS_NAMESPACE.clone(),
    title: format!("{} - Inbox", site_view.site.name),
    link: format!("{protocol_and_hostname}/inbox"),
    items,
    ..Default::default()
  };

  if let Some(site_desc) = site_view.site.description {
    channel.set_description(&site_desc);
  }

  Ok(channel)
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
      let reply_url = format!("{}/comment/{}", protocol_and_hostname, r.comment.id);
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
      let mention_url = format!("{}/comment/{}", protocol_and_hostname, m.comment.id);
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
  published: &DateTime<Utc>,
  url: &str,
  content: &str,
  protocol_and_hostname: &str,
) -> Result<Item, LemmyError> {
  // TODO add images
  let author_url = format!("{protocol_and_hostname}/u/{creator_name}");
  let guid = Some(Guid {
    permalink: true,
    value: url.to_owned(),
  });
  let description = Some(markdown_to_html(content));

  Ok(Item {
    title: Some(format!("Reply from {creator_name}")),
    author: Some(format!(
      "/u/{creator_name} <a href=\"{author_url}\">(link)</a>"
    )),
    pub_date: Some(published.to_rfc2822()),
    comments: Some(url.to_owned()),
    link: Some(url.to_owned()),
    guid,
    description,
    ..Default::default()
  })
}

#[tracing::instrument(skip_all)]
fn create_post_items(
  posts: Vec<PostView>,
  protocol_and_hostname: &str,
) -> Result<Vec<Item>, LemmyError> {
  let mut items: Vec<Item> = Vec::new();

  for p in posts {
    // TODO add images
    let post_url = format!("{}/post/{}", protocol_and_hostname, p.post.id);
    let community_url = format!(
      "{}/c/{}",
      protocol_and_hostname,
      sanitize_html(&p.community.name)
    );
    let dublin_core_ext = Some(DublinCoreExtension {
      creators: vec![p.creator.actor_id.to_string()],
      ..DublinCoreExtension::default()
    });
    let guid = Some(Guid {
      permalink: true,
      value: post_url.clone(),
    });
    let mut description = format!("submitted by <a href=\"{}\">{}</a> to <a href=\"{}\">{}</a><br>{} points | <a href=\"{}\">{} comments</a>",
    p.creator.actor_id,
    sanitize_html(&p.creator.name),
    community_url,
    sanitize_html(&p.community.name),
    p.counts.score,
    post_url,
    p.counts.comments);

    // If its a url post, add it to the description
    let link = Some(if let Some(url) = p.post.url {
      let link_html = format!("<br><a href=\"{url}\">{url}</a>");
      description.push_str(&link_html);
      url.to_string()
    } else {
      post_url.clone()
    });

    if let Some(body) = p.post.body {
      let html = markdown_to_html(&body);
      description.push_str(&html);
    }

    let i = Item {
      title: Some(sanitize_html(&p.post.name)),
      pub_date: Some(p.post.published.to_rfc2822()),
      comments: Some(post_url.clone()),
      guid,
      description: Some(description),
      dublin_core_ext,
      link,
      ..Default::default()
    };

    items.push(i);
  }

  Ok(items)
}
