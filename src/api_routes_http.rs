use actix_web::{guard, web, Error, HttpResponse, Result};
use lemmy_api::{
  comment::{distinguish::distinguish_comment, save::save_comment},
  comment_report::{list::list_comment_reports, resolve::resolve_comment_report},
  local_user::notifications::mark_reply_read::mark_reply_as_read,
  Perform,
};
use lemmy_api_common::{
  comment::{CreateCommentLike, CreateCommentReport, DeleteComment, EditComment, RemoveComment},
  community::{
    AddModToCommunity,
    BanFromCommunity,
    BlockCommunity,
    CreateCommunity,
    DeleteCommunity,
    EditCommunity,
    FollowCommunity,
    HideCommunity,
    RemoveCommunity,
    TransferCommunity,
  },
  context::LemmyContext,
  custom_emoji::{CreateCustomEmoji, DeleteCustomEmoji, EditCustomEmoji},
  person::{
    AddAdmin,
    BanPerson,
    BlockPerson,
    ChangePassword,
    DeleteAccount,
    GetBannedPersons,
    GetCaptcha,
    GetPersonMentions,
    GetReplies,
    GetReportCount,
    GetUnreadCount,
    Login,
    MarkAllAsRead,
    MarkPersonMentionAsRead,
    PasswordChangeAfterReset,
    PasswordReset,
    Register,
    SaveUserSettings,
    VerifyEmail,
  },
  post::{
    CreatePostLike,
    CreatePostReport,
    DeletePost,
    EditPost,
    FeaturePost,
    GetSiteMetadata,
    ListPostReports,
    LockPost,
    MarkPostAsRead,
    RemovePost,
    ResolvePostReport,
    SavePost,
  },
  private_message::{
    CreatePrivateMessage,
    CreatePrivateMessageReport,
    DeletePrivateMessage,
    EditPrivateMessage,
    ListPrivateMessageReports,
    MarkPrivateMessageAsRead,
    ResolvePrivateMessageReport,
  },
  site::{
    ApproveRegistrationApplication,
    GetFederatedInstances,
    GetModlog,
    GetUnreadRegistrationApplicationCount,
    LeaveAdmin,
    ListRegistrationApplications,
    PurgeComment,
    PurgeCommunity,
    PurgePerson,
    PurgePost,
  },
};
use lemmy_api_crud::{
  comment::{create::create_comment, read::get_comment},
  community::list::list_communities,
  post::{create::create_post, read::get_post},
  private_message::read::get_private_message,
  site::{create::create_site, read::get_site, update::update_site},
  PerformCrud,
};
use lemmy_apub::{
  api::{
    list_comments::list_comments,
    list_posts::list_posts,
    read_community::get_community,
    read_person::read_person,
    resolve_object::resolve_object,
    search::search,
  },
  SendActivity,
};
use lemmy_utils::{rate_limit::RateLimitCell, spawn_try_task, SYNCHRONOUS_FEDERATION};
use serde::Deserialize;

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
          .route("", web::put().to(update_site)),
      )
      .service(
        web::resource("/modlog")
          .wrap(rate_limit.message())
          .route(web::get().to(route_get::<GetModlog>)),
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
          .route(web::post().to(route_post_crud::<CreateCommunity>)),
      )
      .service(
        web::scope("/community")
          .wrap(rate_limit.message())
          .route("", web::get().to(get_community))
          .route("", web::put().to(route_post_crud::<EditCommunity>))
          .route("/hide", web::put().to(route_post::<HideCommunity>))
          .route("/list", web::get().to(list_communities))
          .route("/follow", web::post().to(route_post::<FollowCommunity>))
          .route("/block", web::post().to(route_post::<BlockCommunity>))
          .route(
            "/delete",
            web::post().to(route_post_crud::<DeleteCommunity>),
          )
          // Mod Actions
          .route(
            "/remove",
            web::post().to(route_post_crud::<RemoveCommunity>),
          )
          .route("/transfer", web::post().to(route_post::<TransferCommunity>))
          .route("/ban_user", web::post().to(route_post::<BanFromCommunity>))
          .route("/mod", web::post().to(route_post::<AddModToCommunity>)),
      )
      .service(
        web::scope("/federated_instances")
          .wrap(rate_limit.message())
          .route("", web::get().to(route_get::<GetFederatedInstances>)),
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
          .route("", web::put().to(route_post_crud::<EditPost>))
          .route("/delete", web::post().to(route_post_crud::<DeletePost>))
          .route("/remove", web::post().to(route_post_crud::<RemovePost>))
          .route(
            "/mark_as_read",
            web::post().to(route_post::<MarkPostAsRead>),
          )
          .route("/lock", web::post().to(route_post::<LockPost>))
          .route("/feature", web::post().to(route_post::<FeaturePost>))
          .route("/list", web::get().to(list_posts))
          .route("/like", web::post().to(route_post::<CreatePostLike>))
          .route("/save", web::put().to(route_post::<SavePost>))
          .route("/report", web::post().to(route_post::<CreatePostReport>))
          .route(
            "/report/resolve",
            web::put().to(route_post::<ResolvePostReport>),
          )
          .route("/report/list", web::get().to(route_get::<ListPostReports>))
          .route(
            "/site_metadata",
            web::get().to(route_get::<GetSiteMetadata>),
          ),
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
          .route("", web::put().to(route_post_crud::<EditComment>))
          .route("/delete", web::post().to(route_post_crud::<DeleteComment>))
          .route("/remove", web::post().to(route_post_crud::<RemoveComment>))
          .route("/mark_as_read", web::post().to(mark_reply_as_read))
          .route("/distinguish", web::post().to(distinguish_comment))
          .route("/like", web::post().to(route_post::<CreateCommentLike>))
          .route("/save", web::put().to(save_comment))
          .route("/list", web::get().to(list_comments))
          .route("/report", web::post().to(route_post::<CreateCommentReport>))
          .route("/report/resolve", web::put().to(resolve_comment_report))
          .route("/report/list", web::get().to(list_comment_reports)),
      )
      // Private Message
      .service(
        web::scope("/private_message")
          .wrap(rate_limit.message())
          .route("/list", web::get().to(get_private_message))
          .route("", web::post().to(route_post_crud::<CreatePrivateMessage>))
          .route("", web::put().to(route_post_crud::<EditPrivateMessage>))
          .route(
            "/delete",
            web::post().to(route_post_crud::<DeletePrivateMessage>),
          )
          .route(
            "/mark_as_read",
            web::post().to(route_post::<MarkPrivateMessageAsRead>),
          )
          .route(
            "/report",
            web::post().to(route_post::<CreatePrivateMessageReport>),
          )
          .route(
            "/report/resolve",
            web::put().to(route_post::<ResolvePrivateMessageReport>),
          )
          .route(
            "/report/list",
            web::get().to(route_get::<ListPrivateMessageReports>),
          ),
      )
      // User
      .service(
        // Account action, I don't like that it's in /user maybe /accounts
        // Handle /user/register separately to add the register() rate limitter
        web::resource("/user/register")
          .guard(guard::Post())
          .wrap(rate_limit.register())
          .route(web::post().to(route_post_crud::<Register>)),
      )
      .service(
        // Handle captcha separately
        web::resource("/user/get_captcha")
          .wrap(rate_limit.post())
          .route(web::get().to(route_get::<GetCaptcha>)),
      )
      // User actions
      .service(
        web::scope("/user")
          .wrap(rate_limit.message())
          .route("", web::get().to(read_person))
          .route("/mention", web::get().to(route_get::<GetPersonMentions>))
          .route(
            "/mention/mark_as_read",
            web::post().to(route_post::<MarkPersonMentionAsRead>),
          )
          .route("/replies", web::get().to(route_get::<GetReplies>))
          // Admin action. I don't like that it's in /user
          .route("/ban", web::post().to(route_post::<BanPerson>))
          .route("/banned", web::get().to(route_get::<GetBannedPersons>))
          .route("/block", web::post().to(route_post::<BlockPerson>))
          // Account actions. I don't like that they're in /user maybe /accounts
          .route("/login", web::post().to(route_post::<Login>))
          .route(
            "/delete_account",
            web::post().to(route_post_crud::<DeleteAccount>),
          )
          .route(
            "/password_reset",
            web::post().to(route_post::<PasswordReset>),
          )
          .route(
            "/password_change",
            web::post().to(route_post::<PasswordChangeAfterReset>),
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
          .route(
            "/change_password",
            web::put().to(route_post::<ChangePassword>),
          )
          .route("/report_count", web::get().to(route_get::<GetReportCount>))
          .route("/unread_count", web::get().to(route_get::<GetUnreadCount>))
          .route("/verify_email", web::post().to(route_post::<VerifyEmail>))
          .route("/leave_admin", web::post().to(route_post::<LeaveAdmin>)),
      )
      // Admin Actions
      .service(
        web::scope("/admin")
          .wrap(rate_limit.message())
          .route("/add", web::post().to(route_post::<AddAdmin>))
          .route(
            "/registration_application/count",
            web::get().to(route_get::<GetUnreadRegistrationApplicationCount>),
          )
          .route(
            "/registration_application/list",
            web::get().to(route_get::<ListRegistrationApplications>),
          )
          .route(
            "/registration_application/approve",
            web::put().to(route_post::<ApproveRegistrationApplication>),
          )
          .service(
            web::scope("/purge")
              .route("/person", web::post().to(route_post::<PurgePerson>))
              .route("/community", web::post().to(route_post::<PurgeCommunity>))
              .route("/post", web::post().to(route_post::<PurgePost>))
              .route("/comment", web::post().to(route_post::<PurgeComment>)),
          ),
      )
      .service(
        web::scope("/custom_emoji")
          .wrap(rate_limit.message())
          .route("", web::post().to(route_post_crud::<CreateCustomEmoji>))
          .route("", web::put().to(route_post_crud::<EditCustomEmoji>))
          .route(
            "/delete",
            web::post().to(route_post_crud::<DeleteCustomEmoji>),
          ),
      ),
  );
}

async fn perform<'a, Data>(
  data: Data,
  context: web::Data<LemmyContext>,
  apub_data: activitypub_federation::config::Data<LemmyContext>,
) -> Result<HttpResponse, Error>
where
  Data: Perform
    + SendActivity<Response = <Data as Perform>::Response>
    + Clone
    + Deserialize<'a>
    + Send
    + 'static,
{
  let res = data.perform(&context).await?;
  let res_clone = res.clone();
  let fed_task = async move { SendActivity::send_activity(&data, &res_clone, &apub_data).await };
  if *SYNCHRONOUS_FEDERATION {
    fed_task.await?;
  } else {
    spawn_try_task(fed_task);
  }
  Ok(HttpResponse::Ok().json(&res))
}

async fn route_get<'a, Data>(
  data: web::Query<Data>,
  context: web::Data<LemmyContext>,
  apub_data: activitypub_federation::config::Data<LemmyContext>,
) -> Result<HttpResponse, Error>
where
  Data: Perform
    + SendActivity<Response = <Data as Perform>::Response>
    + Clone
    + Deserialize<'a>
    + Send
    + 'static,
{
  perform::<Data>(data.0, context, apub_data).await
}

async fn route_post<'a, Data>(
  data: web::Json<Data>,
  context: web::Data<LemmyContext>,
  apub_data: activitypub_federation::config::Data<LemmyContext>,
) -> Result<HttpResponse, Error>
where
  Data: Perform
    + SendActivity<Response = <Data as Perform>::Response>
    + Clone
    + Deserialize<'a>
    + Send
    + 'static,
{
  perform::<Data>(data.0, context, apub_data).await
}

async fn perform_crud<'a, Data>(
  data: Data,
  context: web::Data<LemmyContext>,
  apub_data: activitypub_federation::config::Data<LemmyContext>,
) -> Result<HttpResponse, Error>
where
  Data: PerformCrud
    + SendActivity<Response = <Data as PerformCrud>::Response>
    + Clone
    + Deserialize<'a>
    + Send
    + 'static,
{
  let res = data.perform(&context).await?;
  let res_clone = res.clone();
  let fed_task = async move { SendActivity::send_activity(&data, &res_clone, &apub_data).await };
  if *SYNCHRONOUS_FEDERATION {
    fed_task.await?;
  } else {
    spawn_try_task(fed_task);
  }
  Ok(HttpResponse::Ok().json(&res))
}

async fn route_post_crud<'a, Data>(
  data: web::Json<Data>,
  context: web::Data<LemmyContext>,
  apub_data: activitypub_federation::config::Data<LemmyContext>,
) -> Result<HttpResponse, Error>
where
  Data: PerformCrud
    + SendActivity<Response = <Data as PerformCrud>::Response>
    + Clone
    + Deserialize<'a>
    + Send
    + 'static,
{
  perform_crud::<Data>(data.0, context, apub_data).await
}
