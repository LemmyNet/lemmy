use crate::convert::{
  convert_comment,
  convert_comment_listing_sort,
  convert_comment_response,
  convert_comment_view,
  convert_community,
  convert_community_listing_sort,
  convert_community_response,
  convert_community_view,
  convert_custom_emoji_response,
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
  convert_resolve_object_response,
  convert_score,
  convert_search_response,
  convert_search_type,
  convert_site,
  convert_site_view,
  convert_votes_to_v3,
};
use activitypub_federation::config::Data as ApubData;
use actix_web::{HttpRequest, HttpResponse, web::*};
use lemmy_api::{
  comment::{
    distinguish::distinguish_comment,
    like::like_comment,
    list_comment_likes::list_comment_likes,
    save::save_comment,
  },
  community::{
    add_mod::add_mod_to_community,
    ban::ban_from_community,
    block::user_block_community,
    follow::follow_community,
    transfer::transfer_community,
  },
  federation::{
    list_comments::list_comments,
    list_person_content::list_person_content,
    list_posts::list_posts,
    read_community::get_community,
    read_person::read_person,
    resolve_object::resolve_object,
    search::search,
  },
  local_user::{
    block::user_block_person,
    login::login,
    logout::logout,
    notifications::mark_all_read::mark_all_notifications_read,
  },
  post::{
    feature::feature_post,
    get_link_metadata::get_link_metadata,
    hide::hide_post,
    like::like_post,
    list_post_likes::list_post_likes,
    lock::lock_post,
    save::save_post,
  },
  reports::{
    comment_report::create::create_comment_report,
    post_report::create::create_post_report,
  },
};
use lemmy_api_019::{
  SuccessResponse as SuccessResponseV3,
  comment::{
    CommentReportResponse as CommentReportResponseV3,
    CommentResponse as CommentResponseV3,
    CreateCommentLike as CreateCommentLikeV3,
    DistinguishComment as DistinguishCommentV3,
    GetComments as GetCommentsV3,
    GetCommentsResponse as GetCommentsResponseV3,
    ListCommentLikes as ListCommentLikesV3,
    ListCommentLikesResponse as ListCommentLikesResponseV3,
    RemoveComment as RemoveCommentV3,
  },
  community::{
    AddModToCommunity as AddModToCommunityV3,
    AddModToCommunityResponse as AddModToCommunityResponseV3,
    BanFromCommunity as BanFromCommunityV3,
    BanFromCommunityResponse as BanFromCommunityResponseV3,
    BlockCommunityResponse as BlockCommunityResponseV3,
    CommunityResponse as CommunityResponseV3,
    CreateCommunity as CreateCommunityV3,
    DeleteCommunity as DeleteCommunityV3,
    GetCommunityResponse as GetCommunityResponseV3,
    ListCommunities as ListCommunitiesV3,
    ListCommunitiesResponse as ListCommunitiesResponseV3,
    RemoveCommunity as RemoveCommunityV3,
    TransferCommunity as TransferCommunityV3,
  },
  custom_emoji::{
    CreateCustomEmoji as CreateCustomEmojiV3,
    CustomEmojiResponse as CustomEmojiResponseV3,
    DeleteCustomEmoji as DeleteCustomEmojiV3,
    EditCustomEmoji as EditCustomEmojiV3,
  },
  lemmy_db_schema::{
    PostFeatureType as PostFeatureTypeV3,
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
    PaginationCursor as V3PaginationCursor,
    PostReportView as PostReportViewV3,
  },
  lemmy_db_views_actor::structs::CommunityModeratorView as CommunityModeratorViewV3,
  person::{
    BlockPersonResponse as BlockPersonResponseV3,
    GetPersonDetails as GetPersonDetailsV3,
    GetPersonDetailsResponse as GetPersonDetailsResponseV3,
    GetRepliesResponse as GetRepliesResponseV3,
    GetUnreadCountResponse as GetUnreadCountResponseV3,
    LoginResponse as LoginResponseV3,
  },
  post::{
    CreatePost as CreatePostV3,
    CreatePostLike as CreatePostLikeV3,
    FeaturePost as FeaturePostV3,
    GetPostResponse as GetPostResponseV3,
    GetPosts as GetPostsV3,
    GetPostsResponse as GetPostsResponseV3,
    GetSiteMetadataResponse as GetSiteMetadataResponseV3,
    HidePost as HidePostV3,
    ListPostLikes as ListPostLikesV3,
    ListPostLikesResponse as ListPostLikesResponseV3,
    LockPost as LockPostV3,
    MarkPostAsRead as MarkPostAsReadV3,
    PostReportResponse as PostReportResponseV3,
    PostResponse as PostResponseV3,
    RemovePost as RemovePostV3,
  },
  site::{
    GetSiteResponse as GetSiteResponseV3,
    ResolveObjectResponse as ResolveObjectResponseV3,
    Search as SearchV3,
    SearchResponse as SearchResponseV3,
  },
};
use lemmy_api_crud::{
  comment::{
    create::create_comment,
    delete::delete_comment,
    remove::remove_comment,
    update::edit_comment,
  },
  community::{
    create::create_community,
    delete::delete_community,
    list::list_communities,
    remove::remove_community,
  },
  custom_emoji::{
    create::create_custom_emoji,
    delete::delete_custom_emoji,
    update::edit_custom_emoji,
  },
  post::{
    create::create_post,
    delete::delete_post,
    read::get_post,
    remove::remove_post,
    update::edit_post,
  },
  site::read::get_site,
  user::{create::register, my_user::get_my_user},
};
use lemmy_api_utils::context::LemmyContext;
use lemmy_db_schema::{PostFeatureType, source::post::PostActions};
use lemmy_db_schema_file::{
  PersonId,
  newtypes::{CommentId, CommunityId, CustomEmojiId, LanguageId, PostId},
};
use lemmy_db_views_comment::api::{
  CreateComment,
  CreateCommentLike,
  DeleteComment,
  DistinguishComment,
  EditComment,
  GetComments,
  ListCommentLikes,
  RemoveComment,
  SaveComment,
};
use lemmy_db_views_community::api::{
  AddModToCommunity,
  BanFromCommunity,
  BlockCommunity,
  CreateCommunity,
  DeleteCommunity,
  FollowCommunity,
  GetCommunity,
  ListCommunities,
  RemoveCommunity,
  TransferCommunity,
};
use lemmy_db_views_custom_emoji::api::{CreateCustomEmoji, DeleteCustomEmoji, EditCustomEmoji};
use lemmy_db_views_local_user::LocalUserView;
use lemmy_db_views_person::api::{BlockPerson, GetPersonDetails};
use lemmy_db_views_person_content_combined::ListPersonContent;
use lemmy_db_views_post::api::{
  CreatePost,
  CreatePostLike,
  DeletePost,
  EditPost,
  FeaturePost,
  GetPost,
  GetPosts,
  GetSiteMetadata,
  HidePost,
  ListPostLikes,
  LockPost,
  RemovePost,
  SavePost,
};
use lemmy_db_views_post_comment_combined::PostCommentCombinedView;
use lemmy_db_views_registration_applications::api::Register;
use lemmy_db_views_report_combined::api::{CreateCommentReport, CreatePostReport};
use lemmy_db_views_site::api::{GetSiteResponse, Login, ResolveObject, Search};
use lemmy_diesel_utils::pagination::PaginationCursor;
use lemmy_utils::error::{LemmyErrorType, LemmyResult};

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
    page_cursor,
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
    page_cursor: page_cursor.map(|c| PaginationCursor(c.0)),
    page,
    limit,
    ..Default::default()
  };
  let res = list_posts(Query(data), context, local_user_view).await?.0;
  Ok(Json(GetPostsResponseV3 {
    posts: res.items.into_iter().map(convert_post_view).collect(),
    next_page: res.next_page.map(|c| V3PaginationCursor(c.0)),
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
    search_term: None,
    creator_id: None,
    creator_username: None,
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
    Some(Box::pin(get_my_user(local_user_view, context)).await?.0)
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
    type_,
    q: search_term,
    community_id,
    community_name,
    creator_id,
    sort,
    limit,
    post_title_only: title_only,
    ..
  } = data;
  let time_range_seconds = convert_post_listing_sort(sort).1;
  let form = Search {
    search_term,
    community_id: community_id.map(|id| CommunityId(id.0)),
    community_name,
    creator_id: creator_id.map(|id| PersonId(id.0)),
    limit,
    time_range_seconds,
    title_only,
    type_: convert_search_type(type_),
    ..Default::default()
  };
  let data = Box::pin(search(Query(form), context, local_user_view)).await?;
  Ok(Json(convert_search_response(data.0, type_)))
}

pub(crate) async fn resolve_object_v3(
  data: Query<ResolveObject>,
  context: ApubData<LemmyContext>,
  local_user_view: Option<LocalUserView>,
) -> LemmyResult<Json<ResolveObjectResponseV3>> {
  let res = resolve_object(data, context, local_user_view).await?;
  let conv = convert_resolve_object_response(res.0);
  Ok(Json(conv))
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

pub(crate) async fn read_person_v3(
  data: Query<GetPersonDetailsV3>,
  context: ApubData<LemmyContext>,
  local_user_view: Option<LocalUserView>,
) -> LemmyResult<Json<GetPersonDetailsResponseV3>> {
  let GetPersonDetailsV3 {
    person_id,
    username,
    limit,
    community_id,
    ..
  } = data.0;
  let data = GetPersonDetails {
    person_id: person_id.map(|id| PersonId(id.0)),
    username: username.clone(),
  };
  let res = read_person(Query(data), context.clone(), local_user_view.clone())
    .await?
    .0;
  let content = list_person_content(
    Query(ListPersonContent {
      type_: None,
      person_id: person_id.map(|id| PersonId(id.0)),
      username: username.clone(),
      community_id: community_id.map(|id| CommunityId(id.0)),
      community_name: None,
      page_cursor: None,
      limit,
    }),
    context,
    local_user_view,
  )
  .await?
  .0;
  let mut posts = vec![];
  let mut comments = vec![];
  for item in content.items {
    match item {
      PostCommentCombinedView::Post(pv) => posts.push(convert_post_view(pv)),
      PostCommentCombinedView::Comment(cv) => comments.push(convert_comment_view(cv)),
    }
  }
  Ok(Json(GetPersonDetailsResponseV3 {
    person_view: convert_person_view(res.person_view),
    site: res.site.map(convert_site),
    comments,
    posts,
    moderates: res
      .moderates
      .into_iter()
      .map(|m| CommunityModeratorViewV3 {
        community: convert_community(m.community),
        moderator: convert_person(m.moderator).0,
      })
      .collect(),
  }))
}

pub(crate) async fn follow_community_v3(
  data: Json<FollowCommunity>,
  context: ApubData<LemmyContext>,
  local_user_view: LocalUserView,
) -> LemmyResult<Json<CommunityResponseV3>> {
  let res = follow_community(data, context, local_user_view).await?;
  convert_community_response(res)
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
    multi_community_id: None,
    search_term: None,
    search_title_only: None,
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

pub(crate) async fn create_community_v3(
  Json(data): Json<CreateCommunityV3>,
  context: ApubData<LemmyContext>,
  local_user_view: LocalUserView,
) -> LemmyResult<Json<CommunityResponseV3>> {
  let CreateCommunityV3 {
    name,
    title,
    description,
    nsfw,
    posting_restricted_to_mods,
    ..
  } = data;
  let data = CreateCommunity {
    name,
    title: Some(title),
    summary: description,
    nsfw,
    posting_restricted_to_mods,
    ..Default::default()
  };
  let res = Box::pin(create_community(Json(data), context, local_user_view)).await?;
  convert_community_response(res)
}

pub(crate) async fn delete_community_v3(
  Json(data): Json<DeleteCommunityV3>,
  context: ApubData<LemmyContext>,
  local_user_view: LocalUserView,
) -> LemmyResult<Json<CommunityResponseV3>> {
  let DeleteCommunityV3 {
    community_id,
    deleted,
  } = data;
  let data = DeleteCommunity {
    community_id: CommunityId(community_id.0),
    deleted,
  };
  let res = delete_community(Json(data), context, local_user_view).await?;
  convert_community_response(res)
}

pub(crate) async fn remove_community_v3(
  Json(data): Json<RemoveCommunityV3>,
  context: ApubData<LemmyContext>,
  local_user_view: LocalUserView,
) -> LemmyResult<Json<CommunityResponseV3>> {
  let RemoveCommunityV3 {
    community_id,
    removed,
    reason,
  } = data;
  let data = RemoveCommunity {
    community_id: CommunityId(community_id.0),
    removed,
    reason: reason.unwrap_or_default(),
  };
  let res = remove_community(Json(data), context, local_user_view).await?;
  convert_community_response(res)
}

pub(crate) async fn ban_from_community_v3(
  Json(data): Json<BanFromCommunityV3>,
  context: ApubData<LemmyContext>,
  local_user_view: LocalUserView,
) -> LemmyResult<Json<BanFromCommunityResponseV3>> {
  let BanFromCommunityV3 {
    community_id,
    person_id,
    ban,
    remove_data,
    reason,
    expires,
  } = data;
  let data = BanFromCommunity {
    community_id: CommunityId(community_id.0),
    person_id: PersonId(person_id.0),
    ban,
    remove_or_restore_data: remove_data,
    reason: reason.unwrap_or_default(),
    expires_at: expires,
  };
  let res = ban_from_community(Json(data), context, local_user_view)
    .await?
    .0;
  Ok(Json(BanFromCommunityResponseV3 {
    person_view: convert_person_view(res.person_view),
    banned: ban,
  }))
}

pub(crate) async fn add_mod_to_community_v3(
  Json(data): Json<AddModToCommunityV3>,
  context: ApubData<LemmyContext>,
  local_user_view: LocalUserView,
) -> LemmyResult<Json<AddModToCommunityResponseV3>> {
  let sql = AddModToCommunity {
    community_id: CommunityId(data.community_id.0),
    person_id: PersonId(data.person_id.0),
    added: data.added,
  };
  let res = add_mod_to_community(Json(sql), context, local_user_view)
    .await?
    .0;
  Ok(Json(AddModToCommunityResponseV3 {
    moderators: res
      .moderators
      .into_iter()
      .map(|m| CommunityModeratorViewV3 {
        community: convert_community(m.community),
        moderator: convert_person(m.moderator).0,
      })
      .collect(),
  }))
}

pub(crate) async fn transfer_community_v3(
  Json(data): Json<TransferCommunityV3>,
  context: Data<LemmyContext>,
  local_user_view: LocalUserView,
) -> LemmyResult<Json<CommunityResponseV3>> {
  let data = TransferCommunity {
    community_id: CommunityId(data.community_id.0),
    person_id: PersonId(data.person_id.0),
  };
  let res = Box::pin(transfer_community(Json(data), context, local_user_view))
    .await?
    .0;
  Ok(Json(CommunityResponseV3 {
    community_view: convert_community_view(res.community_view),
    discussion_languages: convert_language_ids(res.discussion_languages),
  }))
}

pub(crate) async fn list_post_likes_v3(
  Query(data): Query<ListPostLikesV3>,
  context: Data<LemmyContext>,
  local_user_view: LocalUserView,
) -> LemmyResult<Json<ListPostLikesResponseV3>> {
  let ListPostLikesV3 { post_id, limit, .. } = data;
  let form = ListPostLikes {
    post_id: PostId(post_id.0),
    page_cursor: None,
    limit,
  };
  let votes = list_post_likes(Query(form), context, local_user_view).await?;
  Ok(Json(ListPostLikesResponseV3 {
    post_likes: convert_votes_to_v3(votes),
  }))
}

pub(crate) async fn list_comment_likes_v3(
  Query(data): Query<ListCommentLikesV3>,
  context: Data<LemmyContext>,
  local_user_view: LocalUserView,
) -> LemmyResult<Json<ListCommentLikesResponseV3>> {
  let ListCommentLikesV3 {
    comment_id, limit, ..
  } = data;
  let form = ListCommentLikes {
    comment_id: CommentId(comment_id.0),
    page_cursor: None,
    limit,
  };
  let votes = list_comment_likes(Query(form), context, local_user_view).await?;
  Ok(Json(ListCommentLikesResponseV3 {
    comment_likes: convert_votes_to_v3(votes),
  }))
}

pub(crate) async fn remove_post_v3(
  Json(data): Json<RemovePostV3>,
  context: ApubData<LemmyContext>,
  local_user_view: LocalUserView,
) -> LemmyResult<Json<PostResponseV3>> {
  let RemovePostV3 {
    post_id,
    removed,
    reason,
  } = data;
  let data = RemovePost {
    post_id: PostId(post_id.0),
    removed,
    reason: reason.unwrap_or_default(),
    remove_children: None,
  };
  let res = remove_post(Json(data), context, local_user_view).await?;
  convert_post_response(res)
}

pub(crate) async fn mark_post_as_read_v3(
  Json(data): Json<MarkPostAsReadV3>,
  context: Data<LemmyContext>,
  local_user_view: LocalUserView,
) -> LemmyResult<Json<PostResponseV3>> {
  let post_ids: Vec<PostId> = data.post_ids.iter().map(|id| PostId(id.0)).collect();
  PostActions::mark_as_read(&mut context.pool(), local_user_view.person.id, &post_ids).await?;
  let last = post_ids.last().copied().ok_or(LemmyErrorType::NoIdGiven)?;
  let form = GetPost {
    id: Some(last),
    comment_id: None,
  };
  let post = get_post(Query(form), context, Some(local_user_view.clone())).await?;
  Ok(Json(PostResponseV3 {
    post_view: convert_post_view(post.0.post_view),
  }))
}

pub(crate) async fn hide_post_v3(
  Json(data): Json<HidePostV3>,
  context: Data<LemmyContext>,
  local_user_view: LocalUserView,
) -> LemmyResult<Json<PostResponseV3>> {
  let HidePostV3 { post_ids, hide } = data;
  let mut res = None;
  for id in post_ids {
    let form = HidePost {
      post_id: PostId(id.0),
      hide,
    };
    res = Some(hide_post(Json(form), context.clone(), local_user_view.clone()).await?);
  }
  let res = res.ok_or(LemmyErrorType::NoIdGiven)?;
  convert_post_response(res)
}

pub(crate) async fn lock_post_v3(
  Json(data): Json<LockPostV3>,
  context: ApubData<LemmyContext>,
  local_user_view: LocalUserView,
) -> LemmyResult<Json<PostResponseV3>> {
  let LockPostV3 { post_id, locked } = data;
  let form = LockPost {
    post_id: PostId(post_id.0),
    locked,
    reason: String::new(),
  };
  let res = lock_post(Json(form), context, local_user_view).await?;
  convert_post_response(res)
}

pub(crate) async fn feature_post_v3(
  Json(data): Json<FeaturePostV3>,
  context: ApubData<LemmyContext>,
  local_user_view: LocalUserView,
) -> LemmyResult<Json<PostResponseV3>> {
  let FeaturePostV3 {
    post_id,
    featured,
    feature_type,
  } = data;
  let feature_type = match feature_type {
    PostFeatureTypeV3::Local => PostFeatureType::Local,
    PostFeatureTypeV3::Community => PostFeatureType::Community,
  };
  let form = FeaturePost {
    post_id: PostId(post_id.0),
    featured,
    feature_type,
  };
  let res = feature_post(Json(form), context, local_user_view).await?;
  convert_post_response(res)
}

pub(crate) async fn get_link_metadata_v3(
  data: Query<GetSiteMetadata>,
  context: ApubData<LemmyContext>,
  local_user_view: LocalUserView,
) -> LemmyResult<Json<GetSiteMetadataResponseV3>> {
  let res = get_link_metadata(data, context, local_user_view).await?.0;
  // response is unchanged, but v3 has nested private field so we cant convert it normally
  let value = serde_json::to_value(res)?;
  let metadata: GetSiteMetadataResponseV3 = serde_json::from_value(value)?;
  Ok(Json(metadata))
}

pub(crate) async fn remove_comment_v3(
  Json(data): Json<RemoveCommentV3>,
  context: ApubData<LemmyContext>,
  local_user_view: LocalUserView,
) -> LemmyResult<Json<CommentResponseV3>> {
  let RemoveCommentV3 {
    comment_id,
    removed,
    reason,
  } = data;
  let form = RemoveComment {
    comment_id: CommentId(comment_id.0),
    removed,
    reason: reason.unwrap_or_default(),
    remove_children: None,
  };
  let res = remove_comment(Json(form), context, local_user_view).await?;
  convert_comment_response(res)
}

pub(crate) async fn distinguish_comment_v3(
  Json(data): Json<DistinguishCommentV3>,
  context: ApubData<LemmyContext>,
  local_user_view: LocalUserView,
) -> LemmyResult<Json<CommentResponseV3>> {
  let data_v4 = DistinguishComment {
    comment_id: CommentId(data.comment_id.0),
    distinguished: data.distinguished,
  };
  let res = distinguish_comment(Json(data_v4), context, local_user_view).await?;
  convert_comment_response(res)
}

pub(crate) async fn create_custom_emoji_v3(
  Json(data): Json<CreateCustomEmojiV3>,
  context: ApubData<LemmyContext>,
  local_user_view: LocalUserView,
) -> LemmyResult<Json<CustomEmojiResponseV3>> {
  let CreateCustomEmojiV3 {
    category,
    shortcode,
    image_url,
    alt_text,
    keywords,
  } = data;
  let form = CreateCustomEmoji {
    category,
    shortcode,
    image_url: image_url.into(),
    alt_text,
    keywords,
  };
  let res = create_custom_emoji(Json(form), context, local_user_view)
    .await?
    .0;
  convert_custom_emoji_response(res)
}

pub(crate) async fn edit_custom_emoji_v3(
  Json(data): Json<EditCustomEmojiV3>,
  context: ApubData<LemmyContext>,
  local_user_view: LocalUserView,
) -> LemmyResult<Json<CustomEmojiResponseV3>> {
  let EditCustomEmojiV3 {
    id,
    category,
    image_url,
    alt_text,
    keywords,
    ..
  } = data;
  let form = EditCustomEmoji {
    id: CustomEmojiId(id.0),
    category: Some(category),
    shortcode: None,
    image_url: Some(image_url.into()),
    alt_text: Some(alt_text),
    keywords: Some(keywords),
  };
  let res = edit_custom_emoji(Json(form), context, local_user_view)
    .await?
    .0;
  convert_custom_emoji_response(res)
}

pub(crate) async fn delete_custom_emoji_v3(
  Json(data): Json<DeleteCustomEmojiV3>,
  context: ApubData<LemmyContext>,
  local_user_view: LocalUserView,
) -> LemmyResult<Json<SuccessResponseV3>> {
  let DeleteCustomEmojiV3 { id } = data;
  let form = DeleteCustomEmoji {
    id: CustomEmojiId(id.0),
  };
  delete_custom_emoji(Json(form), context, local_user_view).await?;
  Ok(Json(SuccessResponseV3::default()))
}

/// Actix redirect doesn't work with path segments, so it needs to be extracted manually.
pub(crate) async fn redirect_pictrs_image(req: HttpRequest, path: Path<String>) -> Redirect {
  let file = path.into_inner();
  let new_path = format!("/api/v4/image/{file}");

  redirect_with_query_string(req, &new_path).await
}

/// Actix redirect does not by default pass query params.
/// This is necessary for the image proxy endpoint, which needs to pass ?url=...
pub(crate) async fn redirect_with_query_string(req: HttpRequest, new_path: &str) -> Redirect {
  let query = req.query_string();

  // Construct target URL and append original query string
  let target = if query.is_empty() {
    new_path.to_string()
  } else {
    format!("{new_path}?{query}")
  };
  Redirect::to(target)
}
