use activitypub_federation::config::Data as ApubData;
use actix_web::{guard, web::*, HttpRequest, HttpResponse};
use chrono::Utc;
use lemmy_api::{
  comment::like::like_comment,
  federation::{list_comments::list_comments, list_posts::list_posts},
  local_user::{login::login, logout::logout},
  post::like::like_post,
};
use lemmy_api_019::{
  comment::{
    CommentResponse as CommentResponseV3,
    CreateCommentLike as CreateCommentLikeV3,
    GetComments as GetCommentsV3,
    GetCommentsResponse as GetCommentsResponseV3,
  },
  lemmy_db_schema::{
    aggregates::structs::{
      CommentAggregates,
      CommunityAggregates,
      PersonAggregates,
      PostAggregates,
      SiteAggregates,
    },
    newtypes::{
      CommentId as CommentIdV3,
      CommunityId as CommunityIdV3,
      DbUrl as DbUrlV3,
      InstanceId,
      LanguageId as LanguageIdV3,
      LocalUserId as LocalUserIdV3,
      PersonId as PersonIdV3,
      PostId as PostIdV3,
      SiteId as SiteIdV3,
    },
    sensitive::SensitiveString as SensitiveStringV3,
    source::{
      comment::Comment as CommentV3,
      community::Community as CommunityV3,
      local_site::LocalSite as LocalSiteV3,
      local_site_rate_limit::LocalSiteRateLimit as LocalSiteRateLimitV3,
      local_user::LocalUser as LocalUserV3,
      local_user_vote_display_mode::LocalUserVoteDisplayMode,
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
  person::{Login as LoginV3, LoginResponse as LoginResponseV3},
  post::{
    CreatePostLike as CreatePostLikeV3,
    GetPost as GetPostV3,
    GetPostResponse as GetPostResponseV3,
    GetPosts as GetPostsV3,
    GetPostsResponse as GetPostsResponseV3,
    PostResponse as PostResponseV3,
  },
  site::{GetSiteResponse as GetSiteResponseV3, MyUserInfo as MyUserInfoV3},
};
use lemmy_api_crud::{post::read::get_post, site::read::get_site, user::my_user::get_my_user};
use lemmy_api_utils::context::LemmyContext;
use lemmy_db_schema::{
  newtypes::{CommentId, CommunityId, DbUrl, PostId},
  sensitive::SensitiveString,
  source::{
    comment::Comment,
    community::Community,
    local_site::LocalSite,
    local_user::LocalUser,
    person::Person,
    post::Post,
    site::Site,
  },
};
use lemmy_db_views_comment::{
  api::{CreateCommentLike, GetComments},
  CommentView,
};
use lemmy_db_views_community::CommunityView;
use lemmy_db_views_local_user::LocalUserView;
use lemmy_db_views_post::{
  api::{CreatePostLike, GetPosts},
  PostView,
};
use lemmy_db_views_search_combined::api::GetPost;
use lemmy_db_views_site::{
  api::{Login, LoginResponse, MyUserInfo},
  SiteView,
};
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
          .route("/list", get().to(list_posts_v3))
          .route("/like", post().to(like_post_v3)),
      )
      .service(
        scope("/comment")
          .wrap(rate_limit.message())
          .route("/like", post().to(like_comment_v3))
          .route("/list", get().to(list_comments_v3)),
      )
      .service(
        resource("/user/login")
          .guard(guard::Post())
          .wrap(rate_limit.register())
          .route(post().to(login_v3)),
      )
      .service(
        scope("/user")
          .wrap(rate_limit.message())
          .route("/logout", post().to(logout_v3)),
      ),
  );
}

async fn like_comment_v3(
  data: Json<CreateCommentLikeV3>,
  context: ApubData<LemmyContext>,
  local_user_view: LocalUserView,
) -> LemmyResult<Json<CommentResponseV3>> {
  let CreateCommentLikeV3 { comment_id, score } = data.0;
  let data = CreateCommentLike {
    comment_id: CommentId(comment_id.0),
    is_upvote: convert_score(score),
  };
  let res = like_comment(Json(data), context, local_user_view).await?.0;
  Ok(Json(CommentResponseV3 {
    comment_view: convert_comment_view(res.comment_view),
    recipient_ids: vec![],
  }))
}

pub async fn like_post_v3(
  data: Json<CreatePostLikeV3>,
  context: ApubData<LemmyContext>,
  local_user_view: LocalUserView,
) -> LemmyResult<Json<PostResponseV3>> {
  let CreatePostLikeV3 { post_id, score } = data.0;
  let data = CreatePostLike {
    post_id: PostId(post_id.0),
    is_upvote: convert_score(score),
  };
  let res = like_post(Json(data), context, local_user_view).await?.0;
  Ok(Json(PostResponseV3 {
    post_view: convert_post_view(res.post_view),
  }))
}

fn convert_score(score: i16) -> Option<bool> {
  if score <= -1 {
    Some(false)
  } else if score >= 1 {
    Some(true)
  } else {
    None
  }
}

async fn login_v3(
  data: Json<LoginV3>,
  req: HttpRequest,
  context: Data<LemmyContext>,
) -> LemmyResult<Json<LoginResponseV3>> {
  let LoginV3 {
    username_or_email,
    password,
    totp_2fa_token,
  } = data.0;
  let data = Login {
    username_or_email: convert_sensitive(username_or_email),
    password: convert_sensitive(password),
    totp_2fa_token,
    stay_logged_in: None,
  };
  let LoginResponse {
    jwt,
    registration_created,
    verify_email_sent,
  } = login(Json(data), req, context).await?.0;
  Ok(Json(LoginResponseV3 {
    jwt: jwt.map(convert_sensitive2),
    registration_created,
    verify_email_sent,
  }))
}

fn convert_sensitive(s: SensitiveStringV3) -> SensitiveString {
  SensitiveString::from(s.into_inner())
}

fn convert_sensitive2(s: SensitiveString) -> SensitiveStringV3 {
  SensitiveStringV3::from(s.into_inner())
}

async fn logout_v3(
  req: HttpRequest,
  // require login
  _local_user_view: LocalUserView,
  context: ApubData<LemmyContext>,
) -> LemmyResult<HttpResponse> {
  logout(req, _local_user_view, context).await
}

async fn get_site_v3(
  local_user_view: Option<LocalUserView>,
  context: Data<LemmyContext>,
) -> LemmyResult<Json<GetSiteResponseV3>> {
  //let local_user_view_019 = local_user_view.map(convert_local_user_view2);
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

async fn get_post_v3(
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

async fn list_posts_v3(
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

async fn list_comments_v3(
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

fn convert_local_user_view2(local_user_view: LocalUserView) -> LocalUserViewV3 {
  let LocalUserView {
    local_user, person, ..
  } = local_user_view;
  let (person, counts) = convert_person(person);
  let local_user = convert_local_user(local_user);
  LocalUserViewV3 {
    local_user_vote_display_mode: LocalUserVoteDisplayMode {
      local_user_id: local_user.id,
      score: false,
      upvotes: true,
      downvotes: true,
      upvote_percentage: false,
    },
    local_user,
    person,
    counts,
  }
}

fn convert_local_user(local_user: LocalUser) -> LocalUserV3 {
  let LocalUser {
    id,
    person_id,
    show_nsfw,
    theme,
    interface_language,
    show_avatars,
    send_notifications_to_email,
    show_bot_accounts,
    show_read_posts,
    email_verified,
    accepted_application,
    open_links_in_new_tab,
    blur_nsfw,
    infinite_scroll_enabled,
    totp_2fa_enabled,
    enable_animated_images,
    collapse_bot_comments,
    last_donation_notification_at,
    ..
  } = local_user;
  LocalUserV3 {
    id: LocalUserIdV3(id.0),
    person_id: PersonIdV3(person_id.0),
    password_encrypted: Default::default(),
    email: None,
    show_nsfw,
    theme,
    default_sort_type: Default::default(),
    default_listing_type: Default::default(),
    interface_language,
    show_avatars,
    send_notifications_to_email,
    show_scores: false,
    show_bot_accounts,
    show_read_posts,
    email_verified,
    accepted_application,
    totp_2fa_secret: None,
    open_links_in_new_tab,
    blur_nsfw,
    auto_expand: false,
    infinite_scroll_enabled,
    admin: false,
    post_listing_mode: Default::default(),
    totp_2fa_enabled,
    enable_keyboard_navigation: false,
    enable_animated_images,
    collapse_bot_comments,
    last_donation_notification: last_donation_notification_at,
  }
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
    post_actions,
    ..
  } = post_view;
  let (post, counts) = convert_post(post);
  let my_vote = post_actions
    .map(|pa| pa.vote_is_upvote)
    .flatten()
    .map(|vote_is_upvote| if vote_is_upvote { 1 } else { -1 });
  PostViewV3 {
    post,
    creator: convert_person(creator).0,
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
    my_vote,
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
    comment_actions,
    ..
  } = comment_view;
  let (comment, counts) = convert_comment(comment);
  let my_vote = comment_actions
    .map(|pa| pa.vote_is_upvote)
    .flatten()
    .map(|vote_is_upvote| if vote_is_upvote { 1 } else { -1 });
  CommentViewV3 {
    comment,
    creator: convert_person(creator).0,
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
    my_vote,
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

fn convert_person(person: Person) -> (PersonV3, PersonAggregates) {
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
    post_count,
    post_score,
    comment_count,
    comment_score,
    ..
  } = person;
  let id = PersonIdV3(id.0);
  (
    PersonV3 {
      id,
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
    },
    PersonAggregates {
      person_id: id,
      post_count: post_count.into(),
      post_score: post_score.into(),
      comment_count: comment_count.into(),
      comment_score: comment_score.into(),
    },
  )
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
