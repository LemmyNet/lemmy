use super::*;
use crate::db::comment_view::{ReplyQueryBuilder, ReplyView};
use crate::db::community::Community;
use crate::db::post_view::{PostQueryBuilder, PostView};
use crate::db::site_view::SiteView;
use crate::db::user::{Claims, User_};
use crate::db::user_mention_view::{UserMentionQueryBuilder, UserMentionView};
use crate::db::{ListingType, SortType};

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
    .route("/feeds/all.xml", web::get().to(feeds::get_all_feed));
}

async fn get_all_feed(
  info: web::Query<Params>,
  db: web::Data<Pool<ConnectionManager<PgConnection>>>,
) -> Result<HttpResponse, Error> {
  let res = web::block(move || {
    let conn = db.get()?;
    get_feed_all_data(&conn, &get_sort_type(info)?)
  })
  .await
  .map(|rss| {
    HttpResponse::Ok()
      .content_type("application/rss+xml")
      .body(rss)
  })
  .map_err(ErrorBadRequest)?;
  Ok(res)
}

fn get_feed_all_data(conn: &PgConnection, sort_type: &SortType) -> Result<String, failure::Error> {
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

async fn get_feed(
  path: web::Path<(String, String)>,
  info: web::Query<Params>,
  db: web::Data<Pool<ConnectionManager<PgConnection>>>,
) -> Result<HttpResponse, Error> {
  let res = web::block(move || {
    let conn = db.get()?;

    let sort_type = get_sort_type(info)?;

    let request_type = match path.0.as_ref() {
      "u" => RequestType::User,
      "c" => RequestType::Community,
      "front" => RequestType::Front,
      "inbox" => RequestType::Inbox,
      _ => return Err(format_err!("wrong_type")),
    };

    let param = path.1.to_owned();

    match request_type {
      RequestType::User => get_feed_user(&conn, &sort_type, param),
      RequestType::Community => get_feed_community(&conn, &sort_type, param),
      RequestType::Front => get_feed_front(&conn, &sort_type, param),
      RequestType::Inbox => get_feed_inbox(&conn, param),
    }
  })
  .await
  .map(|builder| builder.build().unwrap().to_string())
  .map(|rss| {
    HttpResponse::Ok()
      .content_type("application/rss+xml")
      .body(rss)
  })
  .map_err(ErrorBadRequest)?;
  Ok(res)
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
) -> Result<ChannelBuilder, failure::Error> {
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

  Ok(channel_builder)
}

fn get_feed_community(
  conn: &PgConnection,
  sort_type: &SortType,
  community_name: String,
) -> Result<ChannelBuilder, failure::Error> {
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

  Ok(channel_builder)
}

fn get_feed_front(
  conn: &PgConnection,
  sort_type: &SortType,
  jwt: String,
) -> Result<ChannelBuilder, failure::Error> {
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

  Ok(channel_builder)
}

fn get_feed_inbox(conn: &PgConnection, jwt: String) -> Result<ChannelBuilder, failure::Error> {
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

  Ok(channel_builder)
}

fn create_reply_and_mention_items(
  replies: Vec<ReplyView>,
  mentions: Vec<UserMentionView>,
) -> Vec<Item> {
  let mut reply_items: Vec<Item> = replies
    .iter()
    .map(|r| {
      let reply_url = format!(
        "https://{}/post/{}/comment/{}",
        Settings::get().hostname,
        r.post_id,
        r.id
      );
      build_item(&r.creator_name, &r.published, &reply_url, &r.content)
    })
    .collect();

  let mut mention_items: Vec<Item> = mentions
    .iter()
    .map(|m| {
      let mention_url = format!(
        "https://{}/post/{}/comment/{}",
        Settings::get().hostname,
        m.post_id,
        m.id
      );
      build_item(&m.creator_name, &m.published, &mention_url, &m.content)
    })
    .collect();

  reply_items.append(&mut mention_items);
  reply_items
}

fn build_item(creator_name: &str, published: &NaiveDateTime, url: &str, content: &str) -> Item {
  let mut i = ItemBuilder::default();
  i.title(format!("Reply from {}", creator_name));
  let author_url = format!("https://{}/u/{}", Settings::get().hostname, creator_name);
  i.author(format!(
    "/u/{} <a href=\"{}\">(link)</a>",
    creator_name, author_url
  ));
  let dt = DateTime::<Utc>::from_utc(*published, Utc);
  i.pub_date(dt.to_rfc2822());
  i.comments(url.to_owned());
  let guid = GuidBuilder::default().permalink(true).value(url).build();
  i.guid(guid.unwrap());
  i.link(url.to_owned());
  // TODO add images
  let html = markdown_to_html(&content.to_string());
  i.description(html);
  i.build().unwrap()
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

    items.push(i.build().unwrap());
  }

  items
}
