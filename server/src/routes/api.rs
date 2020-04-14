use crate::api::comment::*;
use crate::api::community::*;
use crate::api::post::*;
use crate::api::site::*;
use crate::api::user::*;
use crate::api::{Oper, Perform};
use actix_web::{web, HttpResponse};
use diesel::r2d2::{ConnectionManager, Pool};
use diesel::PgConnection;
use failure::Error;
use serde::Serialize;

type DbParam = web::Data<Pool<ConnectionManager<PgConnection>>>;

#[rustfmt::skip]
pub fn config(cfg: &mut web::ServiceConfig) {
  cfg
    // Site
    .route("/api/v1/site", web::get().to(route_get::<GetSite, GetSiteResponse>))
    .route("/api/v1/categories", web::get().to(route_get::<ListCategories, ListCategoriesResponse>))
    .route("/api/v1/modlog", web::get().to(route_get::<GetModlog, GetModlogResponse>))
    .route("/api/v1/search", web::get().to(route_get::<Search, SearchResponse>))
    // Community
    .route("/api/v1/community", web::post().to(route_post::<CreateCommunity, CommunityResponse>))
    .route("/api/v1/community", web::get().to(route_get::<GetCommunity, GetCommunityResponse>))
    .route("/api/v1/community", web::put().to(route_post::<EditCommunity, CommunityResponse>))
    .route("/api/v1/community/list", web::get().to(route_get::<ListCommunities, ListCommunitiesResponse>))
    .route("/api/v1/community/follow", web::post().to(route_post::<FollowCommunity, CommunityResponse>))
    // Post
    .route("/api/v1/post", web::post().to(route_post::<CreatePost, PostResponse>))
    .route("/api/v1/post", web::put().to(route_post::<EditPost, PostResponse>))
    .route("/api/v1/post", web::get().to(route_get::<GetPost, GetPostResponse>))
    .route("/api/v1/post/list", web::get().to(route_get::<GetPosts, GetPostsResponse>))
    .route("/api/v1/post/like", web::post().to(route_post::<CreatePostLike, PostResponse>))
    .route("/api/v1/post/save", web::put().to(route_post::<SavePost, PostResponse>))
    // Comment
    .route("/api/v1/comment", web::post().to(route_post::<CreateComment, CommentResponse>))
    .route("/api/v1/comment", web::put().to(route_post::<EditComment, CommentResponse>))
    .route("/api/v1/comment/like", web::post().to(route_post::<CreateCommentLike, CommentResponse>))
    .route("/api/v1/comment/save", web::put().to(route_post::<SaveComment, CommentResponse>))
    // User
    .route("/api/v1/user", web::get().to(route_get::<GetUserDetails, GetUserDetailsResponse>))
    .route("/api/v1/user/mention", web::get().to(route_get::<GetUserMentions, GetUserMentionsResponse>))
    .route("/api/v1/user/mention", web::put().to(route_post::<EditUserMention, UserMentionResponse>))
    .route("/api/v1/user/replies", web::get().to(route_get::<GetReplies, GetRepliesResponse>))
    .route("/api/v1/user/followed_communities", web::get().to(route_get::<GetFollowedCommunities, GetFollowedCommunitiesResponse>))
    // Mod actions
    .route("/api/v1/community/transfer", web::post().to(route_post::<TransferCommunity, GetCommunityResponse>))
    .route("/api/v1/community/ban_user", web::post().to(route_post::<BanFromCommunity, BanFromCommunityResponse>))
    .route("/api/v1/community/mod", web::post().to(route_post::<AddModToCommunity, AddModToCommunityResponse>))
    // Admin actions
    .route("/api/v1/site", web::post().to(route_post::<CreateSite, SiteResponse>))
    .route("/api/v1/site", web::put().to(route_post::<EditSite, SiteResponse>))
    .route("/api/v1/site/transfer", web::post().to(route_post::<TransferSite, GetSiteResponse>))
    .route("/api/v1/site/config", web::get().to(route_get::<GetSiteConfig, GetSiteConfigResponse>))
    .route("/api/v1/site/config", web::put().to(route_post::<SaveSiteConfig, GetSiteConfigResponse>))
    .route("/api/v1/admin/add", web::post().to(route_post::<AddAdmin, AddAdminResponse>))
    .route("/api/v1/user/ban", web::post().to(route_post::<BanUser, BanUserResponse>))
    // User account actions
    .route("/api/v1/user/login", web::post().to(route_post::<Login, LoginResponse>))
    .route("/api/v1/user/register", web::post().to(route_post::<Register, LoginResponse>))
    .route("/api/v1/user/delete_account", web::post().to(route_post::<DeleteAccount, LoginResponse>))
    .route("/api/v1/user/password_reset", web::post().to(route_post::<PasswordReset, PasswordResetResponse>))
    .route("/api/v1/user/password_change", web::post().to(route_post::<PasswordChange, LoginResponse>))
    .route("/api/v1/user/mark_all_as_read", web::post().to(route_post::<MarkAllAsRead, GetRepliesResponse>))
    .route("/api/v1/user/save_user_settings", web::put().to(route_post::<SaveUserSettings, LoginResponse>));
}

fn perform<Request, Response>(data: Request, db: DbParam) -> Result<HttpResponse, Error>
where
  Response: Serialize,
  Oper<Request>: Perform<Response>,
{
  let conn = match db.get() {
    Ok(c) => c,
    Err(e) => return Err(format_err!("{}", e)),
  };
  let oper: Oper<Request> = Oper::new(data);
  let response = oper.perform(&conn);
  Ok(HttpResponse::Ok().json(response?))
}

async fn route_get<Data, Response>(
  data: web::Query<Data>,
  db: DbParam,
) -> Result<HttpResponse, Error>
where
  Data: Serialize,
  Response: Serialize,
  Oper<Data>: Perform<Response>,
{
  perform::<Data, Response>(data.0, db)
}

async fn route_post<Data, Response>(
  data: web::Json<Data>,
  db: DbParam,
) -> Result<HttpResponse, Error>
where
  Data: Serialize,
  Response: Serialize,
  Oper<Data>: Perform<Response>,
{
  perform::<Data, Response>(data.0, db)
}
