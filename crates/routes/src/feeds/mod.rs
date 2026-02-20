mod negotiate_content;
use actix_web::{Error, HttpRequest, HttpResponse, Result, error::ErrorBadRequest, web};
use chrono::{DateTime, Utc};
use lemmy_api_utils::{
  context::LemmyContext,
  utils::{check_private_instance, local_user_view_from_jwt},
};
use lemmy_db_schema::{
  PersonContentType,
  source::{
    community::Community,
    multi_community::MultiCommunity,
    notification::Notification,
    person::Person,
  },
  traits::ApubActor,
};
use lemmy_db_schema_file::enums::{ListingType, ModlogKind, NotificationType, PostSortType};
use lemmy_db_views_modlog::{ModlogView, impls::ModlogQuery};
use lemmy_db_views_notification::{NotificationData, NotificationView, impls::NotificationQuery};
use lemmy_db_views_person_content_combined::impls::PersonContentCombinedQuery;
use lemmy_db_views_post::{PostView, impls::PostQuery};
use lemmy_db_views_site::SiteView;
use lemmy_email::{translations::Lang, user_language};
use lemmy_utils::{
  cache_header::cache_1hour,
  error::LemmyResult,
  settings::structs::Settings,
  utils::markdown::markdown_to_html,
};
use negotiate_content::get_lang_or_negotiate;
use rss::{
  Category,
  Channel,
  EnclosureBuilder,
  Guid,
  Item,
  extension::{ExtensionBuilder, ExtensionMap, dublincore::DublinCoreExtension},
};
use serde::Deserialize;
use std::{collections::BTreeMap, sync::LazyLock};

const RSS_FETCH_LIMIT: i64 = 20;

#[derive(Deserialize)]
struct Params {
  sort: Option<PostSortType>,
  limit: Option<i64>,
}

impl Params {
  fn sort_type(&self) -> PostSortType {
    self.sort.unwrap_or_default()
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
      .route(
        "/m/{multi_name}.xml",
        web::get().to(get_feed_multi_community),
      )
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
  req: HttpRequest,
  web::Query(info): web::Query<Params>,
  context: web::Data<LemmyContext>,
) -> Result<HttpResponse, Error> {
  let lang = get_lang_or_negotiate(&req, &context).await?;

  get_feed_data(
    &context,
    ListingType::All,
    info.sort_type(),
    info.get_limit(),
    lang,
  )
  .await
}

async fn get_local_feed(
  req: HttpRequest,
  web::Query(info): web::Query<Params>,
  context: web::Data<LemmyContext>,
) -> Result<HttpResponse, Error> {
  let lang = get_lang_or_negotiate(&req, &context).await?;

  get_feed_data(
    &context,
    ListingType::Local,
    info.sort_type(),
    info.get_limit(),
    lang,
  )
  .await
}

async fn get_feed_data(
  context: &LemmyContext,
  listing_type: ListingType,
  sort_type: PostSortType,
  limit: i64,
  lang: Lang,
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
  .await?
  .items;

  let title = format!(
    "{} - {}",
    site_view.site.name,
    if listing_type == ListingType::Local {
      lang.local()
    } else {
      lang.all()
    }
  );

  let link = context.settings().get_protocol_and_hostname();
  let items = create_post_items(posts, context.settings(), lang)?;
  Ok(send_feed_response(title, link, None, items, site_view))
}

async fn get_feed_user(
  req: HttpRequest,
  web::Query(info): web::Query<Params>,
  name: web::Path<String>,
  context: web::Data<LemmyContext>,
) -> Result<HttpResponse, Error> {
  let (name, domain) = split_name(&name);

  let person = Person::read_from_name(&mut context.pool(), name, domain, false)
    .await?
    .ok_or(ErrorBadRequest("not_found"))?;

  let site_view = SiteView::read_local(&mut context.pool()).await?;
  check_private_instance(&None, &site_view.local_site)?;
  let lang = get_lang_or_negotiate(&req, &context).await?;

  let content = PersonContentCombinedQuery {
    creator_id: person.id,
    type_: Some(PersonContentType::Posts),
    page_cursor: None,
    limit: Some(info.get_limit()),
    no_limit: None,
  }
  .list(&mut context.pool(), None, site_view.site.instance_id)
  .await?
  .items;

  let posts = content
    .iter()
    // Filter map to collect posts
    .filter_map(|f| f.to_post_view())
    .cloned()
    .collect::<Vec<PostView>>();

  let title = format!("{} - {}", site_view.site.name, person.name);
  let link = person.ap_id.to_string();
  let items = create_post_items(posts, context.settings(), lang)?;
  Ok(send_feed_response(
    title, link, person.bio, items, site_view,
  ))
}

/// Takes a user/community name either in the format `name` or `name@example.com`. Splits
/// it on `@` and returns a tuple of name and optional domain.
fn split_name(name: &str) -> (&str, Option<&str>) {
  if let Some(split) = name.split_once('@') {
    (split.0, Some(split.1))
  } else {
    (name, None)
  }
}

async fn get_feed_community(
  req: HttpRequest,
  web::Query(info): web::Query<Params>,
  name: web::Path<String>,
  context: web::Data<LemmyContext>,
) -> Result<HttpResponse, Error> {
  let (name, domain) = split_name(&name);
  let community = Community::read_from_name(&mut context.pool(), name, domain, false)
    .await?
    .ok_or(ErrorBadRequest("not_found"))?;

  if !community.visibility.can_view_without_login() {
    return Err(ErrorBadRequest("not_found"));
  }

  let site_view = SiteView::read_local(&mut context.pool()).await?;
  check_private_instance(&None, &site_view.local_site)?;
  let lang = get_lang_or_negotiate(&req, &context).await?;

  let posts = PostQuery {
    sort: Some(info.sort_type()),
    community_id: Some(community.id),
    limit: Some(info.get_limit()),
    ..Default::default()
  }
  .list(&site_view.site, &mut context.pool())
  .await?
  .items;

  let title = format!("{} - {}", site_view.site.name, community.name);
  let link = community.ap_id.to_string();
  let items = create_post_items(posts, context.settings(), lang)?;
  Ok(send_feed_response(
    title,
    link,
    community.summary,
    items,
    site_view,
  ))
}

async fn get_feed_multi_community(
  req: HttpRequest,
  web::Query(info): web::Query<Params>,
  name: web::Path<String>,
  context: web::Data<LemmyContext>,
) -> Result<HttpResponse, Error> {
  let (name, domain) = split_name(&name);
  let multi_community = MultiCommunity::read_from_name(&mut context.pool(), name, domain, false)
    .await?
    .ok_or(ErrorBadRequest("not_found"))?;

  let site_view = SiteView::read_local(&mut context.pool()).await?;
  check_private_instance(&None, &site_view.local_site)?;
  let lang = get_lang_or_negotiate(&req, &context).await?;

  let posts = PostQuery {
    sort: Some(info.sort_type()),
    multi_community_id: Some(multi_community.id),
    limit: Some(info.get_limit()),
    ..Default::default()
  }
  .list(&site_view.site, &mut context.pool())
  .await?
  .items;

  let title = format!("{} - {}", site_view.site.name, multi_community.name);
  let link = multi_community.ap_id.to_string();
  let items = create_post_items(posts, context.settings(), lang)?;
  Ok(send_feed_response(
    title,
    link,
    multi_community.summary,
    items,
    site_view,
  ))
}

async fn get_feed_front(
  req: HttpRequest,
  web::Query(info): web::Query<Params>,
  context: web::Data<LemmyContext>,
) -> Result<HttpResponse, Error> {
  let jwt: String = req.match_info().get("jwt").unwrap_or("none").parse()?;
  let site_view = SiteView::read_local(&mut context.pool()).await?;
  let local_user = local_user_view_from_jwt(&jwt, &context).await?;
  let lang = user_language(&local_user.local_user);

  check_private_instance(&Some(local_user.clone()), &site_view.local_site)?;

  let posts = PostQuery {
    listing_type: Some(ListingType::Subscribed),
    local_user: Some(&local_user.local_user),
    sort: Some(info.sort_type()),
    limit: Some(info.get_limit()),
    ..Default::default()
  }
  .list(&site_view.site, &mut context.pool())
  .await?
  .items;

  let title = format!("{} - {}", site_view.site.name, lang.subscribed());
  let link = context.settings().get_protocol_and_hostname();
  let items = create_post_items(posts, context.settings(), lang)?;
  Ok(send_feed_response(title, link, None, items, site_view))
}

fn send_feed_response(
  title: String,
  link: String,
  description: Option<String>,
  items: Vec<Item>,
  site_view: SiteView,
) -> HttpResponse {
  let mut channel = Channel {
    namespaces: RSS_NAMESPACE.clone(),
    title,
    link,
    items,
    ..Default::default()
  };

  let description = description.or(site_view.site.summary);
  if let Some(desc) = description {
    channel.set_description(markdown_to_html(&desc));
  }

  HttpResponse::Ok()
    .content_type("application/rss+xml")
    .body(channel.to_string())
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
  let lang = user_language(&local_user.local_user);

  check_private_instance(&Some(local_user.clone()), &site_view.local_site)?;

  let notifications = NotificationQuery {
    show_bot_accounts,
    ..Default::default()
  }
  .list(&mut context.pool(), &local_user.person)
  .await?
  .items;

  let protocol_and_hostname = context.settings().get_protocol_and_hostname();
  let title = format!("{} - {}", site_view.site.name, lang.notifications());
  let link = format!("{protocol_and_hostname}/notifications");
  let items = create_reply_and_mention_items(notifications, &context, lang)?;
  Ok(send_feed_response(title, link, None, items, site_view))
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
  let lang = user_language(&local_user.local_user);
  check_private_instance(&Some(local_user.clone()), &site_view.local_site)?;

  let modlog = ModlogQuery {
    listing_type: Some(ListingType::ModeratorView),
    local_user: Some(&local_user.local_user),
    hide_modlog_names: Some(false),
    ..Default::default()
  }
  .list(&mut context.pool())
  .await?
  .items;

  let protocol_and_hostname = context.settings().get_protocol_and_hostname();
  let title = format!("{} - {}", local_user.person.name, lang.modlog());
  let link = format!("{protocol_and_hostname}/modlog");
  let items = create_modlog_items(modlog, context.settings(), lang)?;
  Ok(send_feed_response(title, link, None, items, site_view))
}

fn create_reply_and_mention_items(
  notifs: Vec<NotificationView>,
  context: &LemmyContext,
  lang: Lang,
) -> LemmyResult<Vec<Item>> {
  let reply_items: Vec<Item> = notifs
    .iter()
    .flat_map(|v| {
      match &v.data {
        NotificationData::Post(post) => {
          let mention_url = post.post.local_url(context.settings()).ok()?;
          Some(build_item(
            &post.creator,
            &post.post.published_at,
            mention_url.as_str(),
            &post.post.body.clone().unwrap_or_default(),
            &v.notification,
            context.settings(),
            lang,
          ))
        }
        NotificationData::Comment(comment) => {
          let reply_url = comment.comment.local_url(context.settings()).ok()?;
          Some(build_item(
            &comment.creator,
            &comment.comment.published_at,
            reply_url.as_str(),
            &comment.comment.content,
            &v.notification,
            context.settings(),
            lang,
          ))
        }
        NotificationData::PrivateMessage(pm) => {
          let notifs_url = format!(
            "{}/notifications",
            context.settings().get_protocol_and_hostname()
          );
          Some(build_item(
            &pm.creator,
            &pm.private_message.published_at,
            &notifs_url,
            &pm.private_message.content,
            &v.notification,
            context.settings(),
            lang,
          ))
        }
        // skip modlog items
        NotificationData::ModAction(_) => None,
      }
    })
    .collect::<LemmyResult<Vec<Item>>>()?;

  Ok(reply_items)
}

fn create_modlog_items(
  modlog: Vec<ModlogView>,
  settings: &Settings,
  lang: Lang,
) -> LemmyResult<Vec<Item>> {
  // All of these go to your modlog url
  let modlog_url = format!(
    "{}/modlog?listing_type=ModeratorView",
    settings.get_protocol_and_hostname()
  );

  let modlog_items: Vec<Item> = modlog
    .iter()
    .map(|r| {
      let u = |x: Option<String>| x.unwrap_or_else(|| "unknown".to_string());
      let target_instance_domain = u(r.target_instance.as_ref().map(|i| i.domain.clone()));
      let target_person_name = u(r.target_person.as_ref().map(|i| i.name.clone()));
      let target_community_name = u(r.target_community.as_ref().map(|i| i.name.clone()));
      let target_post_name = u(r.target_post.as_ref().map(|i| i.name.clone()));
      let target_comment_content = u(r.target_comment.as_ref().map(|i| i.content.clone()));
      match r.modlog.kind {
        ModlogKind::AdminAllowInstance => build_modlog_item(
          r,
          &modlog_url,
          if r.modlog.is_revert {
            lang.admin_disallowed_instance_x(&target_instance_domain)
          } else {
            lang.admin_allowed_instance_x(&target_instance_domain)
          },
          settings,
        ),
        ModlogKind::AdminBlockInstance => build_modlog_item(
          r,
          &modlog_url,
          if r.modlog.is_revert {
            lang.admin_unblocked_instance_x(&target_instance_domain)
          } else {
            lang.admin_blocked_instance_x(&target_instance_domain)
          },
          settings,
        ),
        ModlogKind::AdminPurgeComment => {
          build_modlog_item(r, &modlog_url, lang.admin_purged_comment(), settings)
        }
        ModlogKind::AdminPurgeCommunity => {
          build_modlog_item(r, &modlog_url, lang.admin_purged_community(), settings)
        }
        ModlogKind::AdminPurgePerson => {
          build_modlog_item(r, &modlog_url, lang.admin_purged_person(), settings)
        }
        ModlogKind::AdminPurgePost => {
          build_modlog_item(r, &modlog_url, lang.admin_purged_post(), settings)
        }
        ModlogKind::AdminAdd => build_modlog_item(
          r,
          &modlog_url,
          if r.modlog.is_revert {
            lang.added_admin_x(&target_person_name)
          } else {
            lang.removed_admin_x(&target_person_name)
          },
          settings,
        ),
        ModlogKind::ModAddToCommunity => build_modlog_item(
          r,
          &modlog_url,
          if r.modlog.is_revert {
            lang.added_mod_x_to_community_y(&target_community_name, &target_person_name)
          } else {
            lang.removed_mod_x_from_community_y(&target_community_name, &target_person_name)
          },
          settings,
        ),
        ModlogKind::AdminBan => build_modlog_item(
          r,
          &modlog_url,
          if r.modlog.is_revert {
            lang.unbanned_user_x(&target_person_name)
          } else {
            lang.banned_user_x(&target_person_name)
          },
          settings,
        ),
        ModlogKind::ModBanFromCommunity => build_modlog_item(
          r,
          &modlog_url,
          if r.modlog.is_revert {
            lang.unbanned_user_x_from_community_y(&target_community_name, &target_person_name)
          } else {
            lang.banned_user_x_from_community_y(&target_community_name, &target_person_name)
          },
          settings,
        ),
        ModlogKind::ModFeaturePostCommunity => build_modlog_item(
          r,
          &modlog_url,
          if r.modlog.is_revert {
            lang.featured_post_x(&target_post_name)
          } else {
            lang.unfeatured_post_x(&target_post_name)
          },
          settings,
        ),
        ModlogKind::AdminFeaturePostSite => build_modlog_item(
          r,
          &modlog_url,
          if r.modlog.is_revert {
            lang.featured_post_x(&target_post_name)
          } else {
            lang.unfeatured_post_x(&target_post_name)
          },
          settings,
        ),
        ModlogKind::ModChangeCommunityVisibility => build_modlog_item(
          r,
          &modlog_url,
          lang.changed_community_x_visibility(&target_community_name),
          settings,
        ),
        ModlogKind::ModLockPost => build_modlog_item(
          r,
          &modlog_url,
          if r.modlog.is_revert {
            lang.unlocked_post_x(&target_post_name)
          } else {
            lang.locked_post_x(&target_post_name)
          },
          settings,
        ),
        ModlogKind::ModRemoveComment => build_modlog_item(
          r,
          &modlog_url,
          if r.modlog.is_revert {
            lang.restored_comment_x(&target_comment_content)
          } else {
            lang.removed_comment_x(&target_comment_content)
          },
          settings,
        ),
        ModlogKind::AdminRemoveCommunity => build_modlog_item(
          r,
          &modlog_url,
          if r.modlog.is_revert {
            lang.restored_community_x(&target_community_name)
          } else {
            lang.removed_community_x(&target_community_name)
          },
          settings,
        ),
        ModlogKind::ModRemovePost => build_modlog_item(
          r,
          &modlog_url,
          if r.modlog.is_revert {
            lang.restored_post_x(&target_post_name)
          } else {
            lang.removed_post_x(&target_post_name)
          },
          settings,
        ),
        ModlogKind::ModTransferCommunity => build_modlog_item(
          r,
          &modlog_url,
          lang.transferred_community_x_to_user_y(&target_community_name, &target_person_name),
          settings,
        ),
        ModlogKind::ModLockComment => build_modlog_item(
          r,
          &modlog_url,
          if r.modlog.is_revert {
            lang.unlocked_comment_x(&target_comment_content)
          } else {
            lang.locked_comment_x(&target_comment_content)
          },
          settings,
        ),
        ModlogKind::ModWarnComment => build_modlog_item(
          r,
          &modlog_url,
          format!(
            "Warned user {} about comment {}",
            &&target_person_name, &&target_comment_content
          ),
          settings,
        ),
        ModlogKind::ModWarnPost => build_modlog_item(
          r,
          &modlog_url,
          format!(
            "Warned user {} about post {}",
            &&target_person_name, &&target_post_name
          ),
          settings,
        ),
      }
    })
    .collect::<LemmyResult<Vec<Item>>>()?;

  Ok(modlog_items)
}

fn build_modlog_item<T: Into<String>>(
  view: &ModlogView,
  url: &str,
  action: T,
  settings: &Settings,
) -> LemmyResult<Item> {
  let guid = Some(Guid {
    permalink: true,
    value: view.modlog.id.0.to_string(),
  });
  let author = if let Some(mod_) = &view.moderator {
    Some(format!(
      "/u/{} <a href=\"{}\">(link)</a>",
      mod_.name,
      mod_.actor_url(settings)?
    ))
  } else {
    None
  };

  Ok(Item {
    title: Some(action.into()),
    author,
    pub_date: Some(view.modlog.published_at.to_rfc2822()),
    link: Some(url.to_owned()),
    guid,
    description: view.modlog.reason.clone(),
    ..Default::default()
  })
}

fn build_item(
  creator: &Person,
  published: &DateTime<Utc>,
  url: &str,
  content: &str,
  notification: &Notification,
  settings: &Settings,
  lang: Lang,
) -> LemmyResult<Item> {
  // TODO add images
  let guid = Some(Guid {
    permalink: true,
    value: url.to_owned(),
  });
  let description = Some(markdown_to_html(content));

  let title = match notification.kind {
    NotificationType::Mention => lang.mention_from_x(creator.name.clone()),
    NotificationType::Reply => lang.reply_from_x(creator.name.clone()),
    NotificationType::Subscribed => lang.subscribed().to_string(),
    NotificationType::PrivateMessage => lang.private_message_from_x(creator.name.clone()),
    NotificationType::ModAction => lang.mod_action().to_string(),
  };
  Ok(Item {
    title: Some(title),
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

fn create_post_items(
  posts: Vec<PostView>,
  settings: &Settings,
  lang: Lang,
) -> LemmyResult<Vec<Item>> {
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
    let mut description = lang.submitted_post_with_meta_info(
      p.creator.actor_url(settings)?,
      &p.community.name,
      community_url,
      &p.creator.name,
      p.post.comments,
      p.post.score,
      &post_url,
    );

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
