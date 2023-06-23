use actix_web::{guard, web, Error, HttpResponse, Result};
use lemmy_api::Perform;
use lemmy_api_common::{
  comment::{
    CreateComment,
    CreateCommentLike,
    CreateCommentReport,
    DeleteComment,
    DistinguishComment,
    EditComment,
    GetComment,
    GetComments,
    ListCommentReports,
    RemoveComment,
    ResolveCommentReport,
    SaveComment,
  },
  community::{
    AddModToCommunity,
    BanFromCommunity,
    BlockCommunity,
    CreateCommunity,
    DeleteCommunity,
    EditCommunity,
    FollowCommunity,
    GetCommunity,
    HideCommunity,
    ListCommunities,
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
    GetPersonDetails,
    GetPersonMentions,
    GetReplies,
    GetReportCount,
    GetUnreadCount,
    Login,
    MarkAllAsRead,
    MarkCommentReplyAsRead,
    MarkPersonMentionAsRead,
    PasswordChangeAfterReset,
    PasswordReset,
    Register,
    SaveUserSettings,
    VerifyEmail,
  },
  post::{
    CreatePost,
    CreatePostLike,
    CreatePostReport,
    DeletePost,
    EditPost,
    FeaturePost,
    GetPost,
    GetPosts,
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
    GetPrivateMessages,
    ListPrivateMessageReports,
    MarkPrivateMessageAsRead,
    ResolvePrivateMessageReport,
  },
  site::{
    ApproveRegistrationApplication,
    CreateSite,
    EditSite,
    GetFederatedInstances,
    GetModlog,
    GetSite,
    GetUnreadRegistrationApplicationCount,
    LeaveAdmin,
    ListRegistrationApplications,
    PurgeComment,
    PurgeCommunity,
    PurgePerson,
    PurgePost,
    ResolveObject,
    Search,
  },
};
use lemmy_api_crud::PerformCrud;
use lemmy_apub::{api::PerformApub, SendActivity};
use lemmy_utils::rate_limit::RateLimitCell;
use serde::Deserialize;

pub fn config(cfg: &mut web::ServiceConfig, rate_limit: &RateLimitCell) {
  cfg.service(
    web::scope("/api/v3")
      // Site
      .service(
        web::scope("/site")
          .wrap(rate_limit.message())
          .route("", web::get().to(route_get_crud::<GetSite>))
          // Admin Actions
          .route("", web::post().to(route_post_crud::<CreateSite>))
          .route("", web::put().to(route_post_crud::<EditSite>)),
      )
      .service(
        web::resource("/modlog")
          .wrap(rate_limit.message())
          .route(web::get().to(route_get::<GetModlog>)),
      )
      .service(
        web::resource("/search")
          .wrap(rate_limit.search())
          .route(web::get().to(route_get_apub::<Search>)),
      )
      .service(
        web::resource("/resolve_object")
          .wrap(rate_limit.message())
          .route(web::get().to(route_get_apub::<ResolveObject>)),
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
          .route("", web::get().to(route_get_apub::<GetCommunity>))
          .route("", web::put().to(route_post_crud::<EditCommunity>))
          .route("/hide", web::put().to(route_post::<HideCommunity>))
          .route("/list", web::get().to(route_get_crud::<ListCommunities>))
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
          .route(web::post().to(route_post_crud::<CreatePost>)),
      )
      .service(
        web::scope("/post")
          .wrap(rate_limit.message())
          .route("", web::get().to(route_get_crud::<GetPost>))
          .route("", web::put().to(route_post_crud::<EditPost>))
          .route("/delete", web::post().to(route_post_crud::<DeletePost>))
          .route("/remove", web::post().to(route_post_crud::<RemovePost>))
          .route(
            "/mark_as_read",
            web::post().to(route_post::<MarkPostAsRead>),
          )
          .route("/lock", web::post().to(route_post::<LockPost>))
          .route("/feature", web::post().to(route_post::<FeaturePost>))
          .route("/list", web::get().to(route_get_apub::<GetPosts>))
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
          .route(web::post().to(route_post_crud::<CreateComment>)),
      )
      .service(
        web::scope("/comment")
          .wrap(rate_limit.message())
          .route("", web::get().to(route_get_crud::<GetComment>))
          .route("", web::put().to(route_post_crud::<EditComment>))
          .route("/delete", web::post().to(route_post_crud::<DeleteComment>))
          .route("/remove", web::post().to(route_post_crud::<RemoveComment>))
          .route(
            "/mark_as_read",
            web::post().to(route_post::<MarkCommentReplyAsRead>),
          )
          .route(
            "/distinguish",
            web::post().to(route_post::<DistinguishComment>),
          )
          .route("/like", web::post().to(route_post::<CreateCommentLike>))
          .route("/save", web::put().to(route_post::<SaveComment>))
          .route("/list", web::get().to(route_get_apub::<GetComments>))
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
          .route("/list", web::get().to(route_get_crud::<GetPrivateMessages>))
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
      // User actions
      .service(
        web::scope("/user")
          .wrap(rate_limit.message())
          .route("", web::get().to(route_get_apub::<GetPersonDetails>))
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
  SendActivity::send_activity(&data, &res, &apub_data).await?;
  Ok(HttpResponse::Ok().json(res))
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

async fn route_get_apub<'a, Data>(
  data: web::Query<Data>,
  context: activitypub_federation::config::Data<LemmyContext>,
) -> Result<HttpResponse, Error>
where
  Data: PerformApub
    + SendActivity<Response = <Data as PerformApub>::Response>
    + Clone
    + Deserialize<'a>
    + Send
    + 'static,
{
  let res = data.perform(&context).await?;
  SendActivity::send_activity(&data.0, &res, &context).await?;
  Ok(HttpResponse::Ok().json(res))
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
  SendActivity::send_activity(&data, &res, &apub_data).await?;
  Ok(HttpResponse::Ok().json(res))
}

async fn route_get_crud<'a, Data>(
  data: web::Query<Data>,
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
