use actix_web::{error::ErrorBadRequest, web, Error, HttpRequest, HttpResponse, Result};
use chrono::{DateTime, Utc};
use lemmy_api_utils::{
  context::LemmyContext,
  utils::{check_private_instance, local_user_view_from_jwt},
};
use lemmy_db_schema::{
  source::{community::Community, person::Person},
  traits::ApubActor,
  PersonContentType,
};
use lemmy_db_schema_file::enums::{ListingType, PostSortType};
use lemmy_db_views_modlog_combined::{impls::ModlogCombinedQuery, ModlogCombinedView};
use lemmy_db_views_notification::{impls::NotificationQuery, NotificationData, NotificationView};
use lemmy_db_views_person_content_combined::impls::PersonContentCombinedQuery;
use lemmy_db_views_post::{impls::PostQuery, PostView};
use lemmy_db_views_site::SiteView;
use lemmy_utils::{
  cache_header::cache_1hour,
  error::LemmyResult,
  settings::structs::Settings,
  utils::markdown::markdown_to_html,
};
use rss::{
  extension::{dublincore::DublinCoreExtension, ExtensionBuilder, ExtensionMap},
  Category,
  Channel,
  EnclosureBuilder,
  Guid,
  Item,
};
use serde::Deserialize;
use std::{collections::BTreeMap, str::FromStr, sync::LazyLock};

const RSS_FETCH_LIMIT: i64 = 20;

#[derive(Deserialize)]
struct Params {
  sort: Option<String>,
  limit: Option<i64>,
}

impl Params {
  fn sort_type(&self) -> Result<PostSortType, Error> {
    let sort_query = self
      .sort
      .clone()
      .unwrap_or_else(|| PostSortType::Hot.to_string());
    PostSortType::from_str(&sort_query).map_err(ErrorBadRequest)
  }
  fn get_limit(&self) -> i64 {
    self.limit.unwrap_or(RSS_FETCH_LIMIT)
  }
}

pub fn config(cfg: &mut web::ServiceConfig) {
  cfg.service(
    web::scope("/feeds")
      .route("/u/{user_name}.xml", web::get().to(get_feed_user))
      .route("/c/{community_name}.xml", web::get().to(get_feed_community))
      .route("/front/{jwt}.xml", web::get().to(get_feed_front))
      .route("/modlog/{jwt}.xml", web::get().to(get_feed_modlog))
      .route("/notifications/{jwt}.xml", web::get().to(get_feed_notifs))
      // Also redirect inbox to notifications. This should probably be deprecated tho.
      .service(web::redirect(
        "/inbox/{jwt}.xml",
        "/notifications/{jwt}.xml",
      ))
      .route("/all.xml", web::get().to(get_all_feed).wrap(cache_1hour()))
      .route(
        "/local.xml",
        web::get().to(get_local_feed).wrap(cache_1hour()),
      ),
  );
}

static RSS_NAMESPACE: LazyLock<BTreeMap<String, String>> = LazyLock::new(|| {
  let mut h = BTreeMap::new();
  h.insert(
    "dc".to_string(),
    rss::extension::dublincore::NAMESPACE.to_string(),
  );
  h.insert(
    "media".to_string(),
    "http://search.yahoo.com/mrss/".to_string(),
  );
  h
});

async fn get_all_feed(
  info: web::Query<Params>,
  context: web::Data<LemmyContext>,
) -> Result<HttpResponse, Error> {
  get_feed_data(
    &context,
    ListingType::All,
    info.sort_type()?,
    info.get_limit(),
  )
  .await
}

async fn get_local_feed(
  info: web::Query<Params>,
  context: web::Data<LemmyContext>,
) -> Result<HttpResponse, Error> {
  get_feed_data(
    &context,
    ListingType::Local,
    info.sort_type()?,
    info.get_limit(),
  )
  .await
}

async fn get_feed_data(
  context: &LemmyContext,
  listing_type: ListingType,
  sort_type: PostSortType,
  limit: i64,
) -> Result<HttpResponse, Error> {
  let site_view = SiteView::read_local(&mut context.pool()).await?;

  check_private_instance(&None, &site_view.local_site)?;

  let posts = PostQuery {
    listing_type: Some(listing_type),
    sort: Some(sort_type),
    limit: Some(limit),
    ..Default::default()
  }
  .list(&site_view.site, &mut context.pool())
  .await?;

  let items = create_post_items(posts, context.settings())?;

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

  Ok(channel_to_http_res(channel))
}

async fn get_feed_user(
  info: web::Query<Params>,
  name: web::Path<String>,
  context: web::Data<LemmyContext>,
) -> Result<HttpResponse, Error> {
  let (name, domain) = split_name(name.into_inner());

  let person = if let Some(domain) = domain {
    Person::read_from_name_and_domain(&mut context.pool(), &name, &domain).await?
  } else {
    Person::read_from_name(&mut context.pool(), &name, false).await?
  }
  .ok_or(ErrorBadRequest("not_found"))?;

  let site_view = SiteView::read_local(&mut context.pool()).await?;
  check_private_instance(&None, &site_view.local_site)?;

  let content = PersonContentCombinedQuery {
    creator_id: person.id,
    type_: Some(PersonContentType::Posts),
    cursor_data: None,
    page_back: None,
    limit: Some(info.get_limit()),
    no_limit: None,
  }
  .list(&mut context.pool(), None, site_view.site.instance_id)
  .await?;

  let posts = content
    .iter()
    // Filter map to collect posts
    .filter_map(|f| f.to_post_view())
    .cloned()
    .collect::<Vec<PostView>>();

  let items = create_post_items(posts, context.settings())?;
  let channel = Channel {
    namespaces: RSS_NAMESPACE.clone(),
    title: format!("{} - {}", site_view.site.name, person.name),
    link: person.ap_id.to_string(),
    items,
    ..Default::default()
  };

  Ok(channel_to_http_res(channel))
}

/// Takes a user/community name either in the format `name` or `name@example.com`. Splits
/// it on `@` and returns a tuple of name and optional domain.
fn split_name(name: String) -> (String, Option<String>) {
  if let Some(split) = name.split_once('@') {
    (split.0.to_string(), Some(split.1.to_string()))
  } else {
    (name, None)
  }
}

async fn get_feed_community(
  info: web::Query<Params>,
  name: web::Path<String>,
  context: web::Data<LemmyContext>,
) -> Result<HttpResponse, Error> {
  let (name, domain) = split_name(name.into_inner());
  let community = if let Some(domain) = domain {
    Community::read_from_name_and_domain(&mut context.pool(), &name, &domain).await?
  } else {
    Community::read_from_name(&mut context.pool(), &name, false).await?
  }
  .ok_or(ErrorBadRequest("not_found"))?;

  if !community.visibility.can_view_without_login() {
    return Err(ErrorBadRequest("not_found"));
  }

  let site_view = SiteView::read_local(&mut context.pool()).await?;
  check_private_instance(&None, &site_view.local_site)?;

  let posts = PostQuery {
    sort: Some(info.sort_type()?),
    community_id: Some(community.id),
    limit: Some(info.get_limit()),
    ..Default::default()
  }
  .list(&site_view.site, &mut context.pool())
  .await?;

  let items = create_post_items(posts, context.settings())?;

  let mut channel = Channel {
    namespaces: RSS_NAMESPACE.clone(),
    title: format!("{} - {}", site_view.site.name, community.name),
    link: community.ap_id.to_string(),
    items,
    ..Default::default()
  };

  if let Some(community_desc) = community.description {
    channel.set_description(markdown_to_html(&community_desc));
  }

  Ok(channel_to_http_res(channel))
}

async fn get_feed_front(
  req: HttpRequest,
  info: web::Query<Params>,
  context: web::Data<LemmyContext>,
) -> Result<HttpResponse, Error> {
  let jwt: String = req.match_info().get("jwt").unwrap_or("none").parse()?;
  let site_view = SiteView::read_local(&mut context.pool()).await?;
  let local_user = local_user_view_from_jwt(&jwt, &context).await?;

  check_private_instance(&Some(local_user.clone()), &site_view.local_site)?;

  let posts = PostQuery {
    listing_type: Some(ListingType::Subscribed),
    local_user: Some(&local_user.local_user),
    sort: Some(info.sort_type()?),
    limit: Some(info.get_limit()),
    ..Default::default()
  }
  .list(&site_view.site, &mut context.pool())
  .await?;

  let protocol_and_hostname = context.settings().get_protocol_and_hostname();
  let items = create_post_items(posts, context.settings())?;
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

  Ok(channel_to_http_res(channel))
}

async fn get_feed_notifs(
  req: HttpRequest,
  _info: web::Query<Params>,
  context: web::Data<LemmyContext>,
) -> Result<HttpResponse, Error> {
  let jwt: String = req.match_info().get("jwt").unwrap_or("none").parse()?;
  let site_view = SiteView::read_local(&mut context.pool()).await?;
  let local_user = local_user_view_from_jwt(&jwt, &context).await?;
  let show_bot_accounts = Some(local_user.local_user.show_bot_accounts);

  check_private_instance(&Some(local_user.clone()), &site_view.local_site)?;

  let notifications = NotificationQuery {
    show_bot_accounts,
    ..Default::default()
  }
  .list(&mut context.pool(), &local_user.person)
  .await?;

  let protocol_and_hostname = context.settings().get_protocol_and_hostname();
  let items = create_reply_and_mention_items(notifications, &context)?;

  let mut channel = Channel {
    namespaces: RSS_NAMESPACE.clone(),
    title: format!("{} - Notifications", site_view.site.name),
    link: format!("{protocol_and_hostname}/notifications"),
    items,
    ..Default::default()
  };

  if let Some(site_desc) = site_view.site.description {
    channel.set_description(&site_desc);
  }

  Ok(channel_to_http_res(channel))
}

/// Gets your ModeratorView modlog
async fn get_feed_modlog(
  req: HttpRequest,
  _info: web::Query<Params>,
  context: web::Data<LemmyContext>,
) -> Result<HttpResponse, Error> {
  let jwt: String = req.match_info().get("jwt").unwrap_or("none").parse()?;
  let site_view = SiteView::read_local(&mut context.pool()).await?;
  let local_user = local_user_view_from_jwt(&jwt, &context).await?;
  check_private_instance(&Some(local_user.clone()), &site_view.local_site)?;

  let modlog = ModlogCombinedQuery {
    listing_type: Some(ListingType::ModeratorView),
    local_user: Some(&local_user.local_user),
    hide_modlog_names: Some(false),
    ..Default::default()
  }
  .list(&mut context.pool())
  .await?;

  let protocol_and_hostname = context.settings().get_protocol_and_hostname();
  let items = create_modlog_items(modlog, context.settings())?;

  let mut channel = Channel {
    namespaces: RSS_NAMESPACE.clone(),
    title: format!("{} - Modlog", local_user.person.name),
    link: format!("{protocol_and_hostname}/modlog"),
    items,
    ..Default::default()
  };

  if let Some(site_desc) = site_view.site.description {
    channel.set_description(&site_desc);
  }

  Ok(channel_to_http_res(channel))
}

fn create_reply_and_mention_items(
  notifs: Vec<NotificationView>,
  context: &LemmyContext,
) -> LemmyResult<Vec<Item>> {
  let reply_items: Vec<Item> = notifs
    .iter()
    .map(|v| match &v.data {
      NotificationData::Post(post) => {
        let mention_url = post.post.local_url(context.settings())?;
        build_item(
          &post.creator,
          &post.post.published_at,
          mention_url.as_str(),
          &post.post.body.clone().unwrap_or_default(),
          context.settings(),
        )
      }
      NotificationData::Comment(comment) => {
        let reply_url = comment.comment.local_url(context.settings())?;
        build_item(
          &comment.creator,
          &comment.comment.published_at,
          reply_url.as_str(),
          &comment.comment.content,
          context.settings(),
        )
      }
      NotificationData::PrivateMessage(pm) => {
        let notifs_url = format!(
          "{}/notifications",
          context.settings().get_protocol_and_hostname()
        );
        build_item(
          &pm.creator,
          &pm.private_message.published_at,
          &notifs_url,
          &pm.private_message.content,
          context.settings(),
        )
      }
    })
    .collect::<LemmyResult<Vec<Item>>>()?;

  Ok(reply_items)
}

fn create_modlog_items(
  modlog: Vec<ModlogCombinedView>,
  settings: &Settings,
) -> LemmyResult<Vec<Item>> {
  // All of these go to your modlog url
  let modlog_url = format!(
    "{}/modlog?listing_type=ModeratorView",
    settings.get_protocol_and_hostname()
  );

  let modlog_items: Vec<Item> = modlog
    .iter()
    .map(|r| match r {
      ModlogCombinedView::AdminAllowInstance(v) => build_modlog_item(
        &v.admin,
        &v.admin_allow_instance.published_at,
        &modlog_url,
        &format!(
          "Admin {} instance - {}",
          if v.admin_allow_instance.allowed {
            "allowed"
          } else {
            "disallowed"
          },
          &v.instance.domain
        ),
        Some(&v.admin_allow_instance.reason),
        settings,
      ),
      ModlogCombinedView::AdminBlockInstance(v) => build_modlog_item(
        &v.admin,
        &v.admin_block_instance.published_at,
        &modlog_url,
        &format!(
          "Admin {} instance - {}",
          if v.admin_block_instance.blocked {
            "blocked"
          } else {
            "unblocked"
          },
          &v.instance.domain
        ),
        Some(&v.admin_block_instance.reason),
        settings,
      ),
      ModlogCombinedView::AdminPurgeComment(v) => build_modlog_item(
        &v.admin,
        &v.admin_purge_comment.published_at,
        &modlog_url,
        "Admin purged comment",
        Some(&v.admin_purge_comment.reason),
        settings,
      ),
      ModlogCombinedView::AdminPurgeCommunity(v) => build_modlog_item(
        &v.admin,
        &v.admin_purge_community.published_at,
        &modlog_url,
        "Admin purged community",
        Some(&v.admin_purge_community.reason),
        settings,
      ),
      ModlogCombinedView::AdminPurgePerson(v) => build_modlog_item(
        &v.admin,
        &v.admin_purge_person.published_at,
        &modlog_url,
        "Admin purged person",
        Some(&v.admin_purge_person.reason),
        settings,
      ),
      ModlogCombinedView::AdminPurgePost(v) => build_modlog_item(
        &v.admin,
        &v.admin_purge_post.published_at,
        &modlog_url,
        "Admin purged post",
        Some(&v.admin_purge_post.reason),
        settings,
      ),
      ModlogCombinedView::AdminAdd(v) => build_modlog_item(
        &v.moderator,
        &v.admin_add.published_at,
        &modlog_url,
        &format!(
          "{} admin {}",
          removed_added_str(v.admin_add.removed),
          &v.other_person.name
        ),
        None,
        settings,
      ),
      ModlogCombinedView::ModAddToCommunity(v) => build_modlog_item(
        &v.moderator,
        &v.mod_add_to_community.published_at,
        &modlog_url,
        &format!(
          "{} mod {} to /c/{}",
          removed_added_str(v.mod_add_to_community.removed),
          &v.other_person.name,
          &v.community.name
        ),
        None,
        settings,
      ),
      ModlogCombinedView::AdminBan(v) => build_modlog_item(
        &v.moderator,
        &v.admin_ban.published_at,
        &modlog_url,
        &format!(
          "{} {}",
          banned_unbanned_str(v.admin_ban.banned),
          &v.other_person.name
        ),
        Some(&v.admin_ban.reason),
        settings,
      ),
      ModlogCombinedView::ModBanFromCommunity(v) => build_modlog_item(
        &v.moderator,
        &v.mod_ban_from_community.published_at,
        &modlog_url,
        &format!(
          "{} {} from /c/{}",
          banned_unbanned_str(v.mod_ban_from_community.banned),
          &v.other_person.name,
          &v.community.name
        ),
        Some(&v.mod_ban_from_community.reason),
        settings,
      ),
      ModlogCombinedView::ModFeaturePost(v) => build_modlog_item(
        &v.moderator,
        &v.mod_feature_post.published_at,
        &modlog_url,
        &format!(
          "{} post {}",
          if v.mod_feature_post.featured {
            "Featured"
          } else {
            "Unfeatured"
          },
          &v.post.name
        ),
        None,
        settings,
      ),
      ModlogCombinedView::ModChangeCommunityVisibility(v) => build_modlog_item(
        &v.moderator,
        &v.mod_change_community_visibility.published_at,
        &modlog_url,
        &format!(
          "Changed /c/{} visibility to {}",
          &v.community.name, &v.mod_change_community_visibility.visibility
        ),
        None,
        settings,
      ),
      ModlogCombinedView::ModLockPost(v) => build_modlog_item(
        &v.moderator,
        &v.mod_lock_post.published_at,
        &modlog_url,
        &format!(
          "{} post {}",
          if v.mod_lock_post.locked {
            "Locked"
          } else {
            "Unlocked"
          },
          &v.post.name
        ),
        Some(&v.mod_lock_post.reason),
        settings,
      ),
      ModlogCombinedView::ModRemoveComment(v) => build_modlog_item(
        &v.moderator,
        &v.mod_remove_comment.published_at,
        &modlog_url,
        &format!(
          "{} comment {}",
          removed_restored_str(v.mod_remove_comment.removed),
          &v.comment.content
        ),
        Some(&v.mod_remove_comment.reason),
        settings,
      ),
      ModlogCombinedView::AdminRemoveCommunity(v) => build_modlog_item(
        &v.moderator,
        &v.admin_remove_community.published_at,
        &modlog_url,
        &format!(
          "{} community /c/{}",
          removed_restored_str(v.admin_remove_community.removed),
          &v.community.name
        ),
        Some(&v.admin_remove_community.reason),
        settings,
      ),
      ModlogCombinedView::ModRemovePost(v) => build_modlog_item(
        &v.moderator,
        &v.mod_remove_post.published_at,
        &modlog_url,
        &format!(
          "{} post {}",
          removed_restored_str(v.mod_remove_post.removed),
          &v.post.name
        ),
        Some(&v.mod_remove_post.reason),
        settings,
      ),
      ModlogCombinedView::ModTransferCommunity(v) => build_modlog_item(
        &v.moderator,
        &v.mod_transfer_community.published_at,
        &modlog_url,
        &format!(
          "Tranferred /c/{} to /u/{}",
          &v.community.name, &v.other_person.name
        ),
        None,
        settings,
      ),
      ModlogCombinedView::ModLockComment(v) => build_modlog_item(
        &v.moderator,
        &v.mod_lock_comment.published_at,
        &modlog_url,
        &format!(
          "{} comment {}",
          if v.mod_lock_comment.locked {
            "Locked"
          } else {
            "Unlocked"
          },
          &v.comment.content
        ),
        Some(&v.mod_lock_comment.reason),
        settings,
      ),
    })
    .collect::<LemmyResult<Vec<Item>>>()?;

  Ok(modlog_items)
}

fn removed_added_str(removed: bool) -> &'static str {
  if removed {
    "Removed"
  } else {
    "Added"
  }
}

fn banned_unbanned_str(banned: bool) -> &'static str {
  if banned {
    "Banned"
  } else {
    "Unbanned"
  }
}

fn removed_restored_str(removed: bool) -> &'static str {
  if removed {
    "Removed"
  } else {
    "Restored"
  }
}

fn build_modlog_item(
  mod_: &Option<Person>,
  published: &DateTime<Utc>,
  url: &str,
  action: &str,
  reason: Option<&String>,
  settings: &Settings,
) -> LemmyResult<Item> {
  let guid = Some(Guid {
    permalink: true,
    value: action.to_owned(),
  });
  let author = if let Some(mod_) = mod_ {
    Some(format!(
      "/u/{} <a href=\"{}\">(link)</a>",
      mod_.name,
      mod_.actor_url(settings)?
    ))
  } else {
    None
  };

  Ok(Item {
    title: Some(action.to_string()),
    author,
    pub_date: Some(published.to_rfc2822()),
    link: Some(url.to_owned()),
    guid,
    description: reason.cloned(),
    ..Default::default()
  })
}

fn build_item(
  creator: &Person,
  published: &DateTime<Utc>,
  url: &str,
  content: &str,
  settings: &Settings,
) -> LemmyResult<Item> {
  // TODO add images
  let guid = Some(Guid {
    permalink: true,
    value: url.to_owned(),
  });
  let description = Some(markdown_to_html(content));

  Ok(Item {
    title: Some(format!("Reply from {}", creator.name)),
    author: Some(format!(
      "/u/{} <a href=\"{}\">(link)</a>",
      creator.name,
      creator.actor_url(settings)?
    )),
    pub_date: Some(published.to_rfc2822()),
    comments: Some(url.to_owned()),
    link: Some(url.to_owned()),
    guid,
    description,
    ..Default::default()
  })
}

fn create_post_items(posts: Vec<PostView>, settings: &Settings) -> LemmyResult<Vec<Item>> {
  let mut items: Vec<Item> = Vec::new();

  for p in posts {
    let post_url = p.post.local_url(settings)?;
    let community_url = &p.community.actor_url(settings)?;
    let dublin_core_ext = Some(DublinCoreExtension {
      creators: vec![p.creator.ap_id.to_string()],
      ..DublinCoreExtension::default()
    });
    let guid = Some(Guid {
      permalink: true,
      value: post_url.to_string(),
    });
    let mut description = format!("submitted by <a href=\"{}\">{}</a> to <a href=\"{}\">{}</a><br>{} points | <a href=\"{}\">{} comments</a>",
    p.creator.actor_url(settings)?,
    &p.creator.name,
    community_url,
    &p.community.name,
    p.post.score,
    post_url,
    p.post.comments);

    // If its a url post, add it to the description
    // and see if we can parse it as a media enclosure.
    let enclosure_opt = p.post.url.map(|url| {
      let mime_type = p
        .post
        .url_content_type
        .unwrap_or_else(|| "application/octet-stream".to_string());

      // If the url directly links to an image, wrap it in an <img> tag for display.
      let link_html = if mime_type.starts_with("image/") {
        format!("<br><a href=\"{url}\"><img src=\"{url}\"/></a>")
      } else {
        format!("<br><a href=\"{url}\">{url}</a>")
      };
      description.push_str(&link_html);

      let mut enclosure_bld = EnclosureBuilder::default();
      enclosure_bld.url(url.as_str().to_string());
      enclosure_bld.mime_type(mime_type);
      enclosure_bld.length("0".to_string());
      enclosure_bld.build()
    });

    if let Some(body) = p.post.body {
      let html = markdown_to_html(&body);
      description.push_str(&html);
    }

    let mut extensions = ExtensionMap::new();

    // If there's a thumbnail URL, add a media:content tag to display it.
    // See https://www.rssboard.org/media-rss#media-content for details.
    if let Some(url) = p.post.thumbnail_url {
      let mut thumbnail_ext = ExtensionBuilder::default();
      thumbnail_ext.name("media:content".to_string());
      thumbnail_ext.attrs(BTreeMap::from([
        ("url".to_string(), url.to_string()),
        ("medium".to_string(), "image".to_string()),
      ]));

      extensions.insert(
        "media".to_string(),
        BTreeMap::from([("content".to_string(), vec![thumbnail_ext.build()])]),
      );
    }
    let category = Category {
      name: p.community.title,
      domain: Some(p.community.ap_id.to_string()),
    };

    let i = Item {
      title: Some(p.post.name),
      pub_date: Some(p.post.published_at.to_rfc2822()),
      comments: Some(post_url.to_string()),
      guid,
      description: Some(description),
      dublin_core_ext,
      link: Some(post_url.to_string()),
      extensions,
      enclosure: enclosure_opt,
      categories: vec![category],
      ..Default::default()
    };

    items.push(i);
  }

  Ok(items)
}

fn channel_to_http_res(channel: Channel) -> HttpResponse {
  HttpResponse::Ok()
    .content_type("application/rss+xml")
    .body(channel.to_string())
}
