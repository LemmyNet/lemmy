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

pub fn config(cfg: &mut web::ServiceConfig) {
  cfg
    .route(
      "/api/v1/login",
      web::post().to(route::<Login, LoginResponse>),
    )
    .route(
      "/api/v1/register",
      web::post().to(route::<Register, LoginResponse>),
    )
    .route(
      "/api/v1/create_community",
      web::post().to(route::<CreateCommunity, CommunityResponse>),
    )
    .route(
      "/api/v1/create_post",
      web::post().to(route::<CreatePost, PostResponse>),
    )
    .route(
      "/api/v1/list_communities",
      web::get().to(route::<ListCommunities, ListCommunitiesResponse>),
    )
    .route(
      "/api/v1/list_categories",
      web::get().to(route::<ListCategories, ListCategoriesResponse>),
    )
    .route(
      "/api/v1/get_post",
      web::get().to(route::<GetPost, GetPostResponse>),
    )
    .route(
      "/api/v1/get_community",
      web::get().to(route::<GetCommunity, GetCommunityResponse>),
    )
    .route(
      "/api/v1/create_communent",
      web::post().to(route::<CreateComment, CommentResponse>),
    )
    .route(
      "/api/v1/edit_comment",
      web::post().to(route::<EditComment, CommentResponse>),
    )
    .route(
      "/api/v1/save_comment",
      web::post().to(route::<SaveComment, CommentResponse>),
    )
    .route(
      "/api/v1/create_comment_like",
      web::post().to(route::<CreateCommentLike, CommentResponse>),
    )
    .route(
      "/api/v1/get_posts",
      web::get().to(route::<GetPosts, GetPostsResponse>),
    )
    .route(
      "/api/v1/create_post_like",
      web::post().to(route::<CreatePostLike, CreatePostLikeResponse>),
    )
    .route(
      "/api/v1/edit_post",
      web::post().to(route::<EditPost, PostResponse>),
    )
    .route(
      "/api/v1/save_post",
      web::post().to(route::<SavePost, PostResponse>),
    )
    .route(
      "/api/v1/edit_community",
      web::post().to(route::<EditCommunity, CommunityResponse>),
    )
    .route(
      "/api/v1/follow_community",
      web::post().to(route::<FollowCommunity, CommunityResponse>),
    )
    .route(
      "/api/v1/get_followed_communities",
      web::get().to(route::<GetFollowedCommunities, GetFollowedCommunitiesResponse>),
    )
    .route(
      "/api/v1/get_user_details",
      web::get().to(route::<GetUserDetails, GetUserDetailsResponse>),
    )
    .route(
      "/api/v1/get_replies",
      web::get().to(route::<GetReplies, GetRepliesResponse>),
    )
    .route(
      "/api/v1/get_user_mentions",
      web::get().to(route::<GetUserMentions, GetUserMentionsResponse>),
    )
    .route(
      "/api/v1/edit_user_mention",
      web::post().to(route::<EditUserMention, UserMentionResponse>),
    )
    .route(
      "/api/v1/get_modlog",
      web::get().to(route::<GetModlog, GetModlogResponse>),
    )
    .route(
      "/api/v1/ban_from_community",
      web::post().to(route::<BanFromCommunity, BanFromCommunityResponse>),
    )
    .route(
      "/api/v1/add_mod_to_community",
      web::post().to(route::<AddModToCommunity, AddModToCommunityResponse>),
    )
    .route(
      "/api/v1/create_site",
      web::post().to(route::<CreateSite, SiteResponse>),
    )
    .route(
      "/api/v1/edit_site",
      web::post().to(route::<EditSite, SiteResponse>),
    )
    .route(
      "/api/v1/get_site",
      web::get().to(route::<GetSite, GetSiteResponse>),
    )
    .route(
      "/api/v1/add_admin",
      web::post().to(route::<AddAdmin, AddAdminResponse>),
    )
    .route(
      "/api/v1/ban_user",
      web::post().to(route::<BanUser, BanUserResponse>),
    )
    .route(
      "/api/v1/search",
      web::post().to(route::<Search, SearchResponse>),
    )
    .route(
      "/api/v1/mark_all_as_read",
      web::post().to(route::<MarkAllAsRead, GetRepliesResponse>),
    )
    .route(
      "/api/v1/save_user_settings",
      web::post().to(route::<SaveUserSettings, LoginResponse>),
    )
    .route(
      "/api/v1/transfer_community",
      web::post().to(route::<TransferCommunity, GetCommunityResponse>),
    )
    .route(
      "/api/v1/transfer_site",
      web::post().to(route::<TransferSite, GetSiteResponse>),
    )
    .route(
      "/api/v1/delete_account",
      web::post().to(route::<DeleteAccount, LoginResponse>),
    )
    .route(
      "/api/v1/password_reset",
      web::post().to(route::<PasswordReset, PasswordResetResponse>),
    )
    .route(
      "/api/v1/password_change",
      web::post().to(route::<PasswordChange, LoginResponse>),
    );
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
