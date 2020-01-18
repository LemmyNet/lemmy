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
    .route("/api/v1/site", web::get().to(route::<GetSite, GetSiteResponse>))
    .route("/api/v1/categories", web::get().to(route::<ListCategories, ListCategoriesResponse>))
    .route("/api/v1/modlog", web::get().to(route::<GetModlog, GetModlogResponse>))
    .route("/api/v1/search", web::post().to(route::<Search, SearchResponse>))
    // Community
    .route("/api/v1/community", web::post().to(route::<CreateCommunity, CommunityResponse>))
    .route("/api/v1/community", web::get().to(route::<GetCommunity, GetCommunityResponse>))
    .route("/api/v1/community", web::put().to(route::<EditCommunity, CommunityResponse>))
    .route("/api/v1/community/list", web::get().to(route::<ListCommunities, ListCommunitiesResponse>))
    .route("/api/v1/community/follow", web::post().to(route::<FollowCommunity, CommunityResponse>))
    // Post
    .route("/api/v1/post", web::post().to(route::<CreatePost, PostResponse>))
    .route("/api/v1/post", web::put().to(route::<EditPost, PostResponse>))
    .route("/api/v1/post", web::get().to(route::<GetPost, GetPostResponse>))
    .route("/api/v1/post/list", web::get().to(route::<GetPosts, GetPostsResponse>))
    .route("/api/v1/post/like", web::post().to(route::<CreatePostLike, CreatePostLikeResponse>))
    .route("/api/v1/post/save", web::post().to(route::<SavePost, PostResponse>))
    .route("/api/v1/post/replies", web::get().to(route::<GetReplies, GetRepliesResponse>))
    // Comment
    .route("/api/v1/comment", web::post().to(route::<CreateComment, CommentResponse>))
    .route("/api/v1/comment", web::put().to(route::<EditComment, CommentResponse>))
    .route("/api/v1/comment/like", web::post().to(route::<CreateCommentLike, CommentResponse>))
    .route("/api/v1/comment/save", web::post().to(route::<SaveComment, CommentResponse>))
    // User
    .route("/api/v1/user", web::get().to(route::<GetUserDetails, GetUserDetailsResponse>))
    .route("/api/v1/user/mentions", web::get().to(route::<GetUserMentions, GetUserMentionsResponse>))
    .route("/api/v1/user/mentions", web::put().to(route::<EditUserMention, UserMentionResponse>))
    .route("/api/v1/user/followed-communities", web::get().to(route::<GetFollowedCommunities, GetFollowedCommunitiesResponse>))
    // Mod actions
    .route("/api/v1/community/transfer", web::post().to(route::<TransferCommunity, GetCommunityResponse>))
    .route("/api/v1/community/ban-user", web::post().to(route::<BanFromCommunity, BanFromCommunityResponse>))
    .route("/api/v1/community/mod", web::post().to(route::<AddModToCommunity, AddModToCommunityResponse>))
    // Admin actions
    .route("/api/v1/site", web::post().to(route::<CreateSite, SiteResponse>))
    .route("/api/v1/site", web::put().to(route::<EditSite, SiteResponse>))
    .route("/api/v1/site/transfer", web::post().to(route::<TransferSite, GetSiteResponse>))
    .route("/api/v1/admin/add", web::post().to(route::<AddAdmin, AddAdminResponse>))
    .route("/api/v1/user/ban", web::post().to(route::<BanUser, BanUserResponse>))
    // User account actions
    .route("/api/v1/user/login", web::post().to(route::<Login, LoginResponse>))
    .route("/api/v1/user/register", web::post().to(route::<Register, LoginResponse>))
    .route("/api/v1/user/delete_account", web::post().to(route::<DeleteAccount, LoginResponse>))
    .route("/api/v1/user/password_reset", web::post().to(route::<PasswordReset, PasswordResetResponse>))
    .route("/api/v1/user/password_change", web::post().to(route::<PasswordChange, LoginResponse>))
    .route("/api/v1/user/mark_all_as_read", web::post().to(route::<MarkAllAsRead, GetRepliesResponse>))
    .route("/api/v1/user/save_user_settings", web::post().to(route::<SaveUserSettings, LoginResponse>));
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

async fn route<Data, Response>(data: web::Query<Data>, db: DbParam) -> Result<HttpResponse, Error>
where
  Data: Serialize,
  Response: Serialize,
  Oper<Data>: Perform<Response>,
{
  perform::<Data, Response>(data.0, db)
}
