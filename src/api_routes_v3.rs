use actix_web::{guard, web::*};
use lemmy_api::{
  comment::{
    distinguish::distinguish_comment,
    like::like_comment,
    list_comment_likes::list_comment_likes,
    save::save_comment,
  },
  community::{
    add_mod::add_mod_to_community,
    ban::ban_from_community,
    block::user_block_community,
    follow::follow_community,
    hide::hide_community,
    transfer::transfer_community,
  },
  local_user::{
    add_admin::add_admin,
    ban_person::ban_from_site,
    block::user_block_person,
    change_password::change_password,
    change_password_after_reset::change_password_after_reset,
    generate_totp_secret::generate_totp_secret,
    get_captcha::get_captcha,
    list_banned::list_banned_users,
    list_logins::list_logins,
    list_media::list_media,
    login::login,
    logout::logout,
    notifications::{
      mark_all_read::mark_all_notifications_read,
      mark_comment_mention_read::mark_comment_mention_as_read,
      mark_reply_read::mark_reply_as_read,
      unread_count::unread_count,
    },
    report_count::report_count,
    reset_password::reset_password,
    save_settings::save_user_settings,
    update_totp::update_totp,
    user_block_instance::user_block_instance,
    validate_auth::validate_auth,
    verify_email::verify_email,
  },
  post::{
    feature::feature_post,
    get_link_metadata::get_link_metadata,
    hide::hide_post,
    like::like_post,
    list_post_likes::list_post_likes,
    lock::lock_post,
    mark_read::mark_post_as_read,
    save::save_post,
  },
  private_message::mark_read::mark_pm_as_read,
  reports::{
    comment_report::{create::create_comment_report, resolve::resolve_comment_report},
    post_report::{create::create_post_report, resolve::resolve_post_report},
    private_message_report::{create::create_pm_report, resolve::resolve_pm_report},
  },
  site::{
    federated_instances::get_federated_instances,
    leave_admin::leave_admin,
    list_all_media::list_all_media,
    mod_log::get_mod_log,
    purge::{
      comment::purge_comment,
      community::purge_community,
      person::purge_person,
      post::purge_post,
    },
    registration_applications::{
      approve::approve_registration_application,
      get::get_registration_application,
      list::list_registration_applications,
      unread_count::get_unread_registration_application_count,
    },
  },
};
use lemmy_api_crud::{
  comment::{
    create::create_comment,
    delete::delete_comment,
    read::get_comment,
    remove::remove_comment,
    update::update_comment,
  },
  community::{
    create::create_community,
    delete::delete_community,
    list::list_communities,
    remove::remove_community,
    update::update_community,
  },
  custom_emoji::{
    create::create_custom_emoji,
    delete::delete_custom_emoji,
    update::update_custom_emoji,
  },
  post::{
    create::create_post,
    delete::delete_post,
    read::get_post,
    remove::remove_post,
    update::update_post,
  },
  private_message::{
    create::create_private_message,
    delete::delete_private_message,
    update::update_private_message,
  },
  site::{create::create_site, read::get_site_v3, update::update_site},
  user::{create::register, delete::delete_account},
};
use lemmy_apub::api::{
  list_comments::list_comments,
  list_posts::list_posts,
  read_community::get_community,
  read_person::read_person,
  resolve_object::resolve_object,
  search::search,
  user_settings_backup::{export_settings, import_settings},
};
use lemmy_routes::images::{
  delete::delete_image,
  download::{get_image, image_proxy},
  pictrs_health,
  upload::upload_image,
};
use lemmy_utils::rate_limit::RateLimitCell;

// Deprecated, use api v4 instead.
// When removing api v3, make sure to keep `/api/v3/image_proxy` as it is still used in old posts.
pub fn config(cfg: &mut ServiceConfig, rate_limit: &RateLimitCell) {
  cfg
    .service(
      resource("/pictrs/image")
        .wrap(rate_limit.image())
        .route(post().to(upload_image)),
    )
    .service(
      scope("/pictrs")
        .wrap(rate_limit.message())
        .route("/image/{filename}", get().to(get_image))
        .route("/image/delete/{token}/{filename}", get().to(delete_image))
        .route("/healthz", get().to(pictrs_health)),
    )
    .service(
      scope("/api/v3")
        .route("/image_proxy", get().to(image_proxy))
        // Site
        .service(
          scope("/site")
            .wrap(rate_limit.message())
            .route("", get().to(get_site_v3))
            // Admin Actions
            .route("", post().to(create_site))
            .route("", put().to(update_site))
            .route("/block", post().to(user_block_instance)),
        )
        .service(
          resource("/modlog")
            .wrap(rate_limit.message())
            .route(get().to(get_mod_log)),
        )
        .service(
          resource("/search")
            .wrap(rate_limit.search())
            .route(get().to(search)),
        )
        .service(
          resource("/resolve_object")
            .wrap(rate_limit.search())
            .route(get().to(resolve_object)),
        )
        // Community
        .service(
          resource("/community")
            .guard(guard::Post())
            .wrap(rate_limit.register())
            .route(post().to(create_community)),
        )
        .service(
          scope("/community")
            .wrap(rate_limit.message())
            .route("", get().to(get_community))
            .route("", put().to(update_community))
            .route("/hide", put().to(hide_community))
            .route("/list", get().to(list_communities))
            .route("/follow", post().to(follow_community))
            .route("/block", post().to(user_block_community))
            .route("/delete", post().to(delete_community))
            // Mod Actions
            .route("/remove", post().to(remove_community))
            .route("/transfer", post().to(transfer_community))
            .route("/ban_user", post().to(ban_from_community))
            .route("/mod", post().to(add_mod_to_community)),
        )
        .service(
          scope("/federated_instances")
            .wrap(rate_limit.message())
            .route("", get().to(get_federated_instances)),
        )
        // Post
        .service(
          resource("/post")
            // Handle POST to /post separately to add the post() rate limitter
            .guard(guard::Post())
            .wrap(rate_limit.post())
            .route(post().to(create_post)),
        )
        .service(
          resource("/post/site_metadata")
            .wrap(rate_limit.search())
            .route(get().to(get_link_metadata)),
        )
        .service(
          scope("/post")
            .wrap(rate_limit.message())
            .route("", get().to(get_post))
            .route("", put().to(update_post))
            .route("/delete", post().to(delete_post))
            .route("/remove", post().to(remove_post))
            .route("/mark_as_read", post().to(mark_post_as_read))
            .route("/hide", post().to(hide_post))
            .route("/lock", post().to(lock_post))
            .route("/feature", post().to(feature_post))
            .route("/list", get().to(list_posts))
            .route("/like", post().to(like_post))
            .route("/like/list", get().to(list_post_likes))
            .route("/save", put().to(save_post))
            .route("/report", post().to(create_post_report))
            .route("/report/resolve", put().to(resolve_post_report)),
        )
        // Comment
        .service(
          // Handle POST to /comment separately to add the comment() rate limitter
          resource("/comment")
            .guard(guard::Post())
            .wrap(rate_limit.comment())
            .route(post().to(create_comment)),
        )
        .service(
          scope("/comment")
            .wrap(rate_limit.message())
            .route("", get().to(get_comment))
            .route("", put().to(update_comment))
            .route("/delete", post().to(delete_comment))
            .route("/remove", post().to(remove_comment))
            .route("/mark_as_read", post().to(mark_reply_as_read))
            .route("/distinguish", post().to(distinguish_comment))
            .route("/like", post().to(like_comment))
            .route("/like/list", get().to(list_comment_likes))
            .route("/save", put().to(save_comment))
            .route("/list", get().to(list_comments))
            .route("/report", post().to(create_comment_report))
            .route("/report/resolve", put().to(resolve_comment_report)),
        )
        // Private Message
        .service(
          scope("/private_message")
            .wrap(rate_limit.message())
            .route("", post().to(create_private_message))
            .route("", put().to(update_private_message))
            .route("/delete", post().to(delete_private_message))
            .route("/mark_as_read", post().to(mark_pm_as_read))
            .route("/report", post().to(create_pm_report))
            .route("/report/resolve", put().to(resolve_pm_report)),
        )
        // User
        .service(
          // Account action, I don't like that it's in /user maybe /accounts
          // Handle /user/register separately to add the register() rate limiter
          resource("/user/register")
            .guard(guard::Post())
            .wrap(rate_limit.register())
            .route(post().to(register)),
        )
        // User
        .service(
          // Handle /user/login separately to add the register() rate limiter
          // TODO: pretty annoying way to apply rate limits for register and login, we should
          //       group them under a common path so that rate limit is only applied once (eg under
          // /account).
          resource("/user/login")
            .guard(guard::Post())
            .wrap(rate_limit.register())
            .route(post().to(login)),
        )
        .service(
          resource("/user/password_reset")
            .wrap(rate_limit.register())
            .route(post().to(reset_password)),
        )
        .service(
          // Handle captcha separately
          resource("/user/get_captcha")
            .wrap(rate_limit.post())
            .route(get().to(get_captcha)),
        )
        .service(
          resource("/user/export_settings")
            .wrap(rate_limit.import_user_settings())
            .route(get().to(export_settings)),
        )
        .service(
          resource("/user/import_settings")
            .wrap(rate_limit.import_user_settings())
            .route(post().to(import_settings)),
        )
        // TODO, all the current account related actions under /user need to get moved here
        // eventually
        .service(
          scope("/account")
            .wrap(rate_limit.message())
            .route("/list_media", get().to(list_media)),
        )
        // User actions
        .service(
          scope("/user")
            .wrap(rate_limit.message())
            .route("", get().to(read_person))
            .route(
              "/mention/mark_as_read",
              post().to(mark_comment_mention_as_read),
            )
            // Admin action. I don't like that it's in /user
            .route("/ban", post().to(ban_from_site))
            .route("/banned", get().to(list_banned_users))
            .route("/block", post().to(user_block_person))
            // TODO Account actions. I don't like that they're in /user maybe /accounts
            .route("/logout", post().to(logout))
            .route("/delete_account", post().to(delete_account))
            .route("/password_change", post().to(change_password_after_reset))
            // TODO mark_all_as_read feels off being in this section as well
            .route("/mark_all_as_read", post().to(mark_all_notifications_read))
            .route("/save_user_settings", put().to(save_user_settings))
            .route("/change_password", put().to(change_password))
            .route("/report_count", get().to(report_count))
            .route("/unread_count", get().to(unread_count))
            .route("/verify_email", post().to(verify_email))
            .route("/leave_admin", post().to(leave_admin))
            .route("/totp/generate", post().to(generate_totp_secret))
            .route("/totp/update", post().to(update_totp))
            .route("/list_logins", get().to(list_logins))
            .route("/validate_auth", get().to(validate_auth)),
        )
        // Admin Actions
        .service(
          scope("/admin")
            .wrap(rate_limit.message())
            .route("/add", post().to(add_admin))
            .route(
              "/registration_application/count",
              get().to(get_unread_registration_application_count),
            )
            .route(
              "/registration_application/list",
              get().to(list_registration_applications),
            )
            .route(
              "/registration_application/approve",
              put().to(approve_registration_application),
            )
            .route(
              "/registration_application",
              get().to(get_registration_application),
            )
            .route("/list_all_media", get().to(list_all_media))
            .service(
              scope("/purge")
                .route("/person", post().to(purge_person))
                .route("/community", post().to(purge_community))
                .route("/post", post().to(purge_post))
                .route("/comment", post().to(purge_comment)),
            ),
        )
        .service(
          scope("/custom_emoji")
            .wrap(rate_limit.message())
            .route("", post().to(create_custom_emoji))
            .route("", put().to(update_custom_emoji))
            .route("/delete", post().to(delete_custom_emoji)),
        ),
    );
}
