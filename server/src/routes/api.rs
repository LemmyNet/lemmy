use crate::{
  api::{comment::*, community::*, post::*, site::*, user::*, Oper, Perform},
  rate_limit::RateLimit,
  routes::{ChatServerParam, DbPoolParam},
  websocket::WebsocketInfo,
};
use actix_web::{client::Client, error::ErrorBadRequest, *};
use serde::Serialize;

pub fn config(cfg: &mut web::ServiceConfig, rate_limit: &RateLimit) {
  cfg.service(
    web::scope("/api/v1")
      // Websockets
      .service(web::resource("/ws").to(super::websocket::chat_route))
      // Site
      .service(
        web::scope("/site")
          .wrap(rate_limit.message())
          .route("", web::get().to(route_get::<GetSite>))
          // Admin Actions
          .route("", web::post().to(route_post::<CreateSite>))
          .route("", web::put().to(route_post::<EditSite>))
          .route("/transfer", web::post().to(route_post::<TransferSite>))
          .route("/config", web::get().to(route_get::<GetSiteConfig>))
          .route("/config", web::put().to(route_post::<SaveSiteConfig>)),
      )
      .service(
        web::resource("/categories")
          .wrap(rate_limit.message())
          .route(web::get().to(route_get::<ListCategories>)),
      )
      .service(
        web::resource("/modlog")
          .wrap(rate_limit.message())
          .route(web::get().to(route_get::<GetModlog>)),
      )
      .service(
        web::resource("/search")
          .wrap(rate_limit.message())
          .route(web::get().to(route_get::<Search>)),
      )
      // Community
      .service(
        web::resource("/community")
          .guard(guard::Post())
          .wrap(rate_limit.register())
          .route(web::post().to(route_post::<CreateCommunity>)),
      )
      .service(
        web::scope("/community")
          .wrap(rate_limit.message())
          .route("", web::get().to(route_get::<GetCommunity>))
          .route("", web::put().to(route_post::<EditCommunity>))
          .route("/list", web::get().to(route_get::<ListCommunities>))
          .route("/follow", web::post().to(route_post::<FollowCommunity>))
          // Mod Actions
          .route("/transfer", web::post().to(route_post::<TransferCommunity>))
          .route("/ban_user", web::post().to(route_post::<BanFromCommunity>))
          .route("/mod", web::post().to(route_post::<AddModToCommunity>)),
      )
      // Post
      .service(
        // Handle POST to /post separately to add the post() rate limitter
        web::resource("/post")
          .guard(guard::Post())
          .wrap(rate_limit.post())
          .route(web::post().to(route_post::<CreatePost>)),
      )
      .service(
        web::scope("/post")
          .wrap(rate_limit.message())
          .route("", web::get().to(route_get::<GetPost>))
          .route("", web::put().to(route_post::<EditPost>))
          .route("/list", web::get().to(route_get::<GetPosts>))
          .route("/like", web::post().to(route_post::<CreatePostLike>))
          .route("/save", web::put().to(route_post::<SavePost>)),
      )
      // Comment
      .service(
        web::scope("/comment")
          .wrap(rate_limit.message())
          .route("", web::post().to(route_post::<CreateComment>))
          .route("", web::put().to(route_post::<EditComment>))
          .route("/like", web::post().to(route_post::<CreateCommentLike>))
          .route("/save", web::put().to(route_post::<SaveComment>)),
      )
      // Private Message
      .service(
        web::scope("/private_message")
          .wrap(rate_limit.message())
          .route("/list", web::get().to(route_get::<GetPrivateMessages>))
          .route("", web::post().to(route_post::<CreatePrivateMessage>))
          .route("", web::put().to(route_post::<EditPrivateMessage>)),
      )
      // User
      .service(
        // Account action, I don't like that it's in /user maybe /accounts
        // Handle /user/register separately to add the register() rate limitter
        web::resource("/user/register")
          .guard(guard::Post())
          .wrap(rate_limit.register())
          .route(web::post().to(route_post::<Register>)),
      )
      // User actions
      .service(
        web::scope("/user")
          .wrap(rate_limit.message())
          .route("", web::get().to(route_get::<GetUserDetails>))
          .route("/mention", web::get().to(route_get::<GetUserMentions>))
          .route("/mention", web::put().to(route_post::<EditUserMention>))
          .route("/replies", web::get().to(route_get::<GetReplies>))
          .route(
            "/followed_communities",
            web::get().to(route_get::<GetFollowedCommunities>),
          )
          // Admin action. I don't like that it's in /user
          .route("/ban", web::post().to(route_post::<BanUser>))
          // Account actions. I don't like that they're in /user maybe /accounts
          .route("/login", web::post().to(route_post::<Login>))
          .route(
            "/delete_account",
            web::post().to(route_post::<DeleteAccount>),
          )
          .route(
            "/password_reset",
            web::post().to(route_post::<PasswordReset>),
          )
          .route(
            "/password_change",
            web::post().to(route_post::<PasswordChange>),
          )
          // mark_all_as_read feels off being in this section as well
          .route(
            "/mark_all_as_read",
            web::post().to(route_post::<MarkAllAsRead>),
          )
          .route(
            "/save_user_settings",
            web::put().to(route_post::<SaveUserSettings>),
          ),
      )
      // Admin Actions
      .service(
        web::resource("/admin/add")
          .wrap(rate_limit.message())
          .route(web::post().to(route_post::<AddAdmin>)),
      ),
  );
}

async fn perform<Request>(
  data: Request,
  client: &Client,
  db: DbPoolParam,
  chat_server: ChatServerParam,
) -> Result<HttpResponse, Error>
where
  Oper<Request>: Perform,
  Request: Send + 'static,
{
  let ws_info = WebsocketInfo {
    chatserver: chat_server.get_ref().to_owned(),
    id: None,
  };

  let oper: Oper<Request> = Oper::new(data, client.clone());

  let res = oper
    .perform(&db, Some(ws_info))
    .await
    .map(|json| HttpResponse::Ok().json(json))
    .map_err(ErrorBadRequest)?;
  Ok(res)
}

async fn route_get<Data>(
  data: web::Query<Data>,
  client: web::Data<Client>,
  db: DbPoolParam,
  chat_server: ChatServerParam,
) -> Result<HttpResponse, Error>
where
  Data: Serialize + Send + 'static,
  Oper<Data>: Perform,
{
  perform::<Data>(data.0, &client, db, chat_server).await
}

async fn route_post<Data>(
  data: web::Json<Data>,
  client: web::Data<Client>,
  db: DbPoolParam,
  chat_server: ChatServerParam,
) -> Result<HttpResponse, Error>
where
  Data: Serialize + Send + 'static,
  Oper<Data>: Perform,
{
  perform::<Data>(data.0, &client, db, chat_server).await
}
