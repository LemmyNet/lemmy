use crate::handlers::{
  block_community_v3,
  block_person_v3,
  create_comment_report_v3,
  create_comment_v3,
  create_post_report_v3,
  create_post_v3,
  delete_comment_v3,
  delete_post_v3,
  follow_community_v3,
  get_community_v3,
  get_post_v3,
  get_site_v3,
  like_comment_v3,
  like_post_v3,
  list_comments_v3,
  list_communities_v3,
  list_posts_v3,
  login_v3,
  logout_v3,
  mark_all_notifications_read_v3,
  register_v3,
  resolve_object_v3,
  save_comment_v3,
  save_post_v3,
  search_v3,
  unread_count_v3,
  update_comment_v3,
  update_post_v3,
};
use actix_web::{guard, web::*};
use lemmy_api::local_user::donation_dialog_shown::donation_dialog_shown;
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
        scope("/community")
          .wrap(rate_limit.message())
          .route("", get().to(get_community_v3))
          .route("/list", get().to(list_communities_v3))
          .route("/follow", post().to(follow_community_v3))
          .route("/block", post().to(block_community_v3)),
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
          .route("", put().to(update_post_v3))
          .route("/delete", post().to(delete_post_v3))
          .route("/list", get().to(list_posts_v3))
          .route("/like", post().to(like_post_v3))
          .route("/save", put().to(save_post_v3))
          .route("/report", post().to(create_post_report_v3)),
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
          .route("", put().to(update_comment_v3))
          .route("/delete", post().to(delete_comment_v3))
          .route("/like", post().to(like_comment_v3))
          .route("/list", get().to(list_comments_v3))
          .route("/save", put().to(save_comment_v3))
          .route("/report", post().to(create_comment_report_v3)),
      )
      .service(
        resource("/user/login")
          .guard(guard::Post())
          .wrap(rate_limit.register())
          .route(post().to(login_v3)),
      )
      .service(
        resource("/user/register")
          .guard(guard::Post())
          .wrap(rate_limit.register())
          .route(post().to(register_v3)),
      )
      .service(
        scope("/user")
          .wrap(rate_limit.message())
          .route("/logout", post().to(logout_v3))
          .route("/unread_count", get().to(unread_count_v3))
          .route("/block", post().to(block_person_v3))
          .route(
            "/mark_all_as_read",
            post().to(mark_all_notifications_read_v3),
          )
          .route("/donation_dialog_shown", post().to(donation_dialog_shown)),
      ),
  );
}
