use activitypub_federation::config::Data as ApubData;
use actix_web::web::*;
use chrono::Utc;
use lemmy_api::federation::{list_comments::list_comments, list_posts::list_posts};
use lemmy_api_019::{
  comment::{GetComments as GetCommentsV3, GetCommentsResponse as GetCommentsResponseV3},
  lemmy_db_schema::{
    aggregates::structs::{CommentAggregates, CommunityAggregates, PostAggregates, SiteAggregates},
    newtypes::{
      CommentId as CommentIdV3,
      CommunityId as CommunityIdV3,
      DbUrl as DbUrlV3,
      InstanceId,
      LanguageId as LanguageIdV3,
      PersonId as PersonIdV3,
      PostId as PostIdV3,
      SiteId as SiteIdV3,
    },
    source::{
      comment::Comment as CommentV3,
      community::Community as CommunityV3,
      local_site::LocalSite as LocalSiteV3,
      local_site_rate_limit::LocalSiteRateLimit as LocalSiteRateLimitV3,
      person::Person as PersonV3,
      post::Post as PostV3,
      site::Site as SiteV3,
    },
    SubscribedType as SubscribedTypeV3,
  },
  lemmy_db_views::structs::{
    CommentView as CommentViewV3,
    LocalUserView as LocalUserViewV3,
    PostView as PostViewV3,
    SiteView as SiteViewV3,
  },
  lemmy_db_views_actor::structs::CommunityView as CommunityViewV3,
  post::{
    GetPost as GetPostV3,
    GetPostResponse as GetPostResponseV3,
    GetPosts as GetPostsV3,
    GetPostsResponse as GetPostsResponseV3,
  },
  site::{GetSiteResponse as GetSiteResponseV3, MyUserInfo as MyUserInfoV3},
};
use lemmy_api_crud::{post::read::get_post, site::read::get_site, user::my_user::get_my_user};
use lemmy_api_utils::context::LemmyContext;
use lemmy_db_schema::{
  newtypes::{CommentId, CommunityId, DbUrl, PostId},
  source::{
    comment::Comment,
    community::Community,
    local_site::LocalSite,
    person::Person,
    post::Post,
    site::Site,
  },
};
use lemmy_db_views_comment::{api::GetComments, CommentView};
use lemmy_db_views_community::CommunityView;
use lemmy_db_views_local_user::LocalUserView;
use lemmy_db_views_post::{api::GetPosts, PostView};
use lemmy_db_views_search_combined::api::GetPost;
use lemmy_db_views_site::{api::MyUserInfo, SiteView};
use lemmy_utils::{error::LemmyResult, rate_limit::RateLimit};
use std::sync::LazyLock;
use url::Url;

pub fn config(cfg: &mut ServiceConfig, rate_limit: &RateLimit) {
  cfg.service(
    scope("/api/v3")
      .wrap(rate_limit.message())
      // Site
      .service(scope("/site").route("", get().to(get_site_v3)))
      .service(
        scope("/post")
          .wrap(rate_limit.message())
          .route("", get().to(get_post_v3))
          .route("/list", get().to(list_posts_v3)),
      )
      .service(
        scope("/comment")
          .wrap(rate_limit.message())
          .route("/list", get().to(list_comments_v3)),
      ),
  );
}

async fn get_site_v3(
  // TODO
  //local_user_view: Option<LocalUserViewV3>,
  context: Data<LemmyContext>,
) -> LemmyResult<Json<GetSiteResponseV3>> {
  let local_user_view = None; //local_user_view.map(convert_local_user_view);
  let site = get_site(local_user_view.clone(), context.clone()).await?.0;
  let my_user = if let Some(local_user_view) = local_user_view {
    Some(get_my_user(local_user_view, context).await?.0)
  } else {
    None
  };
  Ok(Json(GetSiteResponseV3 {
    site_view: convert_site_view(site.site_view),
    admins: vec![],
    version: site.version,
    my_user: convert_my_user(my_user),
    all_languages: vec![],
    discussion_languages: vec![],
    taglines: vec![],
    custom_emojis: vec![],
    blocked_urls: vec![],
  }))
}

pub async fn get_post_v3(
  data: Query<GetPostV3>,
  context: Data<LemmyContext>,
  local_user_view: Option<LocalUserView>,
) -> LemmyResult<Json<GetPostResponseV3>> {
  let GetPostV3 { id, comment_id } = data.0;
  let data = GetPost {
    id: id.map(|id| PostId(id.0)),
    comment_id: comment_id.map(|id| CommentId(id.0)),
  };
  let post = get_post(Query(data), context, local_user_view).await?.0;
  Ok(Json(GetPostResponseV3 {
    post_view: convert_post_view(post.post_view),
    community_view: convert_community_view(post.community_view),
    moderators: vec![],
    cross_posts: post
      .cross_posts
      .into_iter()
      .map(convert_post_view)
      .collect(),
  }))
}

pub async fn list_posts_v3(
  datav3: Query<GetPostsV3>,
  context: ApubData<LemmyContext>,
  local_user_view: Option<LocalUserView>,
) -> LemmyResult<Json<GetPostsResponseV3>> {
  let GetPostsV3 {
    limit,
    community_id,
    community_name,
    show_hidden,
    show_read,
    show_nsfw,
    ..
  } = datav3.0;
  let data = GetPosts {
    type_: Default::default(),
    sort: Default::default(),
    time_range_seconds: Default::default(),
    community_id: community_id.map(|id| CommunityId(id.0)),
    community_name,
    multi_community_id: None,
    show_hidden,
    show_read,
    show_nsfw,
    hide_media: None,
    mark_as_read: None,
    no_comments_only: None,
    page_cursor: None,
    page_back: None,
    limit,
  };
  let posts = list_posts(Query(data), context, local_user_view)
    .await?
    .0
    .posts;
  Ok(Json(GetPostsResponseV3 {
    posts: posts.into_iter().map(convert_post_view).collect(),
    next_page: None,
  }))
}

pub async fn list_comments_v3(
  data: Query<GetCommentsV3>,
  context: ApubData<LemmyContext>,
  local_user_view: Option<LocalUserView>,
) -> LemmyResult<Json<GetCommentsResponseV3>> {
  let GetCommentsV3 {
    max_depth,
    limit,
    community_id,
    community_name,
    post_id,
    parent_id,
    ..
  } = data.0;
  let data = GetComments {
    type_: None,
    sort: None,
    time_range_seconds: None,
    max_depth,
    page_cursor: None,
    page_back: None,
    limit,
    community_id: community_id.map(|c| CommunityId(c.0)),
    community_name,
    post_id: post_id.map(|p| PostId(p.0)),
    parent_id: parent_id.map(|p| CommentId(p.0)),
  };
  let comments = list_comments(Query(data), context, local_user_view)
    .await?
    .0
    .comments;
  Ok(Json(GetCommentsResponseV3 {
    comments: comments.into_iter().map(convert_comment_view).collect(),
  }))
}

static DUMMY_URL: LazyLock<DbUrlV3> =
  LazyLock::new(|| Url::parse("http://example.com").unwrap().into());

fn convert_local_user_view(local_user_view: LocalUserViewV3) -> LocalUserView {
  todo!()
}
fn convert_local_user_view2(local_user_view: LocalUserView) -> LocalUserViewV3 {
  todo!()
}

fn convert_community_view(community_view: CommunityView) -> CommunityViewV3 {
  let CommunityView { community, .. } = community_view;
  let counts = CommunityAggregates {
    community_id: CommunityIdV3(community.id.0),
    subscribers: community.subscribers.into(),
    posts: community.posts.into(),
    comments: community.comments.into(),
    published: community.published_at,
    users_active_day: community.users_active_day.into(),
    users_active_week: community.users_active_week.into(),
    users_active_month: community.users_active_month.into(),
    users_active_half_year: community.users_active_half_year.into(),
    hot_rank: community.hot_rank.into(),
    subscribers_local: community.subscribers_local.into(),
  };
  CommunityViewV3 {
    community: convert_community(community),
    subscribed: SubscribedTypeV3::NotSubscribed,
    blocked: false,
    counts,
    banned_from_community: false,
  }
}

fn convert_post_view(post_view: PostView) -> PostViewV3 {
  let PostView {
    post,
    creator,
    community,
    creator_is_admin,
    creator_is_moderator,
    creator_banned_from_community,
    ..
  } = post_view;
  let (post, counts) = convert_post(post);
  PostViewV3 {
    post,
    creator: convert_person(creator),
    community: convert_community(community),
    image_details: None,
    creator_banned_from_community,
    banned_from_community: false,
    creator_is_moderator,
    creator_is_admin,
    counts,
    subscribed: SubscribedTypeV3::NotSubscribed,
    saved: false,
    read: false,
    hidden: false,
    creator_blocked: false,
    my_vote: Default::default(),
    unread_comments: 0,
  }
}

fn convert_comment_view(comment_view: CommentView) -> CommentViewV3 {
  let CommentView {
    comment,
    creator,
    post,
    community,
    creator_is_admin,
    creator_is_moderator,
    creator_banned_from_community,
    ..
  } = comment_view;
  let (comment, counts) = convert_comment(comment);
  CommentViewV3 {
    comment,
    creator: convert_person(creator),
    post: convert_post(post).0,
    community: convert_community(community),
    counts,
    creator_banned_from_community,
    banned_from_community: false,
    creator_is_moderator,
    creator_is_admin,
    subscribed: SubscribedTypeV3::NotSubscribed,
    saved: false,
    creator_blocked: false,
    my_vote: None,
  }
}

fn convert_comment(comment: Comment) -> (CommentV3, CommentAggregates) {
  let Comment {
    id,
    creator_id,
    post_id,
    content,
    removed,
    published_at,
    updated_at,
    deleted,
    ap_id,
    local,
    path,
    distinguished,
    language_id,
    score,
    upvotes,
    downvotes,
    child_count,
    hot_rank,
    controversy_rank,
    ..
  } = comment;
  let id = CommentIdV3(id.0);
  (
    CommentV3 {
      id,
      creator_id: PersonIdV3(creator_id.0),
      post_id: PostIdV3(post_id.0),
      content,
      removed,
      published: published_at,
      updated: updated_at,
      deleted,
      ap_id: convert_db_url(ap_id),
      local,
      path: path.0,
      distinguished,
      language_id: LanguageIdV3(language_id.0),
    },
    CommentAggregates {
      comment_id: id,
      score: score.into(),
      upvotes: upvotes.into(),
      downvotes: downvotes.into(),
      published: published_at,
      child_count,
      hot_rank: hot_rank.into(),
      controversy_rank: controversy_rank.into(),
    },
  )
}

fn convert_my_user(my_user: Option<MyUserInfo>) -> Option<MyUserInfoV3> {
  if let Some(my_user) = my_user {
    let MyUserInfo {
      local_user_view, ..
    } = my_user;
    Some(MyUserInfoV3 {
      local_user_view: convert_local_user_view2(local_user_view),
      follows: vec![],
      moderates: vec![],
      community_blocks: vec![],
      instance_blocks: vec![],
      person_blocks: vec![],
      discussion_languages: vec![],
    })
  } else {
    None
  }
}

fn convert_person(person: Person) -> PersonV3 {
  let Person {
    id,
    name,
    display_name,
    avatar,
    published_at,
    updated_at,
    ap_id,
    bio,
    local,
    public_key,
    last_refreshed_at,
    banner,
    deleted,
    matrix_user_id,
    bot_account,
    ..
  } = person;
  PersonV3 {
    id: PersonIdV3(id.0),
    name,
    display_name,
    avatar: avatar.map(convert_db_url),
    banned: false,
    published: published_at,
    updated: updated_at,
    actor_id: convert_db_url(ap_id),
    bio,
    local,
    private_key: Default::default(),
    public_key,
    last_refreshed_at,
    banner: banner.map(convert_db_url),
    deleted,
    inbox_url: DUMMY_URL.clone(),
    shared_inbox_url: None,
    matrix_user_id,
    bot_account,
    ban_expires: None,
    instance_id: Default::default(),
  }
}

fn convert_community(community: Community) -> CommunityV3 {
  let Community {
    id,
    name,
    title,
    removed,
    published_at,
    updated_at,
    deleted,
    nsfw,
    ap_id,
    local,
    public_key,
    last_refreshed_at,
    icon,
    banner,
    posting_restricted_to_mods,
    instance_id,
    description,
    ..
  } = community;
  CommunityV3 {
    id: CommunityIdV3(id.0),
    name,
    title,
    description,
    removed,
    published: published_at,
    updated: updated_at,
    deleted,
    nsfw,
    actor_id: convert_db_url(ap_id),
    local,
    private_key: None,
    public_key,
    last_refreshed_at,
    icon: icon.map(convert_db_url),
    banner: banner.map(convert_db_url),
    followers_url: None,
    inbox_url: DUMMY_URL.clone(),
    shared_inbox_url: None,
    hidden: false,
    posting_restricted_to_mods,
    instance_id: InstanceId(instance_id.0),
    moderators_url: None,
    featured_url: None,
    visibility: Default::default(),
  }
}

fn convert_post(post: Post) -> (PostV3, PostAggregates) {
  let Post {
    id,
    name,
    url,
    body,
    creator_id,
    community_id,
    removed,
    locked,
    published_at,
    updated_at,
    deleted,
    nsfw,
    embed_title,
    embed_description,
    thumbnail_url,
    ap_id,
    local,
    embed_video_url,
    language_id,
    featured_community,
    featured_local,
    url_content_type,
    alt_text,
    comments,
    score,
    upvotes,
    downvotes,
    hot_rank,
    hot_rank_active,
    controversy_rank,
    scaled_rank,
    ..
  } = post;
  let post_id = PostIdV3(id.0);
  let creator_id = PersonIdV3(creator_id.0);
  let community_id = CommunityIdV3(community_id.0);
  (
    PostV3 {
      id: post_id,
      name,
      url: url.map(convert_db_url),
      body,
      creator_id,
      community_id,
      removed,
      locked,
      published: published_at,
      updated: updated_at,
      deleted,
      nsfw,
      embed_title,
      embed_description,
      thumbnail_url: thumbnail_url.map(convert_db_url),
      ap_id: convert_db_url(ap_id),
      local,
      embed_video_url: embed_video_url.map(convert_db_url),
      language_id: LanguageIdV3(language_id.0),
      featured_community,
      featured_local,
      url_content_type,
      alt_text,
    },
    PostAggregates {
      post_id,
      comments: comments.into(),
      score: score.into(),
      upvotes: upvotes.into(),
      downvotes: downvotes.into(),
      published: published_at,
      newest_comment_time_necro: Utc::now(),
      newest_comment_time: Utc::now(),
      featured_community,
      featured_local,
      hot_rank: hot_rank.into(),
      hot_rank_active: hot_rank_active.into(),
      community_id,
      creator_id,
      controversy_rank: controversy_rank.into(),
      instance_id: Default::default(),
      scaled_rank: scaled_rank.into(),
    },
  )
}
fn convert_site_view(site_view: SiteView) -> SiteViewV3 {
  let SiteView {
    site, local_site, ..
  } = site_view;
  SiteViewV3 {
    site: convert_site(site),
    local_site: convert_local_site(local_site),
    local_site_rate_limit: dummy_local_site_rate_limit(),
    counts: dummy_local_site_counts(),
  }
}

fn convert_site(site: Site) -> SiteV3 {
  let Site {
    id,
    name,
    sidebar,
    published_at,
    updated_at,
    icon,
    banner,
    description,
    ap_id,
    last_refreshed_at,
    public_key,
    content_warning,
    ..
  } = site;
  SiteV3 {
    id: SiteIdV3(id.0),
    name,
    sidebar,
    published: published_at,
    updated: updated_at,
    icon: icon.map(convert_db_url),
    banner: banner.map(convert_db_url),
    description,
    last_refreshed_at,
    actor_id: convert_db_url(ap_id),
    inbox_url: DUMMY_URL.clone(),
    private_key: Default::default(),
    public_key,
    instance_id: Default::default(),
    content_warning,
  }
}

fn convert_db_url(db_url: DbUrl) -> DbUrlV3 {
  let url: Url = db_url.into();
  url.into()
}

fn convert_local_site(local_site: LocalSite) -> LocalSiteV3 {
  let LocalSite {
    site_id,
    site_setup,
    community_creation_admin_only,
    require_email_verification,
    application_question,
    private_instance,
    default_theme,
    legal_information,
    application_email_admins,
    slur_filter_regex,
    federation_enabled,
    captcha_enabled,
    captcha_difficulty,
    published_at,
    updated_at,
    reports_email_admins,
    federation_signed_fetch,
    ..
  } = local_site;
  LocalSiteV3 {
    id: Default::default(),
    site_id: SiteIdV3(site_id.0),
    site_setup,
    enable_downvotes: true,
    enable_nsfw: true,
    community_creation_admin_only,
    require_email_verification,
    application_question,
    private_instance,
    default_theme,
    default_post_listing_type: Default::default(),
    legal_information,
    hide_modlog_mod_names: true,
    application_email_admins,
    slur_filter_regex,
    actor_name_max_length: Default::default(),
    federation_enabled,
    captcha_enabled,
    captcha_difficulty,
    published: published_at,
    updated: updated_at,
    registration_mode: Default::default(),
    reports_email_admins,
    federation_signed_fetch,
    default_post_listing_mode: Default::default(),
    default_sort_type: Default::default(),
  }
}

fn dummy_local_site_rate_limit() -> LocalSiteRateLimitV3 {
  LocalSiteRateLimitV3 {
    local_site_id: Default::default(),
    message: 0,
    message_per_second: 0,
    post: 0,
    post_per_second: 0,
    register: 0,
    register_per_second: 0,
    image: 0,
    image_per_second: 0,
    comment: 0,
    comment_per_second: 0,
    search: 0,
    search_per_second: 0,
    published: Utc::now(),
    updated: None,
    import_user_settings: 0,
    import_user_settings_per_second: 0,
  }
}

fn dummy_local_site_counts() -> SiteAggregates {
  SiteAggregates {
    site_id: Default::default(),
    users: 0,
    posts: 0,
    comments: 0,
    communities: 0,
    users_active_day: 0,
    users_active_week: 0,
    users_active_month: 0,
    users_active_half_year: 0,
  }
}
