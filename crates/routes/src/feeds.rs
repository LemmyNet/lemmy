use actix_web::{error::ErrorBadRequest, web, Error, HttpRequest, HttpResponse, Result};
use anyhow::anyhow;
use chrono::{DateTime, Utc};
use lemmy_api_common::{
  context::LemmyContext,
  utils::{check_private_instance, local_user_view_from_jwt},
};
use lemmy_db_schema::{
  source::{community::Community, person::Person},
  traits::ApubActor,
  PersonContentType,
};
use lemmy_db_schema_file::enums::{ListingType, PostSortType};
use lemmy_db_views_inbox_combined::{impls::InboxCombinedQuery, InboxCombinedView};
use lemmy_db_views_modlog_combined::{impls::ModlogCombinedQuery, ModlogCombinedView};
use lemmy_db_views_person_content_combined::impls::PersonContentCombinedQuery;
use lemmy_db_views_post::{impls::PostQuery, PostView};
use lemmy_db_views_site::SiteView;
use lemmy_utils::{
  cache_header::cache_1hour,
  error::{LemmyError, LemmyErrorType, LemmyResult},
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

enum RequestType {
  Community,
  User,
  Front,
  Inbox,
  Modlog,
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
  Ok(
    get_feed_data(
      &context,
      ListingType::All,
      info.sort_type()?,
      info.get_limit(),
    )
    .await?,
  )
}

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
    )
    .await?,
  )
}

async fn get_feed_data(
  context: &LemmyContext,
  listing_type: ListingType,
  sort_type: PostSortType,
  limit: i64,
) -> LemmyResult<HttpResponse> {
  let site_view = SiteView::read_local(&mut context.pool()).await?;

  check_private_instance(&None, &site_view.local_site)?;

  let posts = PostQuery {
    listing_type: (Some(listing_type)),
    sort: (Some(sort_type)),
    limit: (Some(limit)),
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

  let rss = channel.to_string();
  Ok(
    HttpResponse::Ok()
      .content_type("application/rss+xml")
      .body(rss),
  )
}

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
    "modlog" => RequestType::Modlog,
    _ => return Err(ErrorBadRequest(LemmyError::from(anyhow!("wrong_type")))),
  };

  let builder = match request_type {
    RequestType::User => get_feed_user(&context, &info.get_limit(), &param).await,
    RequestType::Community => {
      get_feed_community(&context, &info.sort_type()?, &info.get_limit(), &param).await
    }
    RequestType::Front => {
      get_feed_front(&context, &info.sort_type()?, &info.get_limit(), &param).await
    }
    RequestType::Inbox => get_feed_inbox(&context, &param).await,
    RequestType::Modlog => get_feed_modlog(&context, &param).await,
  }
  .map_err(ErrorBadRequest)?;

  let rss = builder.to_string();

  Ok(
    HttpResponse::Ok()
      .content_type("application/rss+xml")
      .body(rss),
  )
}

async fn get_feed_user(
  context: &LemmyContext,
  limit: &i64,
  user_name: &str,
) -> LemmyResult<Channel> {
  let site_view = SiteView::read_local(&mut context.pool()).await?;
  let person = Person::read_from_name(&mut context.pool(), user_name, false)
    .await?
    .ok_or(LemmyErrorType::NotFound)?;

  check_private_instance(&None, &site_view.local_site)?;

  let content = PersonContentCombinedQuery {
    creator_id: person.id,
    type_: Some(PersonContentType::Posts),
    cursor_data: None,
    page_back: None,
    limit: (Some(*limit)),
  }
  .list(&mut context.pool(), &None, site_view.site.instance_id)
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

  Ok(channel)
}

async fn get_feed_community(
  context: &LemmyContext,
  sort_type: &PostSortType,
  limit: &i64,
  community_name: &str,
) -> LemmyResult<Channel> {
  let site_view = SiteView::read_local(&mut context.pool()).await?;
  let community = Community::read_from_name(&mut context.pool(), community_name, false)
    .await?
    .ok_or(LemmyErrorType::NotFound)?;
  if !community.visibility.can_view_without_login() {
    return Err(LemmyErrorType::NotFound.into());
  }

  check_private_instance(&None, &site_view.local_site)?;

  let posts = PostQuery {
    sort: (Some(*sort_type)),
    community_id: (Some(community.id)),
    limit: (Some(*limit)),
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

  Ok(channel)
}

async fn get_feed_front(
  context: &LemmyContext,
  sort_type: &PostSortType,
  limit: &i64,
  jwt: &str,
) -> LemmyResult<Channel> {
  let site_view = SiteView::read_local(&mut context.pool()).await?;
  let local_user = local_user_view_from_jwt(jwt, context).await?;

  check_private_instance(&Some(local_user.clone()), &site_view.local_site)?;

  let posts = PostQuery {
    listing_type: (Some(ListingType::Subscribed)),
    local_user: (Some(&local_user.local_user)),
    sort: (Some(*sort_type)),
    limit: (Some(*limit)),
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

  Ok(channel)
}

async fn get_feed_inbox(context: &LemmyContext, jwt: &str) -> LemmyResult<Channel> {
  let site_view = SiteView::read_local(&mut context.pool()).await?;
  let local_instance_id = site_view.site.instance_id;
  let local_user = local_user_view_from_jwt(jwt, context).await?;
  let my_person_id = local_user.person.id;
  let show_bot_accounts = Some(local_user.local_user.show_bot_accounts);

  check_private_instance(&Some(local_user.clone()), &site_view.local_site)?;

  let inbox = InboxCombinedQuery {
    show_bot_accounts,
    ..Default::default()
  }
  .list(&mut context.pool(), my_person_id, local_instance_id)
  .await?;

  let protocol_and_hostname = context.settings().get_protocol_and_hostname();
  let items = create_reply_and_mention_items(inbox, context)?;

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

/// Gets your ModeratorView modlog
async fn get_feed_modlog(context: &LemmyContext, jwt: &str) -> LemmyResult<Channel> {
  let site_view = SiteView::read_local(&mut context.pool()).await?;
  let local_user = local_user_view_from_jwt(jwt, context).await?;
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

  Ok(channel)
}

fn create_reply_and_mention_items(
  inbox: Vec<InboxCombinedView>,
  context: &LemmyContext,
) -> LemmyResult<Vec<Item>> {
  let reply_items: Vec<Item> = inbox
    .iter()
    .map(|r| match r {
      InboxCombinedView::CommentReply(v) => {
        let reply_url = v.comment.local_url(context.settings())?;
        build_item(
          &v.creator,
          &v.comment.published,
          reply_url.as_str(),
          &v.comment.content,
          context.settings(),
        )
      }
      InboxCombinedView::CommentMention(v) => {
        let mention_url = v.comment.local_url(context.settings())?;
        build_item(
          &v.creator,
          &v.comment.published,
          mention_url.as_str(),
          &v.comment.content,
          context.settings(),
        )
      }
      InboxCombinedView::PostMention(v) => {
        let mention_url = v.post.local_url(context.settings())?;
        build_item(
          &v.creator,
          &v.post.published,
          mention_url.as_str(),
          &v.post.body.clone().unwrap_or_default(),
          context.settings(),
        )
      }
      InboxCombinedView::PrivateMessage(v) => {
        let inbox_url = format!("{}/inbox", context.settings().get_protocol_and_hostname());
        build_item(
          &v.creator,
          &v.private_message.published,
          &inbox_url,
          &v.private_message.content,
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
        &v.admin_allow_instance.published,
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
        &v.admin_allow_instance.reason,
        settings,
      ),
      ModlogCombinedView::AdminBlockInstance(v) => build_modlog_item(
        &v.admin,
        &v.admin_block_instance.published,
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
        &v.admin_block_instance.reason,
        settings,
      ),
      ModlogCombinedView::AdminPurgeComment(v) => build_modlog_item(
        &v.admin,
        &v.admin_purge_comment.published,
        &modlog_url,
        "Admin purged comment",
        &v.admin_purge_comment.reason,
        settings,
      ),
      ModlogCombinedView::AdminPurgeCommunity(v) => build_modlog_item(
        &v.admin,
        &v.admin_purge_community.published,
        &modlog_url,
        "Admin purged community",
        &v.admin_purge_community.reason,
        settings,
      ),
      ModlogCombinedView::AdminPurgePerson(v) => build_modlog_item(
        &v.admin,
        &v.admin_purge_person.published,
        &modlog_url,
        "Admin purged person",
        &v.admin_purge_person.reason,
        settings,
      ),
      ModlogCombinedView::AdminPurgePost(v) => build_modlog_item(
        &v.admin,
        &v.admin_purge_post.published,
        &modlog_url,
        "Admin purged post",
        &v.admin_purge_post.reason,
        settings,
      ),
      ModlogCombinedView::ModAdd(v) => build_modlog_item(
        &v.moderator,
        &v.mod_add.published,
        &modlog_url,
        &format!(
          "{} admin {}",
          removed_added_str(v.mod_add.removed),
          &v.other_person.name
        ),
        &None,
        settings,
      ),
      ModlogCombinedView::ModAddCommunity(v) => build_modlog_item(
        &v.moderator,
        &v.mod_add_community.published,
        &modlog_url,
        &format!(
          "{} mod {} to /c/{}",
          removed_added_str(v.mod_add_community.removed),
          &v.other_person.name,
          &v.community.name
        ),
        &None,
        settings,
      ),
      ModlogCombinedView::ModBan(v) => build_modlog_item(
        &v.moderator,
        &v.mod_ban.published,
        &modlog_url,
        &format!(
          "{} {}",
          banned_unbanned_str(v.mod_ban.banned),
          &v.other_person.name
        ),
        &v.mod_ban.reason,
        settings,
      ),
      ModlogCombinedView::ModBanFromCommunity(v) => build_modlog_item(
        &v.moderator,
        &v.mod_ban_from_community.published,
        &modlog_url,
        &format!(
          "{} {} from /c/{}",
          banned_unbanned_str(v.mod_ban_from_community.banned),
          &v.other_person.name,
          &v.community.name
        ),
        &v.mod_ban_from_community.reason,
        settings,
      ),
      ModlogCombinedView::ModFeaturePost(v) => build_modlog_item(
        &v.moderator,
        &v.mod_feature_post.published,
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
        &None,
        settings,
      ),
      ModlogCombinedView::ModChangeCommunityVisibility(v) => build_modlog_item(
        &v.moderator,
        &v.mod_change_community_visibility.published,
        &modlog_url,
        &format!(
          "Changed /c/{} visibility to {}",
          &v.community.name, &v.mod_change_community_visibility.visibility
        ),
        &None,
        settings,
      ),
      ModlogCombinedView::ModLockPost(v) => build_modlog_item(
        &v.moderator,
        &v.mod_lock_post.published,
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
        &v.mod_lock_post.reason,
        settings,
      ),
      ModlogCombinedView::ModRemoveComment(v) => build_modlog_item(
        &v.moderator,
        &v.mod_remove_comment.published,
        &modlog_url,
        &format!(
          "{} comment {}",
          removed_restored_str(v.mod_remove_comment.removed),
          &v.comment.content
        ),
        &v.mod_remove_comment.reason,
        settings,
      ),
      ModlogCombinedView::ModRemoveCommunity(v) => build_modlog_item(
        &v.moderator,
        &v.mod_remove_community.published,
        &modlog_url,
        &format!(
          "{} community /c/{}",
          removed_restored_str(v.mod_remove_community.removed),
          &v.community.name
        ),
        &v.mod_remove_community.reason,
        settings,
      ),
      ModlogCombinedView::ModRemovePost(v) => build_modlog_item(
        &v.moderator,
        &v.mod_remove_post.published,
        &modlog_url,
        &format!(
          "{} post {}",
          removed_restored_str(v.mod_remove_post.removed),
          &v.post.name
        ),
        &v.mod_remove_post.reason,
        settings,
      ),
      ModlogCombinedView::ModTransferCommunity(v) => build_modlog_item(
        &v.moderator,
        &v.mod_transfer_community.published,
        &modlog_url,
        &format!(
          "Tranferred /c/{} to /u/{}",
          &v.community.name, &v.other_person.name
        ),
        &None,
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
  reason: &Option<String>,
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
    description: reason.clone(),
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
      pub_date: Some(p.post.published.to_rfc2822()),
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
