use std::ops::Deref;
use actix_web::{guard, web, Error, HttpResponse, Result};
use activitypub_federation::config::Data as ContextData;
use serde_json::Value;
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
    GetCaptcha,
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
  websocket::{
    routes::chat_route,
    serialize_websocket_message,
    structs::{CommunityJoin, ModJoin, PostJoin, UserJoin},
    UserOperation,
    UserOperationApub,
    UserOperationCrud,
  },
};
use lemmy_api_crud::PerformCrud;
use lemmy_apub::{api::PerformApub, SendActivity};
use lemmy_utils::{error::LemmyError, rate_limit::RateLimitCell, ConnectionId};
use serde::Deserialize;
use std::result;

pub fn config(cfg: &mut web::ServiceConfig, rate_limit: &RateLimitCell) {
  cfg.service(
    web::scope("/api/v3")
      // Websocket
      .service(web::resource("/ws").to(chat_route))
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
          .route("/mod", web::post().to(route_post::<AddModToCommunity>))
          .route("/join", web::post().to(route_post::<CommunityJoin>))
          .route("/mod/join", web::post().to(route_post::<ModJoin>)),
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
          .route("/join", web::post().to(route_post::<PostJoin>))
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
          .route("", web::get().to(route_get_apub::<GetPersonDetails>))
          .route("/mention", web::get().to(route_get::<GetPersonMentions>))
          .route(
            "/mention/mark_as_read",
            web::post().to(route_post::<MarkPersonMentionAsRead>),
          )
          .route("/replies", web::get().to(route_get::<GetReplies>))
          .route("/join", web::post().to(route_post::<UserJoin>))
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
          ),
      )
      .service(
        web::scope("/admin/purge")
          .wrap(rate_limit.message())
          .route("/person", web::post().to(route_post::<PurgePerson>))
          .route("/community", web::post().to(route_post::<PurgeCommunity>))
          .route("/post", web::post().to(route_post::<PurgePost>))
          .route("/comment", web::post().to(route_post::<PurgeComment>)),
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
  let res = data.perform(&context, None).await?;
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
  let res = data.perform(&context, None).await?;
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
  let res = data.perform(&context, None).await?;
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

pub async fn match_websocket_operation_crud(
  context: ContextData<LemmyContext>,
  id: ConnectionId,
  op: UserOperationCrud,
  data: Value,
) -> result::Result<String, LemmyError> {
  match op {
    // User ops
    UserOperationCrud::Register => {
      do_websocket_operation_crud::<Register>(context, id, op, data).await
    }
    UserOperationCrud::DeleteAccount => {
      do_websocket_operation_crud::<DeleteAccount>(context, id, op, data).await
    }

    // Private Message ops
    UserOperationCrud::CreatePrivateMessage => {
      do_websocket_operation_crud::<CreatePrivateMessage>(context, id, op, data).await
    }
    UserOperationCrud::EditPrivateMessage => {
      do_websocket_operation_crud::<EditPrivateMessage>(context, id, op, data).await
    }
    UserOperationCrud::DeletePrivateMessage => {
      do_websocket_operation_crud::<DeletePrivateMessage>(context, id, op, data).await
    }
    UserOperationCrud::GetPrivateMessages => {
      do_websocket_operation_crud::<GetPrivateMessages>(context, id, op, data).await
    }

    // Site ops
    UserOperationCrud::CreateSite => {
      do_websocket_operation_crud::<CreateSite>(context, id, op, data).await
    }
    UserOperationCrud::EditSite => {
      do_websocket_operation_crud::<EditSite>(context, id, op, data).await
    }
    UserOperationCrud::GetSite => {
      do_websocket_operation_crud::<GetSite>(context, id, op, data).await
    }

    // Community ops
    UserOperationCrud::ListCommunities => {
      do_websocket_operation_crud::<ListCommunities>(context, id, op, data).await
    }
    UserOperationCrud::CreateCommunity => {
      do_websocket_operation_crud::<CreateCommunity>(context, id, op, data).await
    }
    UserOperationCrud::EditCommunity => {
      do_websocket_operation_crud::<EditCommunity>(context, id, op, data).await
    }
    UserOperationCrud::DeleteCommunity => {
      do_websocket_operation_crud::<DeleteCommunity>(context, id, op, data).await
    }
    UserOperationCrud::RemoveCommunity => {
      do_websocket_operation_crud::<RemoveCommunity>(context, id, op, data).await
    }

    // Post ops
    UserOperationCrud::CreatePost => {
      do_websocket_operation_crud::<CreatePost>(context, id, op, data).await
    }
    UserOperationCrud::GetPost => {
      do_websocket_operation_crud::<GetPost>(context, id, op, data).await
    }
    UserOperationCrud::EditPost => {
      do_websocket_operation_crud::<EditPost>(context, id, op, data).await
    }
    UserOperationCrud::DeletePost => {
      do_websocket_operation_crud::<DeletePost>(context, id, op, data).await
    }
    UserOperationCrud::RemovePost => {
      do_websocket_operation_crud::<RemovePost>(context, id, op, data).await
    }

    // Comment ops
    UserOperationCrud::CreateComment => {
      do_websocket_operation_crud::<CreateComment>(context, id, op, data).await
    }
    UserOperationCrud::EditComment => {
      do_websocket_operation_crud::<EditComment>(context, id, op, data).await
    }
    UserOperationCrud::DeleteComment => {
      do_websocket_operation_crud::<DeleteComment>(context, id, op, data).await
    }
    UserOperationCrud::RemoveComment => {
      do_websocket_operation_crud::<RemoveComment>(context, id, op, data).await
    }
    UserOperationCrud::GetComment => {
      do_websocket_operation_crud::<GetComment>(context, id, op, data).await
    }
    // Emojis
    UserOperationCrud::CreateCustomEmoji => {
      do_websocket_operation_crud::<CreateCustomEmoji>(context, id, op, data).await
    }
    UserOperationCrud::EditCustomEmoji => {
      do_websocket_operation_crud::<EditCustomEmoji>(context, id, op, data).await
    }
    UserOperationCrud::DeleteCustomEmoji => {
      do_websocket_operation_crud::<DeleteCustomEmoji>(context, id, op, data).await
    }
  }
}

async fn do_websocket_operation_crud<'a, 'b, Data>(
  context: ContextData<LemmyContext>,
  id: ConnectionId,
  op: UserOperationCrud,
  data: Value,
) -> result::Result<String, LemmyError>
where
  Data: PerformCrud + SendActivity<Response = <Data as PerformCrud>::Response> + Send,
  for<'de> Data: Deserialize<'de>,
{
  let parsed_data: Data = serde_json::from_value(data)?;
  let res = parsed_data
    .perform(&web::Data::new(context.app_data().clone()), Some(id))
    .await?;
  SendActivity::send_activity(&parsed_data, &res, &context).await?;
  serialize_websocket_message(&op, &res)
}

pub async fn match_websocket_operation_apub(
  context: ContextData<LemmyContext>,
  id: ConnectionId,
  op: UserOperationApub,
  data: Value,
) -> result::Result<String, LemmyError> {
  match op {
    UserOperationApub::GetPersonDetails => {
      do_websocket_operation_apub::<GetPersonDetails>(context, id, op, data).await
    }
    UserOperationApub::GetCommunity => {
      do_websocket_operation_apub::<GetCommunity>(context, id, op, data).await
    }
    UserOperationApub::GetComments => {
      do_websocket_operation_apub::<GetComments>(context, id, op, data).await
    }
    UserOperationApub::GetPosts => {
      do_websocket_operation_apub::<GetPosts>(context, id, op, data).await
    }
    UserOperationApub::ResolveObject => {
      do_websocket_operation_apub::<ResolveObject>(context, id, op, data).await
    }
    UserOperationApub::Search => do_websocket_operation_apub::<Search>(context, id, op, data).await,
  }
}

async fn do_websocket_operation_apub<'a, 'b, Data>(
  context: ContextData<LemmyContext>,
  id: ConnectionId,
  op: UserOperationApub,
  data: Value,

) -> result::Result<String, LemmyError>
where
  Data: PerformApub + SendActivity<Response = <Data as PerformApub>::Response> + Send,
  for<'de> Data: Deserialize<'de>,
{
  let parsed_data: Data = serde_json::from_value(data)?;
  let res = parsed_data
    .perform(&context, Some(id))
    .await?;
  SendActivity::send_activity(&parsed_data, &res, &context).await?;
  serialize_websocket_message(&op, &res)
}

pub async fn match_websocket_operation(
  context: LemmyContext,
  id: ConnectionId,
  op: UserOperation,
  data: Value,
) -> result::Result<String, LemmyError> {
  match op {
    // User ops
    UserOperation::Login => do_websocket_operation::<Login>(context, id, op, data).await,
    UserOperation::GetCaptcha => do_websocket_operation::<GetCaptcha>(context, id, op, data).await,
    UserOperation::GetReplies => do_websocket_operation::<GetReplies>(context, id, op, data).await,
    UserOperation::AddAdmin => do_websocket_operation::<AddAdmin>(context, id, op, data).await,
    UserOperation::GetUnreadRegistrationApplicationCount => {
      do_websocket_operation::<GetUnreadRegistrationApplicationCount>(context, id, op, data).await
    }
    UserOperation::ListRegistrationApplications => {
      do_websocket_operation::<ListRegistrationApplications>(context, id, op, data).await
    }
    UserOperation::ApproveRegistrationApplication => {
      do_websocket_operation::<ApproveRegistrationApplication>(context, id, op, data).await
    }
    UserOperation::BanPerson => do_websocket_operation::<BanPerson>(context, id, op, data).await,
    UserOperation::GetBannedPersons => {
      do_websocket_operation::<GetBannedPersons>(context, id, op, data).await
    }
    UserOperation::BlockPerson => {
      do_websocket_operation::<BlockPerson>(context, id, op, data).await
    }
    UserOperation::GetPersonMentions => {
      do_websocket_operation::<GetPersonMentions>(context, id, op, data).await
    }
    UserOperation::MarkPersonMentionAsRead => {
      do_websocket_operation::<MarkPersonMentionAsRead>(context, id, op, data).await
    }
    UserOperation::MarkCommentReplyAsRead => {
      do_websocket_operation::<MarkCommentReplyAsRead>(context, id, op, data).await
    }
    UserOperation::MarkAllAsRead => {
      do_websocket_operation::<MarkAllAsRead>(context, id, op, data).await
    }
    UserOperation::PasswordReset => {
      do_websocket_operation::<PasswordReset>(context, id, op, data).await
    }
    UserOperation::PasswordChange => {
      do_websocket_operation::<PasswordChangeAfterReset>(context, id, op, data).await
    }
    UserOperation::UserJoin => do_websocket_operation::<UserJoin>(context, id, op, data).await,
    UserOperation::PostJoin => do_websocket_operation::<PostJoin>(context, id, op, data).await,
    UserOperation::CommunityJoin => {
      do_websocket_operation::<CommunityJoin>(context, id, op, data).await
    }
    UserOperation::ModJoin => do_websocket_operation::<ModJoin>(context, id, op, data).await,
    UserOperation::SaveUserSettings => {
      do_websocket_operation::<SaveUserSettings>(context, id, op, data).await
    }
    UserOperation::ChangePassword => {
      do_websocket_operation::<ChangePassword>(context, id, op, data).await
    }
    UserOperation::GetReportCount => {
      do_websocket_operation::<GetReportCount>(context, id, op, data).await
    }
    UserOperation::GetUnreadCount => {
      do_websocket_operation::<GetUnreadCount>(context, id, op, data).await
    }
    UserOperation::VerifyEmail => {
      do_websocket_operation::<VerifyEmail>(context, id, op, data).await
    }

    // Private Message ops
    UserOperation::MarkPrivateMessageAsRead => {
      do_websocket_operation::<MarkPrivateMessageAsRead>(context, id, op, data).await
    }
    UserOperation::CreatePrivateMessageReport => {
      do_websocket_operation::<CreatePrivateMessageReport>(context, id, op, data).await
    }
    UserOperation::ResolvePrivateMessageReport => {
      do_websocket_operation::<ResolvePrivateMessageReport>(context, id, op, data).await
    }
    UserOperation::ListPrivateMessageReports => {
      do_websocket_operation::<ListPrivateMessageReports>(context, id, op, data).await
    }

    // Site ops
    UserOperation::GetModlog => do_websocket_operation::<GetModlog>(context, id, op, data).await,
    UserOperation::PurgePerson => {
      do_websocket_operation::<PurgePerson>(context, id, op, data).await
    }
    UserOperation::PurgeCommunity => {
      do_websocket_operation::<PurgeCommunity>(context, id, op, data).await
    }
    UserOperation::PurgePost => do_websocket_operation::<PurgePost>(context, id, op, data).await,
    UserOperation::PurgeComment => {
      do_websocket_operation::<PurgeComment>(context, id, op, data).await
    }
    UserOperation::TransferCommunity => {
      do_websocket_operation::<TransferCommunity>(context, id, op, data).await
    }
    UserOperation::LeaveAdmin => do_websocket_operation::<LeaveAdmin>(context, id, op, data).await,

    // Community ops
    UserOperation::FollowCommunity => {
      do_websocket_operation::<FollowCommunity>(context, id, op, data).await
    }
    UserOperation::BlockCommunity => {
      do_websocket_operation::<BlockCommunity>(context, id, op, data).await
    }
    UserOperation::BanFromCommunity => {
      do_websocket_operation::<BanFromCommunity>(context, id, op, data).await
    }
    UserOperation::AddModToCommunity => {
      do_websocket_operation::<AddModToCommunity>(context, id, op, data).await
    }

    // Post ops
    UserOperation::LockPost => do_websocket_operation::<LockPost>(context, id, op, data).await,
    UserOperation::FeaturePost => {
      do_websocket_operation::<FeaturePost>(context, id, op, data).await
    }
    UserOperation::CreatePostLike => {
      do_websocket_operation::<CreatePostLike>(context, id, op, data).await
    }
    UserOperation::MarkPostAsRead => {
      do_websocket_operation::<MarkPostAsRead>(context, id, op, data).await
    }
    UserOperation::SavePost => do_websocket_operation::<SavePost>(context, id, op, data).await,
    UserOperation::CreatePostReport => {
      do_websocket_operation::<CreatePostReport>(context, id, op, data).await
    }
    UserOperation::ListPostReports => {
      do_websocket_operation::<ListPostReports>(context, id, op, data).await
    }
    UserOperation::ResolvePostReport => {
      do_websocket_operation::<ResolvePostReport>(context, id, op, data).await
    }
    UserOperation::GetSiteMetadata => {
      do_websocket_operation::<GetSiteMetadata>(context, id, op, data).await
    }

    // Comment ops
    UserOperation::SaveComment => {
      do_websocket_operation::<SaveComment>(context, id, op, data).await
    }
    UserOperation::CreateCommentLike => {
      do_websocket_operation::<CreateCommentLike>(context, id, op, data).await
    }
    UserOperation::DistinguishComment => {
      do_websocket_operation::<DistinguishComment>(context, id, op, data).await
    }
    UserOperation::CreateCommentReport => {
      do_websocket_operation::<CreateCommentReport>(context, id, op, data).await
    }
    UserOperation::ListCommentReports => {
      do_websocket_operation::<ListCommentReports>(context, id, op, data).await
    }
    UserOperation::ResolveCommentReport => {
      do_websocket_operation::<ResolveCommentReport>(context, id, op, data).await
    }
  }
}

async fn do_websocket_operation<'a, 'b, Data>(
  context: LemmyContext,
  id: ConnectionId,
  op: UserOperation,
  data: Value,
) -> result::Result<String, LemmyError>
where
  Data: Perform + SendActivity<Response = <Data as Perform>::Response> + Send,
  for<'de> Data: Deserialize<'de>,
{
  let parsed_data: Data = serde_json::from_value(data)?;
  let res = parsed_data
    .perform(&web::Data::new(context.clone()), Some(id))
    .await?;
    // TODO 
  // SendActivity::send_activity(&parsed_data, &res, &context).await?;
  serialize_websocket_message(&op, &res)
}
