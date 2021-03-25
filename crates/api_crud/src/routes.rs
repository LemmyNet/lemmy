use crate::PerformCrud;
use actix_web::{error::ErrorBadRequest, *};
use lemmy_api_common::{comment::*, community::*, person::*, post::*, site::*};
use lemmy_utils::rate_limit::RateLimit;
use lemmy_websocket::LemmyContext;
use serde::Deserialize;

pub fn config(cfg: &mut web::ServiceConfig, rate_limit: &RateLimit) {
  cfg
    .service(
      web::scope("/api/v2")
        // Site
        .service(
          web::scope("/site")
            .wrap(rate_limit.message())
            .route("", web::get().to(route_get::<GetSite>))
            // Admin Actions
            .route("", web::post().to(route_post::<CreateSite>))
            .route("", web::put().to(route_post::<EditSite>)),
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
            .route("/delete", web::post().to(route_post::<DeleteCommunity>))
            // Mod Actions
            .route("/remove", web::post().to(route_post::<RemoveCommunity>)),
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
            .route("/delete", web::post().to(route_post::<DeletePost>))
            .route("/remove", web::post().to(route_post::<RemovePost>))
            .route("/list", web::get().to(route_get::<GetPosts>)),
        )
        // Comment
        .service(
          web::scope("/comment")
            .wrap(rate_limit.message())
            .route("", web::post().to(route_post::<CreateComment>))
            .route("", web::put().to(route_post::<EditComment>))
            .route("/delete", web::post().to(route_post::<DeleteComment>))
            .route("/remove", web::post().to(route_post::<RemoveComment>))
            .route("/list", web::get().to(route_get::<GetComments>)),
        ),
    )
    // Private Message
    .service(
      web::scope("/private_message")
        .wrap(rate_limit.message())
        .route("/list", web::get().to(route_get::<GetPrivateMessages>))
        .route("", web::post().to(route_post::<CreatePrivateMessage>))
        .route("", web::put().to(route_post::<EditPrivateMessage>))
        .route(
          "/delete",
          web::post().to(route_post::<DeletePrivateMessage>),
        ),
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
        .route("", web::get().to(route_get::<GetPersonDetails>))
        .route(
          "/delete_account",
          web::post().to(route_post::<DeleteAccount>),
        ),
    );
}

async fn perform<Request>(
  data: Request,
  context: web::Data<LemmyContext>,
) -> Result<HttpResponse, Error>
where
  Request: PerformCrud,
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
  Data: Deserialize<'a> + Send + 'static + PerformCrud,
{
  perform::<Data>(data.0, context).await
}

async fn route_post<'a, Data>(
  data: web::Json<Data>,
  context: web::Data<LemmyContext>,
) -> Result<HttpResponse, Error>
where
  Data: Deserialize<'a> + Send + 'static + PerformCrud,
{
  perform::<Data>(data.0, context).await
}
