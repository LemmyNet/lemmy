use crate::Perform;
use actix_web::{error::ErrorBadRequest, *};
use lemmy_api_common::{comment::*, community::*, person::*, post::*, site::*, websocket::*};
use lemmy_utils::rate_limit::RateLimit;
use lemmy_websocket::{routes::chat_route, LemmyContext};
use serde::Deserialize;

pub fn config(cfg: &mut web::ServiceConfig, rate_limit: &RateLimit) {
  cfg.service(
    web::scope("/api/v2")
      // Websockets
      .service(web::resource("/ws").to(chat_route))
      // Site
      .service(
        web::scope("/site")
          .wrap(rate_limit.message())
          // Admin Actions
          .route("/transfer", web::post().to(route_post::<TransferSite>))
          .route("/config", web::get().to(route_get::<GetSiteConfig>))
          .route("/config", web::put().to(route_post::<SaveSiteConfig>)),
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
        web::scope("/community")
          .wrap(rate_limit.message())
          .route("/follow", web::post().to(route_post::<FollowCommunity>))
          .route("/transfer", web::post().to(route_post::<TransferCommunity>))
          .route("/ban_user", web::post().to(route_post::<BanFromCommunity>))
          .route("/mod", web::post().to(route_post::<AddModToCommunity>))
          .route("/join", web::post().to(route_post::<CommunityJoin>))
          .route("/mod/join", web::post().to(route_post::<ModJoin>)),
      )
      // Post
      .service(
        web::scope("/post")
          .wrap(rate_limit.message())
          .route("/lock", web::post().to(route_post::<LockPost>))
          .route("/sticky", web::post().to(route_post::<StickyPost>))
          .route("/like", web::post().to(route_post::<CreatePostLike>))
          .route("/save", web::put().to(route_post::<SavePost>))
          .route("/join", web::post().to(route_post::<PostJoin>))
          .route("/report", web::post().to(route_post::<CreatePostReport>))
          .route(
            "/report/resolve",
            web::put().to(route_post::<ResolvePostReport>),
          )
          .route("/report/list", web::get().to(route_get::<ListPostReports>)),
      )
      // Comment
      .service(
        web::scope("/comment")
          .wrap(rate_limit.message())
          .route(
            "/mark_as_read",
            web::post().to(route_post::<MarkCommentAsRead>),
          )
          .route("/like", web::post().to(route_post::<CreateCommentLike>))
          .route("/save", web::put().to(route_post::<SaveComment>))
          .route("/report", web::post().to(route_post::<CreateCommentReport>))
          .route(
            "/report/resolve",
            web::put().to(route_post::<ResolveCommentReport>),
          )
          .route(
            "/report/list",
            web::get().to(route_get::<ListCommentReports>),
          ),
      )
      // Private Message
      .service(
        web::scope("/private_message")
          .wrap(rate_limit.message())
          .route(
            "/mark_as_read",
            web::post().to(route_post::<MarkPrivateMessageAsRead>),
          ),
      )
      // User actions
      .service(
        web::scope("/user")
          .wrap(rate_limit.message())
          .route("/mention", web::get().to(route_get::<GetPersonMentions>))
          .route(
            "/mention/mark_as_read",
            web::post().to(route_post::<MarkPersonMentionAsRead>),
          )
          .route("/replies", web::get().to(route_get::<GetReplies>))
          .route(
            "/followed_communities",
            web::get().to(route_get::<GetFollowedCommunities>),
          )
          .route("/join", web::post().to(route_post::<UserJoin>))
          // Admin action. I don't like that it's in /user
          .route("/ban", web::post().to(route_post::<BanPerson>))
          // Account actions. I don't like that they're in /user maybe /accounts
          .route("/login", web::post().to(route_post::<Login>))
          .route("/get_captcha", web::get().to(route_get::<GetCaptcha>))
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
          )
          .route("/report_count", web::get().to(route_get::<GetReportCount>)),
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
  context: web::Data<LemmyContext>,
) -> Result<HttpResponse, Error>
where
  Request: Perform,
  Request: Send + 'static,
{
  let res = data
    .perform(&context, None)
    .await
    .map(|json| HttpResponse::Ok().json(json))
    .map_err(ErrorBadRequest)?;
  Ok(res)
}

async fn route_get<'a, Data>(
  data: web::Query<Data>,
  context: web::Data<LemmyContext>,
) -> Result<HttpResponse, Error>
where
  Data: Deserialize<'a> + Send + 'static + Perform,
{
  perform::<Data>(data.0, context).await
}

async fn route_post<'a, Data>(
  data: web::Json<Data>,
  context: web::Data<LemmyContext>,
) -> Result<HttpResponse, Error>
where
  Data: Deserialize<'a> + Send + 'static + Perform,
{
  perform::<Data>(data.0, context).await
}
