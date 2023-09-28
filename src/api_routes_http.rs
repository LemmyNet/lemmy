use actix_web::{guard, web};
use lemmy_api::{
  comment::{distinguish::distinguish_comment, like::like_comment, save::save_comment},
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
    transfer::transfer_community,
  },
  local_user::{
    add_admin::add_admin,
    ban_person::ban_from_site,
    block::block_person,
    change_password::change_password,
    change_password_after_reset::change_password_after_reset,
    get_captcha::get_captcha,
    list_banned::list_banned_users,
    login::login,
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
    verify_email::verify_email,
  },
  post::{
    feature::feature_post,
    get_link_metadata::get_link_metadata,
    like::like_post,
    lock::lock_post,
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
    mod_log::get_mod_log,
    purge::{
      comment::purge_comment,
      community::purge_community,
      person::purge_person,
      post::purge_post,
    },
    registration_applications::{
      approve::approve_registration_application,
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
    read::get_private_message,
    update::update_private_message,
  },
  site::{create::create_site, read::get_site, update::update_site},
  user::{create::register, delete::delete_account},
};
use lemmy_apub::api::{
  list_comments::list_comments,
  list_posts::list_posts,
  read_community::get_community,
  read_person::read_person,
  resolve_object::resolve_object,
  search::search,
  user_settings_backup::{export_user_backup, import_user_backup},
};
use lemmy_utils::rate_limit::RateLimitCell;

pub fn config(cfg: &mut web::ServiceConfig, rate_limit: &RateLimitCell) {
  cfg.service(
    web::scope("/api/v3")
      // Site
      .service(
        web::scope("/site")
          .wrap(rate_limit.message())
          .route("", web::get().to(get_site))
          // Admin Actions
          .route("", web::post().to(create_site))
          .route("", web::put().to(update_site))
          .route("/block", web::post().to(block_instance)),
      )
      .service(
        web::resource("/modlog")
          .wrap(rate_limit.message())
          .route(web::get().to(get_mod_log)),
      )
      .service(
        web::resource("/search")
          .wrap(rate_limit.search())
          .route(web::get().to(search)),
      )
      .service(
        web::resource("/resolve_object")
          .wrap(rate_limit.message())
          .route(web::get().to(resolve_object)),
      )
      // Community
      .service(
        web::resource("/community")
          .guard(guard::Post())
          .wrap(rate_limit.register())
          .route(web::post().to(create_community)),
      )
      .service(
        web::scope("/community")
          .wrap(rate_limit.message())
          .route("", web::get().to(get_community))
          .route("", web::put().to(update_community))
          .route("/hide", web::put().to(hide_community))
          .route("/list", web::get().to(list_communities))
          .route("/follow", web::post().to(follow_community))
          .route("/block", web::post().to(block_community))
          .route("/delete", web::post().to(delete_community))
          // Mod Actions
          .route("/remove", web::post().to(remove_community))
          .route("/transfer", web::post().to(transfer_community))
          .route("/ban_user", web::post().to(ban_from_community))
          .route("/mod", web::post().to(add_mod_to_community)),
      )
      .service(
        web::scope("/federated_instances")
          .wrap(rate_limit.message())
          .route("", web::get().to(get_federated_instances)),
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
          .wrap(rate_limit.message())
          .route("", web::get().to(get_post))
          .route("", web::put().to(update_post))
          .route("/delete", web::post().to(delete_post))
          .route("/remove", web::post().to(remove_post))
          .route("/mark_as_read", web::post().to(mark_post_as_read))
          .route("/lock", web::post().to(lock_post))
          .route("/feature", web::post().to(feature_post))
          .route("/list", web::get().to(list_posts))
          .route("/like", web::post().to(like_post))
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
          .wrap(rate_limit.message())
          .route("", web::get().to(get_comment))
          .route("", web::put().to(update_comment))
          .route("/delete", web::post().to(delete_comment))
          .route("/remove", web::post().to(remove_comment))
          .route("/mark_as_read", web::post().to(mark_reply_as_read))
          .route("/distinguish", web::post().to(distinguish_comment))
          .route("/like", web::post().to(like_comment))
          .route("/save", web::put().to(save_comment))
          .route("/list", web::get().to(list_comments))
          .route("/report", web::post().to(create_comment_report))
          .route("/report/resolve", web::put().to(resolve_comment_report))
          .route("/report/list", web::get().to(list_comment_reports)),
      )
      // Private Message
      .service(
        web::scope("/private_message")
          .wrap(rate_limit.message())
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
        // Account action, I don't like that it's in /user maybe /accounts
        // Handle /user/register separately to add the register() rate limitter
        web::resource("/user/register")
          .guard(guard::Post())
          .wrap(rate_limit.register())
          .route(web::post().to(register)),
      )
      .service(
        // Handle captcha separately
        web::resource("/user/get_captcha")
          .wrap(rate_limit.post())
          .route(web::get().to(get_captcha)),
      )
      // User actions
      .service(
        web::scope("/user")
          .wrap(rate_limit.message())
          .route("", web::get().to(read_person))
          .route("/mention", web::get().to(list_mentions))
          .route(
            "/mention/mark_as_read",
            web::post().to(mark_person_mention_as_read),
          )
          .route("/replies", web::get().to(list_replies))
          // Admin action. I don't like that it's in /user
          .route("/ban", web::post().to(ban_from_site))
          .route("/banned", web::get().to(list_banned_users))
          .route("/block", web::post().to(block_person))
          // Account actions. I don't like that they're in /user maybe /accounts
          .route("/login", web::post().to(login))
          .route("/delete_account", web::post().to(delete_account))
          .route("/password_reset", web::post().to(reset_password))
          .route(
            "/password_change",
            web::post().to(change_password_after_reset),
          )
          // mark_all_as_read feels off being in this section as well
          .route(
            "/mark_all_as_read",
            web::post().to(mark_all_notifications_read),
          )
          .route("/save_user_settings", web::put().to(save_user_settings))
          .route("/change_password", web::put().to(change_password))
          .route("/report_count", web::get().to(report_count))
          .route("/unread_count", web::get().to(unread_count))
          .route("/verify_email", web::post().to(verify_email))
          .route("/export", web::get().to(export_user_backup)),
      )
      .service(
        // Handle captcha separately
        web::resource("/user/import")
          .wrap(rate_limit.post())
          .route(web::get().to(import_user_backup)),
      )
      // Admin Actions
      .service(
        web::scope("/admin")
          .wrap(rate_limit.message())
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
          .service(
            web::scope("/purge")
              .route("/person", web::post().to(purge_person))
              .route("/community", web::post().to(purge_community))
              .route("/post", web::post().to(purge_post))
              .route("/comment", web::post().to(purge_comment)),
          ),
      )
      .service(
        web::scope("/custom_emoji")
          .wrap(rate_limit.message())
          .route("", web::post().to(create_custom_emoji))
          .route("", web::put().to(update_custom_emoji))
          .route("/delete", web::post().to(delete_custom_emoji)),
      ),
  );
  cfg.service(
    web::scope("/sitemap.xml")
      .wrap(rate_limit.message())
      .route("", web::get().to(get_sitemap)),
  );
}
