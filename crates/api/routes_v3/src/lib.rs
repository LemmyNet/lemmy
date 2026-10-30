use crate::handlers::{
  add_mod_to_community_v3,
  ban_from_community_v3,
  block_community_v3,
  block_person_v3,
  create_comment_report_v3,
  create_comment_v3,
  create_community_v3,
  create_custom_emoji_v3,
  create_post_report_v3,
  create_post_v3,
  delete_comment_v3,
  delete_community_v3,
  delete_custom_emoji_v3,
  delete_post_v3,
  distinguish_comment_v3,
  edit_custom_emoji_v3,
  feature_post_v3,
  follow_community_v3,
  get_community_v3,
  get_link_metadata_v3,
  get_post_v3,
  get_site_v3,
  hide_post_v3,
  like_comment_v3,
  like_post_v3,
  list_comment_likes_v3,
  list_comments_v3,
  list_communities_v3,
  list_post_likes_v3,
  list_posts_v3,
  lock_post_v3,
  login_v3,
  logout_v3,
  mark_all_notifications_read_v3,
  mark_post_as_read_v3,
  read_person_v3,
  register_v3,
  remove_comment_v3,
  remove_community_v3,
  remove_post_v3,
  resolve_object_v3,
  save_comment_v3,
  save_post_v3,
  search_v3,
  transfer_community_v3,
  unread_count_v3,
  update_comment_v3,
  update_post_v3,
};
use actix_web::{guard, web::*};
use lemmy_api::local_user::{
  mark_donation_dialog_shown::mark_donation_dialog_shown,
  validate_auth::validate_auth,
};
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
        resource("/community")
          .guard(guard::Post())
          .wrap(rate_limit.register())
          .route(post().to(create_community_v3)),
      )
      .service(
        scope("/community")
          .wrap(rate_limit.message())
          .route("", get().to(get_community_v3))
          .route("/list", get().to(list_communities_v3))
          .route("/follow", post().to(follow_community_v3))
          .route("/block", post().to(block_community_v3))
          .route("/delete", post().to(delete_community_v3))
          .route("/remove", post().to(remove_community_v3))
          .route("/ban_user", post().to(ban_from_community_v3))
          .route("/mod", post().to(add_mod_to_community_v3))
          .route("/transfer", post().to(transfer_community_v3)),
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
          .route("/like/list", get().to(list_post_likes_v3))
          .route("/save", put().to(save_post_v3))
          .route("/report", post().to(create_post_report_v3))
          .route("/remove", post().to(remove_post_v3))
          .route("/mark_as_read", post().to(mark_post_as_read_v3))
          .route("/hide", post().to(hide_post_v3))
          .route("/lock", post().to(lock_post_v3))
          .route("/feature", post().to(feature_post_v3))
          .route("/site_metadata", get().to(get_link_metadata_v3)),
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
          .route("/like/list", get().to(list_comment_likes_v3))
          .route("/list", get().to(list_comments_v3))
          .route("/save", put().to(save_comment_v3))
          .route("/report", post().to(create_comment_report_v3))
          .route("/remove", post().to(remove_comment_v3))
          .route("/distinguish", post().to(distinguish_comment_v3)),
      )
      .service(
        scope("/custom_emoji")
          .wrap(rate_limit.message())
          .route("", post().to(create_custom_emoji_v3))
          .route("", put().to(edit_custom_emoji_v3))
          .route("/delete", post().to(delete_custom_emoji_v3)),
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
          .route("", get().to(read_person_v3))
          .route("/logout", post().to(logout_v3))
          .route("/unread_count", get().to(unread_count_v3))
          .route("/block", post().to(block_person_v3))
          .route(
            "/mark_all_as_read",
            post().to(mark_all_notifications_read_v3),
          )
          .route("/validate_auth", get().to(validate_auth))
          .route(
            "/donation_dialog_shown",
            post().to(mark_donation_dialog_shown),
          ),
      ),
  );

  // A few additional image redirects.
  // See https://github.com/LemmyNet/lemmy/issues/6734
  cfg
    .service(redirect("/pictrs/image", "/api/v4/image"))
    .service(redirect("/pictrs/healthz", "/api/v4/health"))
    .service(redirect("/api/v3/image_proxy", "/api/v4/proxy"));
}
