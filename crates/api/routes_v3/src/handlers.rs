use crate::convert::{
  convert_comment,
  convert_comment_listing_sort,
  convert_comment_response,
  convert_comment_view,
  convert_community,
  convert_community_listing_sort,
  convert_community_view,
  convert_language_ids,
  convert_listing_type,
  convert_login_response,
  convert_my_user,
  convert_person,
  convert_person_view,
  convert_post,
  convert_post_listing_sort,
  convert_post_response,
  convert_post_view,
  convert_score,
  convert_search_response,
  convert_site,
  convert_site_view,
};
use activitypub_federation::config::Data as ApubData;
use actix_web::{HttpRequest, HttpResponse, web::*};
use lemmy_api::{
  comment::{like::like_comment, save::save_comment},
  community::{block::user_block_community, follow::follow_community},
  federation::{
    list_comments::list_comments,
    list_posts::list_posts,
    read_community::get_community,
    resolve_object::resolve_object,
    search::search,
  },
  local_user::{
    block::user_block_person,
    login::login,
    logout::logout,
    notifications::mark_all_read::mark_all_notifications_read,
  },
  post::{like::like_post, save::save_post},
  reports::{
    comment_report::create::create_comment_report,
    post_report::create::create_post_report,
  },
};
use lemmy_api_019::{
  comment::{
    CommentReportResponse as CommentReportResponseV3,
    CommentResponse as CommentResponseV3,
    CreateCommentLike as CreateCommentLikeV3,
    GetComments as GetCommentsV3,
    GetCommentsResponse as GetCommentsResponseV3,
  },
  community::{
    BlockCommunityResponse as BlockCommunityResponseV3,
    CommunityResponse as CommunityResponseV3,
    GetCommunityResponse as GetCommunityResponseV3,
    ListCommunities as ListCommunitiesV3,
    ListCommunitiesResponse as ListCommunitiesResponseV3,
  },
  lemmy_db_schema::{
    SubscribedType as SubscribedTypeV3,
    newtypes::LanguageId as LanguageIdV3,
    source::{
      comment_report::CommentReport as CommentReportV3,
      language::Language as LanguageV3,
      local_site_url_blocklist::LocalSiteUrlBlocklist as LocalSiteUrlBlocklistV3,
      post_report::PostReport as PostReportV3,
      tagline::Tagline as TaglineV3,
    },
  },
  lemmy_db_views::structs::{
    CommentReportView as CommentReportViewV3,
    PostReportView as PostReportViewV3,
  },
  lemmy_db_views_actor::structs::CommunityModeratorView as CommunityModeratorViewV3,
  person::{
    BlockPersonResponse as BlockPersonResponseV3,
    GetRepliesResponse as GetRepliesResponseV3,
    GetUnreadCountResponse as GetUnreadCountResponseV3,
    LoginResponse as LoginResponseV3,
  },
  post::{
    CreatePost as CreatePostV3,
    CreatePostLike as CreatePostLikeV3,
    GetPostResponse as GetPostResponseV3,
    GetPosts as GetPostsV3,
    GetPostsResponse as GetPostsResponseV3,
    PostReportResponse as PostReportResponseV3,
    PostResponse as PostResponseV3,
  },
  site::{
    GetSiteResponse as GetSiteResponseV3,
    ResolveObjectResponse as ResolveObjectResponseV3,
    Search as SearchV3,
    SearchResponse as SearchResponseV3,
  },
};
use lemmy_api_crud::{
  comment::{create::create_comment, delete::delete_comment, update::edit_comment},
  community::list::list_communities,
  post::{create::create_post, delete::delete_post, read::get_post, update::edit_post},
  site::read::get_site,
  user::{create::register, my_user::get_my_user},
};
use lemmy_api_utils::context::LemmyContext;
use lemmy_db_schema::newtypes::{CommentId, CommunityId, LanguageId, PostId};
use lemmy_db_schema_file::PersonId;
use lemmy_db_views_comment::api::{
  CreateComment,
  CreateCommentLike,
  DeleteComment,
  EditComment,
  GetComments,
  SaveComment,
};
use lemmy_db_views_community::api::{
  BlockCommunity,
  FollowCommunity,
  GetCommunity,
  ListCommunities,
};
use lemmy_db_views_local_user::LocalUserView;
use lemmy_db_views_person::api::BlockPerson;
use lemmy_db_views_post::api::{
  CreatePost,
  CreatePostLike,
  DeletePost,
  EditPost,
  GetPosts,
  SavePost,
};
use lemmy_db_views_registration_applications::api::Register;
use lemmy_db_views_report_combined::api::{CreateCommentReport, CreatePostReport};
use lemmy_db_views_search_combined::{Search, api::GetPost};
use lemmy_db_views_site::api::{GetSiteResponse, Login, ResolveObject};
use lemmy_utils::error::LemmyResult;

pub(crate) async fn get_post_v3(
  data: Query<GetPost>,
  context: Data<LemmyContext>,
  local_user_view: Option<LocalUserView>,
) -> LemmyResult<Json<GetPostResponseV3>> {
  let post = get_post(data, context, local_user_view).await?.0;
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

pub(crate) async fn list_posts_v3(
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
    type_,
    sort,
    page,
    ..
  } = datav3.0;
  let (sort, time_range_seconds) = convert_post_listing_sort(sort);
  let data = GetPosts {
    type_: type_.map(convert_listing_type),
    sort,
    time_range_seconds,
    community_id: community_id.map(|id| CommunityId(id.0)),
    community_name,
    show_hidden,
    show_read,
    show_nsfw,
    page,
    limit,
    ..Default::default()
  };
  let res = list_posts(Query(data), context, local_user_view).await?.0;
  Ok(Json(GetPostsResponseV3 {
    posts: res.into_iter().map(convert_post_view).collect(),
    next_page: None,
  }))
}

pub(crate) async fn list_comments_v3(
  Query(data): Query<GetCommentsV3>,
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
    type_,
    sort,
    ..
  } = data;
  let sort = sort.map(convert_comment_listing_sort);
  let data = GetComments {
    type_: type_.map(convert_listing_type),
    sort,
    max_depth,
    page_cursor: None,
    limit,
    community_id: community_id.map(|c| CommunityId(c.0)),
    community_name,
    post_id: post_id.map(|p| PostId(p.0)),
    parent_id: parent_id.map(|p| CommentId(p.0)),
    time_range_seconds: None,
  };
  let comments = list_comments(Query(data), context, local_user_view)
    .await?
    .0;
  Ok(Json(GetCommentsResponseV3 {
    comments: comments.into_iter().map(convert_comment_view).collect(),
  }))
}

pub(crate) async fn logout_v3(
  req: HttpRequest,
  local_user_view: LocalUserView,
  context: ApubData<LemmyContext>,
) -> LemmyResult<HttpResponse> {
  logout(req, local_user_view, context).await
}

pub(crate) async fn get_site_v3(
  local_user_view: Option<LocalUserView>,
  context: Data<LemmyContext>,
) -> LemmyResult<Json<GetSiteResponseV3>> {
  let GetSiteResponse {
    site_view,
    admins,
    version,
    all_languages,
    discussion_languages,
    tagline,
    blocked_urls,
    ..
  } = get_site(local_user_view.clone(), context.clone()).await?.0;
  let my_user = if let Some(local_user_view) = local_user_view {
    Some(get_my_user(local_user_view, context).await?.0)
  } else {
    None
  };
  Ok(Json(GetSiteResponseV3 {
    site_view: convert_site_view(site_view),
    admins: admins.into_iter().map(convert_person_view).collect(),
    version,
    my_user: convert_my_user(my_user),
    all_languages: all_languages
      .into_iter()
      .map(|l| LanguageV3 {
        id: LanguageIdV3(l.id.0),
        code: l.code,
        name: l.name,
      })
      .collect(),
    discussion_languages: convert_language_ids(discussion_languages),
    taglines: tagline
      .into_iter()
      .map(|t| TaglineV3 {
        id: t.id.0,
        local_site_id: Default::default(),
        content: t.content,
        published: t.published_at,
        updated: t.updated_at,
      })
      .collect(),
    custom_emojis: vec![],
    blocked_urls: blocked_urls
      .into_iter()
      .map(|b| LocalSiteUrlBlocklistV3 {
        id: b.id,
        url: b.url,
        published: b.published_at,
        updated: b.updated_at,
      })
      .collect(),
  }))
}

pub(crate) async fn login_v3(
  data: Json<Login>,
  req: HttpRequest,
  context: Data<LemmyContext>,
) -> LemmyResult<Json<LoginResponseV3>> {
  let res = login(data, req, context).await?.0;
  convert_login_response(res)
}

pub(crate) async fn like_comment_v3(
  Json(data): Json<CreateCommentLikeV3>,
  context: ApubData<LemmyContext>,
  local_user_view: LocalUserView,
) -> LemmyResult<Json<CommentResponseV3>> {
  let CreateCommentLikeV3 { comment_id, score } = data;
  let data = CreateCommentLike {
    comment_id: CommentId(comment_id.0),
    is_upvote: convert_score(score),
  };
  let res = like_comment(Json(data), context, local_user_view).await?;
  convert_comment_response(res)
}

pub(crate) async fn like_post_v3(
  Json(data): Json<CreatePostLikeV3>,
  context: ApubData<LemmyContext>,
  local_user_view: LocalUserView,
) -> LemmyResult<Json<PostResponseV3>> {
  let CreatePostLikeV3 { post_id, score } = data;
  let data = CreatePostLike {
    post_id: PostId(post_id.0),
    is_upvote: convert_score(score),
  };
  let res = like_post(Json(data), context, local_user_view).await?;
  convert_post_response(res)
}

pub(crate) async fn create_comment_v3(
  data: Json<CreateComment>,
  context: ApubData<LemmyContext>,
  local_user_view: LocalUserView,
) -> LemmyResult<Json<CommentResponseV3>> {
  let res = Box::pin(create_comment(data, context, local_user_view)).await?;
  convert_comment_response(res)
}

pub(crate) async fn create_post_v3(
  Json(data): Json<CreatePostV3>,
  context: ApubData<LemmyContext>,
  local_user_view: LocalUserView,
) -> LemmyResult<Json<PostResponseV3>> {
  let CreatePostV3 {
    name,
    community_id,
    url,
    body,
    alt_text,
    honeypot,
    nsfw,
    language_id,
    custom_thumbnail,
  } = data;
  let data = CreatePost {
    name,
    community_id: CommunityId(community_id.0),
    url,
    body,
    alt_text,
    honeypot,
    nsfw,
    language_id: language_id.map(|l| LanguageId(l.0)),
    custom_thumbnail,
    tags: None,
    scheduled_publish_time_at: None,
  };
  let res = Box::pin(create_post(Json(data), context, local_user_view)).await?;
  convert_post_response(res)
}

pub(crate) async fn search_v3(
  Query(data): Query<SearchV3>,
  context: ApubData<LemmyContext>,
  local_user_view: Option<LocalUserView>,
) -> LemmyResult<Json<SearchResponseV3>> {
  let SearchV3 {
    q,
    community_id,
    community_name,
    creator_id,
    limit,
    type_,
    ..
  } = data;
  let data = Search {
    q,
    community_id: community_id.map(|i| CommunityId(i.0)),
    community_name,
    creator_id: creator_id.map(|i| PersonId(i.0)),
    limit,
    ..Default::default()
  };
  let res = search(Query(data), context, local_user_view).await?;
  Ok(Json(convert_search_response(res.0.search, type_)))
}

pub(crate) async fn resolve_object_v3(
  data: Query<ResolveObject>,
  context: ApubData<LemmyContext>,
  local_user_view: Option<LocalUserView>,
) -> LemmyResult<Json<ResolveObjectResponseV3>> {
  let res = resolve_object(data, context, local_user_view).await?;
  let mut conv = convert_search_response(res.0.resolve.into_iter().collect(), None);
  Ok(Json(ResolveObjectResponseV3 {
    comment: conv.comments.pop(),
    post: conv.posts.pop(),
    community: conv.communities.pop(),
    person: conv.users.pop(),
  }))
}

pub(crate) async fn save_post_v3(
  data: Json<SavePost>,
  context: Data<LemmyContext>,
  local_user_view: LocalUserView,
) -> LemmyResult<Json<PostResponseV3>> {
  let res = save_post(data, context, local_user_view).await?;
  convert_post_response(res)
}

pub(crate) async fn save_comment_v3(
  data: Json<SaveComment>,
  context: Data<LemmyContext>,
  local_user_view: LocalUserView,
) -> LemmyResult<Json<CommentResponseV3>> {
  let res = save_comment(data, context, local_user_view).await?;
  convert_comment_response(res)
}

pub async fn unread_count_v3(
  _context: Data<LemmyContext>,
  _local_user_view: LocalUserView,
) -> LemmyResult<Json<GetUnreadCountResponseV3>> {
  // Hardcoded to 0 because new notifications cant be returned via old api.
  Ok(Json(GetUnreadCountResponseV3 {
    replies: 0,
    mentions: 0,
    private_messages: 0,
  }))
}

pub async fn mark_all_notifications_read_v3(
  context: Data<LemmyContext>,
  local_user_view: LocalUserView,
) -> LemmyResult<Json<GetRepliesResponseV3>> {
  mark_all_notifications_read(context, local_user_view).await?;
  Ok(Json(GetRepliesResponseV3 { replies: vec![] }))
}

pub async fn create_post_report_v3(
  data: Json<CreatePostReport>,
  context: ApubData<LemmyContext>,
  local_user_view: LocalUserView,
) -> LemmyResult<Json<PostReportResponseV3>> {
  let res = Box::pin(create_post_report(data, context, local_user_view))
    .await?
    .0
    .post_report_view;
  let (post, counts) = convert_post(res.post);
  let post_report = PostReportV3 {
    id: Default::default(),
    creator_id: Default::default(),
    post_id: Default::default(),
    original_post_name: Default::default(),
    original_post_url: Default::default(),
    original_post_body: Default::default(),
    reason: Default::default(),
    resolved: Default::default(),
    resolver_id: Default::default(),
    published: Default::default(),
    updated: Default::default(),
  };
  Ok(Json(PostReportResponseV3 {
    post_report_view: PostReportViewV3 {
      post_report,
      post,
      community: convert_community(res.community),
      creator: convert_person(res.creator).0,
      post_creator: convert_person(res.post_creator).0,
      creator_banned_from_community: false,
      creator_is_moderator: false,
      creator_is_admin: false,
      subscribed: SubscribedTypeV3::NotSubscribed,
      saved: false,
      read: false,
      hidden: false,
      creator_blocked: false,
      my_vote: None,
      unread_comments: 0,
      counts,
      resolver: None,
    },
  }))
}

pub async fn create_comment_report_v3(
  data: Json<CreateCommentReport>,
  context: ApubData<LemmyContext>,
  local_user_view: LocalUserView,
) -> LemmyResult<Json<CommentReportResponseV3>> {
  let res = Box::pin(create_comment_report(data, context, local_user_view))
    .await?
    .0
    .comment_report_view;
  let (comment, counts) = convert_comment(res.comment);
  let comment_report = CommentReportV3 {
    id: Default::default(),
    creator_id: Default::default(),
    comment_id: Default::default(),
    original_comment_text: Default::default(),
    reason: Default::default(),
    resolved: Default::default(),
    resolver_id: Default::default(),
    published: Default::default(),
    updated: Default::default(),
  };
  Ok(Json(CommentReportResponseV3 {
    comment_report_view: CommentReportViewV3 {
      comment_report,
      comment,
      post: convert_post(res.post).0,
      community: convert_community(res.community),
      creator: convert_person(res.creator).0,
      comment_creator: convert_person(res.comment_creator).0,
      creator_banned_from_community: false,
      creator_is_moderator: false,
      creator_is_admin: false,
      subscribed: SubscribedTypeV3::NotSubscribed,
      saved: false,
      creator_blocked: false,
      my_vote: None,
      counts,
      resolver: None,
    },
  }))
}

pub(crate) async fn get_community_v3(
  data: Query<GetCommunity>,
  context: ApubData<LemmyContext>,
  local_user_view: Option<LocalUserView>,
) -> LemmyResult<Json<GetCommunityResponseV3>> {
  let res = get_community(data, context, local_user_view).await?.0;
  Ok(Json(GetCommunityResponseV3 {
    community_view: convert_community_view(res.community_view),
    site: res.site.map(convert_site),
    moderators: res
      .moderators
      .into_iter()
      .map(|m| CommunityModeratorViewV3 {
        community: convert_community(m.community),
        moderator: convert_person(m.moderator).0,
      })
      .collect(),
    discussion_languages: convert_language_ids(res.discussion_languages),
  }))
}

pub(crate) async fn follow_community_v3(
  data: Json<FollowCommunity>,
  context: ApubData<LemmyContext>,
  local_user_view: LocalUserView,
) -> LemmyResult<Json<CommunityResponseV3>> {
  let res = follow_community(data, context, local_user_view).await?.0;
  Ok(Json(CommunityResponseV3 {
    community_view: convert_community_view(res.community_view),
    discussion_languages: convert_language_ids(res.discussion_languages),
  }))
}

pub(crate) async fn block_community_v3(
  data: Json<BlockCommunity>,
  context: ApubData<LemmyContext>,
  local_user_view: LocalUserView,
) -> LemmyResult<Json<BlockCommunityResponseV3>> {
  let blocked = data.block;
  let res = user_block_community(data, context, local_user_view)
    .await?
    .0;
  Ok(Json(BlockCommunityResponseV3 {
    community_view: convert_community_view(res.community_view),
    blocked,
  }))
}

pub(crate) async fn delete_post_v3(
  data: Json<DeletePost>,
  context: ApubData<LemmyContext>,
  local_user_view: LocalUserView,
) -> LemmyResult<Json<PostResponseV3>> {
  let res = delete_post(data, context, local_user_view).await?;
  convert_post_response(res)
}
pub(crate) async fn update_post_v3(
  data: Json<EditPost>,
  context: ApubData<LemmyContext>,
  local_user_view: LocalUserView,
) -> LemmyResult<Json<PostResponseV3>> {
  let res = Box::pin(edit_post(data, context, local_user_view)).await?;
  convert_post_response(res)
}
pub(crate) async fn delete_comment_v3(
  data: Json<DeleteComment>,
  context: ApubData<LemmyContext>,
  local_user_view: LocalUserView,
) -> LemmyResult<Json<CommentResponseV3>> {
  let res = delete_comment(data, context, local_user_view).await?;
  convert_comment_response(res)
}
pub(crate) async fn update_comment_v3(
  data: Json<EditComment>,
  context: ApubData<LemmyContext>,
  local_user_view: LocalUserView,
) -> LemmyResult<Json<CommentResponseV3>> {
  let res = Box::pin(edit_comment(data, context, local_user_view)).await?;
  convert_comment_response(res)
}
pub(crate) async fn list_communities_v3(
  Query(data): Query<ListCommunitiesV3>,
  context: Data<LemmyContext>,
  local_user_view: Option<LocalUserView>,
) -> LemmyResult<Json<ListCommunitiesResponseV3>> {
  let ListCommunitiesV3 {
    type_,
    sort,
    show_nsfw,
    limit,
    ..
  } = data;
  let (sort, time_range_seconds) = convert_community_listing_sort(sort);
  let data = ListCommunities {
    type_: type_.map(convert_listing_type),
    sort,
    time_range_seconds,
    show_nsfw,
    page_cursor: None,
    limit,
  };
  let res = list_communities(Query(data), context, local_user_view)
    .await?
    .0;
  Ok(Json(ListCommunitiesResponseV3 {
    communities: res.into_iter().map(convert_community_view).collect(),
  }))
}

pub(crate) async fn register_v3(
  data: Json<Register>,
  req: HttpRequest,
  context: ApubData<LemmyContext>,
) -> LemmyResult<Json<LoginResponseV3>> {
  let res = Box::pin(register(data, req, context)).await?.0;
  convert_login_response(res)
}

pub(crate) async fn block_person_v3(
  data: Json<BlockPerson>,
  context: Data<LemmyContext>,
  local_user_view: LocalUserView,
) -> LemmyResult<Json<BlockPersonResponseV3>> {
  let blocked = data.block;
  let res = user_block_person(data, context, local_user_view).await?.0;
  Ok(Json(BlockPersonResponseV3 {
    person_view: convert_person_view(res.person_view),
    blocked,
  }))
}
