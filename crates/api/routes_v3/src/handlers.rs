use crate::convert::{
  convert_comment_view,
  convert_community_view,
  convert_my_user,
  convert_person_view,
  convert_post_listing_sort,
  convert_post_listing_type,
  convert_post_view,
  convert_score,
  convert_search_response,
  convert_sensitive2,
  convert_site_view,
};
use activitypub_federation::config::Data as ApubData;
use actix_web::{HttpRequest, HttpResponse, web::*};
use lemmy_api::{
  comment::like::like_comment,
  federation::{
    list_comments::list_comments,
    list_posts::list_posts,
    resolve_object::resolve_object,
    search::search,
  },
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
    newtypes::LanguageId as LanguageIdV3,
    source::{
      language::Language as LanguageV3,
      local_site_url_blocklist::LocalSiteUrlBlocklist as LocalSiteUrlBlocklistV3,
      tagline::Tagline as TaglineV3,
    },
  },
  person::LoginResponse as LoginResponseV3,
  post::{
    CreatePost as CreatePostV3,
    CreatePostLike as CreatePostLikeV3,
    GetPostResponse as GetPostResponseV3,
    GetPosts as GetPostsV3,
    GetPostsResponse as GetPostsResponseV3,
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
  comment::create::create_comment,
  post::{create::create_post, read::get_post},
  site::read::get_site,
  user::my_user::get_my_user,
};
use lemmy_api_utils::context::LemmyContext;
use lemmy_db_schema::newtypes::{CommentId, CommunityId, LanguageId, PersonId, PostId};
use lemmy_db_views_comment::api::{CreateComment, CreateCommentLike, GetComments};
use lemmy_db_views_local_user::LocalUserView;
use lemmy_db_views_post::api::{CreatePost, CreatePostLike, GetPosts};
use lemmy_db_views_search_combined::{Search, api::GetPost};
use lemmy_db_views_site::api::{GetSiteResponse, Login, LoginResponse, ResolveObject};
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
    ..
  } = datav3.0;
  let data = GetPosts {
    type_: type_.map(convert_post_listing_type),
    sort: sort.map(convert_post_listing_sort),
    time_range_seconds: Default::default(),
    community_id: community_id.map(|id| CommunityId(id.0)),
    community_name,
    show_hidden,
    show_read,
    show_nsfw,
    limit,
    ..Default::default()
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

pub(crate) async fn list_comments_v3(
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
    discussion_languages: discussion_languages
      .into_iter()
      .map(|id| LanguageIdV3(id.0))
      .collect(),
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
  let LoginResponse {
    jwt,
    registration_created,
    verify_email_sent,
  } = login(data, req, context).await?.0;
  Ok(Json(LoginResponseV3 {
    jwt: jwt.map(convert_sensitive2),
    registration_created,
    verify_email_sent,
  }))
}

pub(crate) async fn like_comment_v3(
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

pub(crate) async fn like_post_v3(
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

pub(crate) async fn create_comment_v3(
  data: Json<CreateComment>,
  context: ApubData<LemmyContext>,
  local_user_view: LocalUserView,
) -> LemmyResult<Json<CommentResponseV3>> {
  let res = Box::pin(create_comment(data, context, local_user_view)).await?;
  Ok(Json(CommentResponseV3 {
    comment_view: convert_comment_view(res.0.comment_view),
    recipient_ids: vec![],
  }))
}

pub(crate) async fn create_post_v3(
  data: Json<CreatePostV3>,
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
  } = data.0;
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
  Ok(Json(PostResponseV3 {
    post_view: convert_post_view(res.0.post_view),
  }))
}

pub(crate) async fn search_v3(
  data: Query<SearchV3>,
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
  } = data.0;
  let data = Search {
    q,
    community_id: community_id.map(|i| CommunityId(i.0)),
    community_name,
    creator_id: creator_id.map(|i| PersonId(i.0)),
    limit,
    ..Default::default()
  };
  let res = search(Query(data), context, local_user_view).await?;
  Ok(Json(convert_search_response(res.0.results, type_)))
}

pub(crate) async fn resolve_object_v3(
  data: Query<ResolveObject>,
  context: ApubData<LemmyContext>,
  local_user_view: Option<LocalUserView>,
) -> LemmyResult<Json<ResolveObjectResponseV3>> {
  let res = resolve_object(data, context, local_user_view).await?;
  let mut conv = convert_search_response(res.0.results, None);
  Ok(Json(ResolveObjectResponseV3 {
    comment: conv.comments.pop(),
    post: conv.posts.pop(),
    community: conv.communities.pop(),
    person: conv.users.pop(),
  }))
}
