use crate::handlers::{
  create_comment_v3,
  create_post_v3,
  get_post_v3,
  get_site_v3,
  like_comment_v3,
  like_post_v3,
  list_comments_v3,
  list_posts_v3,
  login_v3,
  logout_v3,
  resolve_object_v3,
  search_v3,
};
use actix_web::{guard, web::*};
use lemmy_utils::rate_limit::RateLimit;

mod convert;
mod handlers;

pub fn config(cfg: &mut ServiceConfig, rate_limit: &RateLimit) {
  cfg.service(
    scope("/api/v3")
      .wrap(rate_limit.message())
      // Site
      .service(scope("/site").route("", get().to(get_site_v3)))
      .service(
        resource("/search")
          .wrap(rate_limit.search())
          .route(get().to(search_v3)),
      )
      .service(
        resource("/resolve_object")
          .wrap(rate_limit.message())
          .route(get().to(resolve_object_v3)),
      )
      .service(
        resource("/post")
          .guard(guard::Post())
          .wrap(rate_limit.post())
          .route(post().to(create_post_v3)),
      )
      .service(
        scope("/post")
          .wrap(rate_limit.message())
          .route("", get().to(get_post_v3))
          .route("/list", get().to(list_posts_v3))
          .route("/like", post().to(like_post_v3)),
      )
      .service(
        resource("/comment")
          .guard(guard::Post())
          .wrap(rate_limit.comment())
          .route(post().to(create_comment_v3)),
      )
      .service(
        scope("/comment")
          .wrap(rate_limit.message())
          .route("/like", post().to(like_comment_v3))
          .route("/list", get().to(list_comments_v3)),
      )
      .service(
        resource("/user/login")
          .guard(guard::Post())
          .wrap(rate_limit.register())
          .route(post().to(login_v3)),
      )
      .service(
        scope("/user")
          .wrap(rate_limit.message())
          .route("/logout", post().to(logout_v3)),
      ),
  );
}
