use actix_web::{guard, web};
use lemmy_api::{
  comment::{
    distinguish::distinguish_comment,
    like::like_comment,
    list_comment_likes::list_comment_likes,
    save::save_comment,
  },
  comment_report::{
    create::create_comment_report,
    list::list_comment_reports,
    resolve::resolve_comment_report,
  },
  community::{
    add_mod::add_mod_to_community,
    ban::ban_from_community,
    block::block_community,
    follow::follow_community,
    hide::hide_community,
    pending_follows::{
      approve::post_pending_follows_approve,
      count::get_pending_follows_count,
      list::get_pending_follows_list,
    },
    random::get_random_community,
    transfer::transfer_community,
  },
  local_user::{
    add_admin::add_admin,
    ban_person::ban_from_site,
    block::block_person,
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
      list_mentions::list_mentions,
      list_replies::list_replies,
      mark_all_read::mark_all_notifications_read,
      mark_mention_read::mark_person_mention_as_read,
      mark_reply_read::mark_reply_as_read,
      unread_count::unread_count,
    },
    report_count::report_count,
    reset_password::reset_password,
    save_settings::save_user_settings,
    update_totp::update_totp,
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
    mark_many_read::mark_posts_as_read,
    mark_read::mark_post_as_read,
    save::save_post,
  },
  post_report::{
    create::create_post_report,
    list::list_post_reports,
    resolve::resolve_post_report,
  },
  private_message::mark_read::mark_pm_as_read,
  private_message_report::{
    create::create_pm_report,
    list::list_pm_reports,
    resolve::resolve_pm_report,
  },
  site::{
    block::block_instance,
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
  sitemap::get_sitemap,
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
    list::list_custom_emojis,
    update::update_custom_emoji,
  },
  oauth_provider::{
    create::create_oauth_provider,
    delete::delete_oauth_provider,
    update::update_oauth_provider,
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
    read::get_private_message,
    update::update_private_message,
  },
  site::{create::create_site, read::get_site, update::update_site},
  tagline::{
    create::create_tagline,
    delete::delete_tagline,
    list::list_taglines,
    update::update_tagline,
  },
  user::{
    create::{authenticate_with_oauth, register},
    delete::delete_account,
    my_user::get_my_user,
  },
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
use lemmy_routes::images::image_proxy;
use lemmy_utils::rate_limit::RateLimitCell;

pub fn config(cfg: &mut web::ServiceConfig, rate_limit: &RateLimitCell) {
  cfg.service(
    web::scope("/api/v4")
      .wrap(rate_limit.message())
      .route("/image_proxy", web::get().to(image_proxy))
      // Site
      .service(
        web::scope("/site")
          .route("", web::get().to(get_site))
          // Admin Actions
          .route("", web::post().to(create_site))
          .route("", web::put().to(update_site))
          .route("/block", web::post().to(block_instance)),
      )
      .route("/modlog", web::get().to(get_mod_log))
      .service(
        web::resource("/search")
          .wrap(rate_limit.search())
          .route(web::get().to(search)),
      )
      .route("/resolve_object", web::get().to(resolve_object))
      // Community
      .service(
        web::resource("/community")
          .guard(guard::Post())
          .wrap(rate_limit.register())
          .route(web::post().to(create_community)),
      )
      .service(
        web::scope("/community")
          .route("", web::get().to(get_community))
          .route("", web::put().to(update_community))
          .route("/random", web::get().to(get_random_community))
          .route("/hide", web::put().to(hide_community))
          .route("/list", web::get().to(list_communities))
          .route("/follow", web::post().to(follow_community))
          .route("/block", web::post().to(block_community))
          .route("/delete", web::post().to(delete_community))
          // Mod Actions
          .route("/remove", web::post().to(remove_community))
          .route("/transfer", web::post().to(transfer_community))
          .route("/ban_user", web::post().to(ban_from_community))
          .route("/mod", web::post().to(add_mod_to_community))
          .service(
            web::scope("/pending_follows")
              .route("/count", web::get().to(get_pending_follows_count))
              .route("/list", web::get().to(get_pending_follows_list))
              .route("/approve", web::post().to(post_pending_follows_approve)),
          ),
      )
      .route(
        "/federated_instances",
        web::get().to(get_federated_instances),
      )
      // Post
      .service(
        // Handle POST to /post separately to add the post() rate limitter
        web::resource("/post")
          .guard(guard::Post())
          .wrap(rate_limit.post())
          .route(web::post().to(create_post)),
      )
      .service(
        web::scope("/post")
          .route("", web::get().to(get_post))
          .route("", web::put().to(update_post))
          .route("/delete", web::post().to(delete_post))
          .route("/remove", web::post().to(remove_post))
          .route("/mark_as_read", web::post().to(mark_post_as_read))
          .route("/mark_many_as_read", web::post().to(mark_posts_as_read))
          .route("/hide", web::post().to(hide_post))
          .route("/lock", web::post().to(lock_post))
          .route("/feature", web::post().to(feature_post))
          .route("/list", web::get().to(list_posts))
          .route("/like", web::post().to(like_post))
          .route("/like/list", web::get().to(list_post_likes))
          .route("/save", web::put().to(save_post))
          .route("/report", web::post().to(create_post_report))
          .route("/report/resolve", web::put().to(resolve_post_report))
          .route("/report/list", web::get().to(list_post_reports))
          .route("/site_metadata", web::get().to(get_link_metadata)),
      )
      // Comment
      .service(
        // Handle POST to /comment separately to add the comment() rate limitter
        web::resource("/comment")
          .guard(guard::Post())
          .wrap(rate_limit.comment())
          .route(web::post().to(create_comment)),
      )
      .service(
        web::scope("/comment")
          .route("", web::get().to(get_comment))
          .route("", web::put().to(update_comment))
          .route("/delete", web::post().to(delete_comment))
          .route("/remove", web::post().to(remove_comment))
          .route("/mark_as_read", web::post().to(mark_reply_as_read))
          .route("/distinguish", web::post().to(distinguish_comment))
          .route("/like", web::post().to(like_comment))
          .route("/like/list", web::get().to(list_comment_likes))
          .route("/save", web::put().to(save_comment))
          .route("/list", web::get().to(list_comments))
          .route("/report", web::post().to(create_comment_report))
          .route("/report/resolve", web::put().to(resolve_comment_report))
          .route("/report/list", web::get().to(list_comment_reports)),
      )
      // Private Message
      .service(
        web::scope("/private_message")
          .route("/list", web::get().to(get_private_message))
          .route("", web::post().to(create_private_message))
          .route("", web::put().to(update_private_message))
          .route("/delete", web::post().to(delete_private_message))
          .route("/mark_as_read", web::post().to(mark_pm_as_read))
          .route("/report", web::post().to(create_pm_report))
          .route("/report/resolve", web::put().to(resolve_pm_report))
          .route("/report/list", web::get().to(list_pm_reports)),
      )
      // User
      .service(
        web::scope("/account/auth")
          .guard(guard::Post())
          .wrap(rate_limit.register())
          .route("register", web::post().to(register))
          .route("login", web::post().to(login))
          .route("/logout", web::post().to(logout))
          .route("password_reset", web::post().to(reset_password))
          .route("get_captcha", web::get().to(get_captcha))
          .route(
            "/password_change",
            web::post().to(change_password_after_reset),
          )
          .route("/change_password", web::put().to(change_password))
          .route("/totp/generate", web::post().to(generate_totp_secret))
          .route("/totp/update", web::post().to(update_totp))
          .route("/verify_email", web::post().to(verify_email)),
      )
      .service(
        web::scope("/account/settings")
          .wrap(rate_limit.import_user_settings())
          .route("/export", web::get().to(export_settings))
          .route("/import", web::post().to(import_settings)),
      )
      .service(
        web::scope("/account")
          .route("/my_user", web::get().to(get_my_user))
          .route("/list_media", web::get().to(list_media))
          .route("/mention", web::get().to(list_mentions))
          .route("/replies", web::get().to(list_replies))
          .route("/block", web::post().to(block_person))
          .route("/delete", web::post().to(delete_account))
          .route(
            "/mention/mark_as_read",
            web::post().to(mark_person_mention_as_read),
          )
          .route(
            "/mention/mark_all_as_read",
            web::post().to(mark_all_notifications_read),
          )
          .route("/settings/save", web::put().to(save_user_settings))
          .route("/report_count", web::get().to(report_count))
          .route("/unread_count", web::get().to(unread_count))
          .route("/list_logins", web::get().to(list_logins))
          .route("/validate_auth", web::get().to(validate_auth)),
      )
      // User actions
      .route("/person", web::get().to(read_person))
      // Admin Actions
      .service(
        web::scope("/admin")
          .route("/add", web::post().to(add_admin))
          .route(
            "/registration_application/count",
            web::get().to(get_unread_registration_application_count),
          )
          .route(
            "/registration_application/list",
            web::get().to(list_registration_applications),
          )
          .route(
            "/registration_application/approve",
            web::put().to(approve_registration_application),
          )
          .route(
            "/registration_application",
            web::get().to(get_registration_application),
          )
          .route("/list_all_media", web::get().to(list_all_media))
          .service(
            web::scope("/purge")
              .route("/person", web::post().to(purge_person))
              .route("/community", web::post().to(purge_community))
              .route("/post", web::post().to(purge_post))
              .route("/comment", web::post().to(purge_comment)),
          )
          .service(
            web::scope("/tagline")
              .route("", web::post().to(create_tagline))
              .route("", web::put().to(update_tagline))
              .route("/delete", web::post().to(delete_tagline))
              .route("/list", web::get().to(list_taglines)),
          )
          .route("/ban", web::post().to(ban_from_site))
          .route("/banned", web::get().to(list_banned_users))
          .route("/leave", web::post().to(leave_admin)),
      )
      .service(
        web::scope("/custom_emoji")
          .route("", web::post().to(create_custom_emoji))
          .route("", web::put().to(update_custom_emoji))
          .route("/delete", web::post().to(delete_custom_emoji))
          .route("/list", web::get().to(list_custom_emojis)),
      )
      .service(
        web::scope("/oauth_provider")
          .route("", web::post().to(create_oauth_provider))
          .route("", web::put().to(update_oauth_provider))
          .route("/delete", web::post().to(delete_oauth_provider)),
      )
      .service(
        web::scope("/oauth")
          .wrap(rate_limit.register())
          .route("/authenticate", web::post().to(authenticate_with_oauth)),
      )
      .route("/sitemap.xml", web::get().to(get_sitemap)),
  );
}
