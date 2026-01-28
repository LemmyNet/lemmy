use actix_web::web::Json;
use chrono::Utc;
use lemmy_api_019::{
  comment::CommentResponse as CommentResponseV3,
  lemmy_db_schema::{
    CommentSortType as CommentSortTypeV3,
    ListingType as ListingTypeV3,
    RegistrationMode as RegistrationModeV3,
    SearchType as SearchTypeV3,
    SortType as SortTypeV3,
    SubscribedType as SubscribedTypeV3,
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
      local_user_vote_display_mode::LocalUserVoteDisplayMode as LocalUserVoteDisplayModeV3,
      person::Person as PersonV3,
      post::Post as PostV3,
      site::Site as SiteV3,
    },
  },
  lemmy_db_views::structs::{
    CommentView as CommentViewV3,
    LocalUserView as LocalUserViewV3,
    PostView as PostViewV3,
    SiteView as SiteViewV3,
  },
  lemmy_db_views_actor::structs::{CommunityView as CommunityViewV3, PersonView as PersonViewV3},
  person::LoginResponse as LoginResponseV3,
  post::PostResponse as PostResponseV3,
  site::{MyUserInfo as MyUserInfoV3, SearchResponse as SearchResponseV3},
};
use lemmy_db_schema::{
  CommunitySortType,
  newtypes::LanguageId,
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
use lemmy_db_schema_file::enums::{
  CommentSortType,
  CommunityFollowerState,
  ListingType,
  PostSortType,
  RegistrationMode,
};
use lemmy_db_views_comment::{CommentView, api::CommentResponse};
use lemmy_db_views_community::CommunityView;
use lemmy_db_views_local_user::LocalUserView;
use lemmy_db_views_person::PersonView;
use lemmy_db_views_post::{PostView, api::PostResponse};
use lemmy_db_views_search_combined::SearchCombinedView;
use lemmy_db_views_site::{
  SiteView,
  api::{LoginResponse, MyUserInfo},
};
use lemmy_diesel_utils::{dburl::DbUrl, sensitive::SensitiveString};
use lemmy_utils::error::LemmyResult;
use std::sync::LazyLock;
use url::Url;

#[allow(clippy::expect_used)]
static DUMMY_URL: LazyLock<DbUrlV3> = LazyLock::new(|| {
  Url::parse("http://example.com")
    .expect("parse dummy url")
    .into()
});

pub(crate) fn convert_local_user_view2(local_user_view: LocalUserView) -> LocalUserViewV3 {
  let LocalUserView {
    local_user, person, ..
  } = local_user_view;
  let (person, counts) = convert_person(person);
  let local_user = convert_local_user(local_user);
  LocalUserViewV3 {
    local_user_vote_display_mode: LocalUserVoteDisplayModeV3 {
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

pub(crate) fn convert_local_user(local_user: LocalUser) -> LocalUserV3 {
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

pub(crate) fn convert_community_view(community_view: CommunityView) -> CommunityViewV3 {
  let CommunityView {
    community,
    community_actions,
    ..
  } = community_view;
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
    subscribed: convert_subscribed_type(community_actions.as_ref().and_then(|c| c.follow_state)),
    blocked: community_actions
      .as_ref()
      .and_then(|c| c.blocked_at)
      .is_some(),
    counts,
    banned_from_community: community_actions.and_then(|c| c.received_ban_at).is_some(),
  }
}

fn convert_subscribed_type(state: Option<CommunityFollowerState>) -> SubscribedTypeV3 {
  match state {
    Some(CommunityFollowerState::Accepted) => SubscribedTypeV3::Subscribed,
    Some(CommunityFollowerState::Pending) => SubscribedTypeV3::Pending,
    Some(CommunityFollowerState::ApprovalRequired) => SubscribedTypeV3::Pending,
    Some(CommunityFollowerState::Denied) => SubscribedTypeV3::NotSubscribed,
    None => SubscribedTypeV3::NotSubscribed,
  }
}

pub(crate) fn convert_post_view(post_view: PostView) -> PostViewV3 {
  let PostView {
    post,
    creator,
    community,
    creator_is_admin,
    creator_is_moderator,
    creator_banned_from_community,
    post_actions,
    community_actions,
    ..
  } = post_view;
  let (post, counts) = convert_post(post);
  let my_vote = post_actions
    .as_ref()
    .and_then(|pa| pa.vote_is_upvote)
    .map(|vote_is_upvote| if vote_is_upvote { 1 } else { -1 });
  PostViewV3 {
    post,
    creator: convert_person(creator).0,
    community: convert_community(community),
    image_details: None,
    creator_banned_from_community,
    banned_from_community: community_actions.and_then(|c| c.received_ban_at).is_some(),
    creator_is_moderator,
    creator_is_admin,
    counts,
    subscribed: SubscribedTypeV3::NotSubscribed,
    saved: post_actions.as_ref().and_then(|p| p.saved_at).is_some(),
    read: post_actions.as_ref().and_then(|p| p.read_at).is_some(),
    hidden: post_actions.as_ref().and_then(|p| p.hidden_at).is_some(),
    creator_blocked: false,
    my_vote,
    unread_comments: 0,
  }
}

pub(crate) fn convert_comment_view(comment_view: CommentView) -> CommentViewV3 {
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
    .as_ref()
    .and_then(|pa| pa.vote_is_upvote)
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
    saved: comment_actions.and_then(|c| c.saved_at).is_some(),
    creator_blocked: false,
    my_vote,
  }
}

pub(crate) fn convert_comment(comment: Comment) -> (CommentV3, CommentAggregates) {
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

pub(crate) fn convert_my_user(my_user: Option<MyUserInfo>) -> Option<MyUserInfoV3> {
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

pub(crate) fn convert_person(person: Person) -> (PersonV3, PersonAggregates) {
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

pub(crate) fn convert_community(community: Community) -> CommunityV3 {
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
    summary: description,
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

pub(crate) fn convert_post(post: Post) -> (PostV3, PostAggregates) {
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
pub(crate) fn convert_site_view(site_view: SiteView) -> SiteViewV3 {
  let SiteView {
    site, local_site, ..
  } = site_view;

  let counts = SiteAggregates {
    site_id: SiteIdV3(site.id.0),
    users: local_site.users.into(),
    posts: local_site.posts.into(),
    comments: local_site.comments.into(),
    communities: local_site.communities.into(),
    users_active_day: local_site.users_active_day.into(),
    users_active_week: local_site.users_active_week.into(),
    users_active_month: local_site.users_active_month.into(),
    users_active_half_year: local_site.users_active_half_year.into(),
  };
  SiteViewV3 {
    site: convert_site(site),
    local_site: convert_local_site(local_site),
    local_site_rate_limit: dummy_local_site_rate_limit(),
    counts,
  }
}

pub(crate) fn convert_site(site: Site) -> SiteV3 {
  let Site {
    id,
    name,
    sidebar,
    published_at,
    updated_at,
    icon,
    banner,
    summary: description,
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

pub(crate) fn convert_db_url(db_url: DbUrl) -> DbUrlV3 {
  let url: Url = db_url.into();
  url.into()
}

pub(crate) fn convert_local_site(local_site: LocalSite) -> LocalSiteV3 {
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
    registration_mode,
    ..
  } = local_site;
  let registration_mode = match registration_mode {
    RegistrationMode::Closed => RegistrationModeV3::Closed,
    RegistrationMode::RequireApplication => RegistrationModeV3::RequireApplication,
    RegistrationMode::Open => RegistrationModeV3::Open,
  };
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
    actor_name_max_length: 20,
    federation_enabled,
    captcha_enabled,
    captcha_difficulty,
    published: published_at,
    updated: updated_at,
    registration_mode,
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

pub(crate) fn convert_person_view(person_view: PersonView) -> PersonViewV3 {
  let PersonView { person, .. } = person_view;
  let (person, counts) = convert_person(person);
  PersonViewV3 {
    person,
    counts,
    // explicitly set to false to hide all admin options from ui
    is_admin: false,
  }
}
pub(crate) fn convert_sensitive(s: SensitiveString) -> SensitiveStringV3 {
  SensitiveStringV3::from(s.into_inner())
}

pub(crate) fn convert_score(score: i16) -> Option<bool> {
  if score <= -1 {
    Some(false)
  } else if score >= 1 {
    Some(true)
  } else {
    None
  }
}
pub(crate) fn convert_search_response(
  views: Vec<SearchCombinedView>,
  type_: Option<SearchTypeV3>,
) -> SearchResponseV3 {
  let mut res = SearchResponseV3 {
    type_: type_.unwrap_or(SearchTypeV3::All),
    comments: vec![],
    posts: vec![],
    communities: vec![],
    users: vec![],
  };
  for v in views {
    match v {
      SearchCombinedView::Post(p) => res.posts.push(convert_post_view(p)),
      SearchCombinedView::Comment(c) => res.comments.push(convert_comment_view(c)),
      SearchCombinedView::Community(c) => res.communities.push(convert_community_view(c)),
      SearchCombinedView::Person(p) => res.users.push(convert_person_view(p)),
      SearchCombinedView::MultiCommunity(_) => continue,
    }
  }
  res
}

pub(crate) fn convert_post_listing_sort(
  sort_type: Option<SortTypeV3>,
) -> (Option<PostSortType>, Option<i32>) {
  const HOUR: i32 = 60 * 60;
  const DAY: i32 = 24 * HOUR;
  const WEEK: i32 = 7 * DAY;
  const MONTH: i32 = 30 * DAY;
  const YEAR: i32 = 365 * DAY;

  let Some(sort_type) = sort_type else {
    return (None, None);
  };
  let max = |s| (Some(s), Some(i32::MAX));
  let top = |t| (Some(PostSortType::Top), Some(t));
  match sort_type {
    SortTypeV3::Active => max(PostSortType::Active),
    SortTypeV3::Hot => max(PostSortType::Hot),
    SortTypeV3::New => max(PostSortType::New),
    SortTypeV3::Old => max(PostSortType::Old),
    SortTypeV3::Controversial => max(PostSortType::Controversial),
    SortTypeV3::MostComments => max(PostSortType::MostComments),
    SortTypeV3::NewComments => max(PostSortType::NewComments),
    SortTypeV3::Scaled => max(PostSortType::Scaled),
    SortTypeV3::TopHour => top(HOUR),
    SortTypeV3::TopSixHour => top(6 * HOUR),
    SortTypeV3::TopTwelveHour => top(12 * HOUR),
    SortTypeV3::TopDay => top(DAY),
    SortTypeV3::TopWeek => top(WEEK),
    SortTypeV3::TopAll => top(i32::MAX),
    SortTypeV3::TopMonth => top(MONTH),
    SortTypeV3::TopThreeMonths => top(3 * MONTH),
    SortTypeV3::TopSixMonths => top(6 * MONTH),
    SortTypeV3::TopNineMonths => top(9 * MONTH),
    SortTypeV3::TopYear => top(YEAR),
  }
}

pub(crate) fn convert_comment_listing_sort(sort_type: CommentSortTypeV3) -> CommentSortType {
  match sort_type {
    CommentSortTypeV3::Hot => CommentSortType::Hot,
    CommentSortTypeV3::Top => CommentSortType::Top,
    CommentSortTypeV3::New => CommentSortType::New,
    CommentSortTypeV3::Old => CommentSortType::Old,
    CommentSortTypeV3::Controversial => CommentSortType::Controversial,
  }
}

pub(crate) fn convert_community_listing_sort(
  sort_type: Option<SortTypeV3>,
) -> (Option<CommunitySortType>, Option<i32>) {
  const HOUR: i32 = 60 * 60;
  const DAY: i32 = 24 * HOUR;
  const WEEK: i32 = 7 * DAY;
  const MONTH: i32 = 30 * DAY;
  const YEAR: i32 = 365 * DAY;

  let Some(sort_type) = sort_type else {
    return (Some(CommunitySortType::default()), Some(i32::MAX));
  };
  let max = |s| (Some(s), Some(i32::MAX));
  let top = |t| (Some(CommunitySortType::Hot), Some(t));
  match sort_type {
    SortTypeV3::Active
    | SortTypeV3::Hot
    | SortTypeV3::MostComments
    | SortTypeV3::NewComments
    | SortTypeV3::Controversial
    | SortTypeV3::Scaled => max(CommunitySortType::Hot),
    SortTypeV3::New => max(CommunitySortType::New),
    SortTypeV3::Old => max(CommunitySortType::Old),
    SortTypeV3::TopHour => top(HOUR),
    SortTypeV3::TopSixHour => top(6 * HOUR),
    SortTypeV3::TopTwelveHour => top(12 * HOUR),
    SortTypeV3::TopDay => top(DAY),
    SortTypeV3::TopWeek => top(WEEK),
    SortTypeV3::TopAll => top(i32::MAX),
    SortTypeV3::TopMonth => top(MONTH),
    SortTypeV3::TopThreeMonths => top(3 * MONTH),
    SortTypeV3::TopSixMonths => top(6 * MONTH),
    SortTypeV3::TopNineMonths => top(9 * MONTH),
    SortTypeV3::TopYear => top(YEAR),
  }
}

pub(crate) fn convert_listing_type(listing_type: ListingTypeV3) -> ListingType {
  match listing_type {
    ListingTypeV3::All => ListingType::All,
    ListingTypeV3::Local => ListingType::Local,
    ListingTypeV3::Subscribed => ListingType::Subscribed,
    ListingTypeV3::ModeratorView => ListingType::ModeratorView,
  }
}
pub(crate) fn convert_post_response(res: Json<PostResponse>) -> LemmyResult<Json<PostResponseV3>> {
  Ok(Json(PostResponseV3 {
    post_view: convert_post_view(res.0.post_view),
  }))
}
pub(crate) fn convert_comment_response(
  res: Json<CommentResponse>,
) -> LemmyResult<Json<CommentResponseV3>> {
  Ok(Json(CommentResponseV3 {
    comment_view: convert_comment_view(res.0.comment_view),
    recipient_ids: vec![],
  }))
}

pub(crate) fn convert_language_ids(data: Vec<LanguageId>) -> Vec<LanguageIdV3> {
  data.into_iter().map(|l| LanguageIdV3(l.0)).collect()
}

pub(crate) fn convert_login_response(res: LoginResponse) -> LemmyResult<Json<LoginResponseV3>> {
  let LoginResponse {
    jwt,
    registration_created,
    verify_email_sent,
  } = res;
  Ok(Json(LoginResponseV3 {
    jwt: jwt.map(convert_sensitive),
    registration_created,
    verify_email_sent,
  }))
}
