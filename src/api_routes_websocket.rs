use activitypub_federation::config::Data as ContextData;
use actix::{
  fut,
  Actor,
  ActorContext,
  ActorFutureExt,
  AsyncContext,
  ContextFutureSpawner,
  Handler,
  Running,
  StreamHandler,
  WrapFuture,
};
use actix_web::{web, Error, HttpRequest, HttpResponse};
use actix_web_actors::ws;
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
  websocket::{
    handlers::{
      connect::{Connect, Disconnect},
      WsMessage,
    },
    serialize_websocket_message,
    structs::{CommunityJoin, ModJoin, PostJoin, UserJoin},
    UserOperation,
    UserOperationApub,
    UserOperationCrud,
  },
};
use lemmy_api_crud::PerformCrud;
use lemmy_apub::{api::PerformApub, SendActivity};
use lemmy_routes::WithAuth;
use lemmy_utils::{
  error::{LemmyError, LemmyResult},
  rate_limit::RateLimitCell,
  ConnectionId,
  IpAddr,
};
use serde::Deserialize;
use serde_json::Value;
use std::{
  ops::Deref,
  result,
  str::FromStr,
  time::{Duration, Instant},
};
use tracing::{debug, error};

/// How often heartbeat pings are sent
const HEARTBEAT_INTERVAL: Duration = Duration::from_secs(25);

/// How long before lack of client response causes a timeout
const CLIENT_TIMEOUT: Duration = Duration::from_secs(60);

pub struct WsChatSession {
  /// unique session id
  pub id: ConnectionId,

  pub ip: IpAddr,

  /// Client must send ping at least once per 10 seconds (CLIENT_TIMEOUT),
  /// otherwise we drop connection.
  pub hb: Instant,

  /// The context data
  apub_data: ContextData<LemmyContext>,
}

pub async fn websocket(
  req: HttpRequest,
  body: web::Payload,
  rate_limiter: web::Data<RateLimitCell>,
  apub_data: ContextData<LemmyContext>,
) -> Result<HttpResponse, Error> {
  let client_ip = IpAddr(
    req
      .connection_info()
      .realip_remote_addr()
      .unwrap_or("blank_ip")
      .to_string(),
  );

  let check = rate_limiter.message().check(client_ip.clone());
  if !check {
    debug!(
      "Websocket join with IP: {} has been rate limited.",
      &client_ip
    );
    return Ok(HttpResponse::TooManyRequests().finish());
  }

  ws::start(
    WsChatSession {
      id: 0,
      ip: client_ip,
      hb: Instant::now(),
      apub_data,
    },
    &req,
    body,
  )
}

/// helper method that sends ping to client every few seconds (HEARTBEAT_INTERVAL).
///
/// also this method checks heartbeats from client
fn hb(ctx: &mut ws::WebsocketContext<WsChatSession>) {
  ctx.run_interval(HEARTBEAT_INTERVAL, |act, ctx| {
    // check client heartbeats
    if Instant::now().duration_since(act.hb) > CLIENT_TIMEOUT {
      // heartbeat timed out

      // notify chat server
      act
        .apub_data
        .chat_server()
        .do_send(Disconnect { id: act.id });

      // stop actor
      ctx.stop();

      // don't try to send a ping
      return;
    }

    ctx.ping(b"");
  });
}

impl Actor for WsChatSession {
  type Context = ws::WebsocketContext<Self>;

  /// Method is called on actor start.
  /// We register ws session with ChatServer
  fn started(&mut self, ctx: &mut Self::Context) {
    // we'll start heartbeat process on session start.
    hb(ctx);

    // register self in chat server. `AsyncContext::wait` register
    // future within context, but context waits until this future resolves
    // before processing any other events.
    // HttpContext::state() is instance of WsChatSessionState, state is shared
    // across all routes within application
    let addr = ctx.address();
    self
      .apub_data
      .chat_server()
      .send(Connect {
        addr: addr.recipient(),
      })
      .into_actor(self)
      .then(|res, act, ctx| {
        match res {
          Ok(res) => act.id = res,
          // something is wrong with chat server
          _ => ctx.stop(),
        }
        fut::ready(())
      })
      .wait(ctx);
  }
  fn stopping(&mut self, _: &mut Self::Context) -> Running {
    // notify chat server
    self
      .apub_data
      .chat_server()
      .do_send(Disconnect { id: self.id });
    Running::Stop
  }
}

/// Handle messages from chat server, we simply send it to peer websocket
impl Handler<WsMessage> for WsChatSession {
  type Result = ();

  fn handle(&mut self, msg: WsMessage, ctx: &mut Self::Context) {
    ctx.text(msg.0);
  }
}

/// WebSocket message handler
impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for WsChatSession {
  fn handle(&mut self, msg: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
    let msg = match msg {
      Err(_) => {
        ctx.stop();
        return;
      }
      Ok(msg) => msg,
    };

    match msg {
      ws::Message::Ping(msg) => {
        self.hb = Instant::now();
        ctx.pong(&msg);
      }
      ws::Message::Pong(_) => {
        self.hb = Instant::now();
      }
      ws::Message::Text(text) => {
        let ip_clone = self.ip.clone();
        let id_clone = self.id.to_owned();
        let context_clone = self.apub_data.reset_request_count();

        let fut = Box::pin(async move {
          let msg = text.trim().to_string();
          parse_json_message(msg, ip_clone, id_clone, context_clone).await
        });
        fut
          .into_actor(self)
          .then(|res, _, ctx| {
            match res {
              Ok(res) => ctx.text(res),
              Err(e) => error!("{}", &e),
            }
            actix::fut::ready(())
          })
          .spawn(ctx);
      }
      ws::Message::Binary(_) => println!("Unexpected binary"),
      ws::Message::Close(reason) => {
        ctx.close(reason);
        ctx.stop();
      }
      ws::Message::Continuation(_) => {
        ctx.stop();
      }
      ws::Message::Nop => (),
    }
  }
}

/// Entry point for our websocket route
async fn parse_json_message(
  msg: String,
  ip: IpAddr,
  connection_id: ConnectionId,
  context: ContextData<LemmyContext>,
) -> Result<String, LemmyError> {
  let rate_limiter = context.settings_updated_channel();
  let json: Value = serde_json::from_str(&msg)?;
  let data = json
    .get("data")
    .cloned()
    .ok_or_else(|| LemmyError::from_message("missing data"))?;

  let missing_op_err = || LemmyError::from_message("missing op");

  let op = json
    .get("op")
    .ok_or_else(missing_op_err)?
    .as_str()
    .ok_or_else(missing_op_err)?;

  // check if api call passes the rate limit, and generate future for later execution
  if let Ok(user_operation_crud) = UserOperationCrud::from_str(op) {
    let passed = match user_operation_crud {
      UserOperationCrud::Register => rate_limiter.register().check(ip),
      UserOperationCrud::CreatePost => rate_limiter.post().check(ip),
      UserOperationCrud::CreateCommunity => rate_limiter.register().check(ip),
      UserOperationCrud::CreateComment => rate_limiter.comment().check(ip),
      _ => rate_limiter.message().check(ip),
    };
    check_rate_limit_passed(passed)?;
    match_websocket_operation_crud(context, connection_id, user_operation_crud, data).await
  } else if let Ok(user_operation) = UserOperation::from_str(op) {
    let passed = match user_operation {
      UserOperation::GetCaptcha => rate_limiter.post().check(ip),
      _ => rate_limiter.message().check(ip),
    };
    check_rate_limit_passed(passed)?;
    match_websocket_operation(context, connection_id, user_operation, data).await
  } else {
    let user_operation = UserOperationApub::from_str(op)?;
    let passed = match user_operation {
      UserOperationApub::Search => rate_limiter.search().check(ip),
      _ => rate_limiter.message().check(ip),
    };
    check_rate_limit_passed(passed)?;
    match_websocket_operation_apub(context, connection_id, user_operation, data).await
  }
}

fn check_rate_limit_passed(passed: bool) -> Result<(), LemmyError> {
  if passed {
    Ok(())
  } else {
    // if rate limit was hit, respond with message
    Err(LemmyError::from_message("rate_limit_error"))
  }
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
) -> LemmyResult<String>
where
  Data: PerformCrud + SendActivity<Response = <Data as PerformCrud>::Response> + Send,
  for<'de> Data: Deserialize<'de>,
{
  let parsed_data: WithAuth<Data> = serde_json::from_value(data)?;
  let data = parsed_data.data;
  let auth = parsed_data.auth;
  let res = data
    .perform(
      &web::Data::new(context.deref().clone()),
      auth.clone(),
      Some(id),
    )
    .await?;
  SendActivity::send_activity(&data, auth, &res, &context).await?;
  serialize_websocket_message(&op, &res)
}

pub async fn match_websocket_operation_apub(
  context: ContextData<LemmyContext>,
  id: ConnectionId,
  op: UserOperationApub,
  data: Value,
) -> LemmyResult<String> {
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
) -> LemmyResult<String>
where
  Data: PerformApub + SendActivity<Response = <Data as PerformApub>::Response> + Send,
  for<'de> Data: Deserialize<'de>,
{
  let parsed_data: WithAuth<Data> = serde_json::from_value(data)?;
  let data = parsed_data.data;
  let auth = parsed_data.auth;
  let res = data.perform(&context, auth.clone(), Some(id)).await?;
  SendActivity::send_activity(&data, auth, &res, &context).await?;
  serialize_websocket_message(&op, &res)
}

pub async fn match_websocket_operation(
  context: ContextData<LemmyContext>,
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
    UserOperation::GetFederatedInstances => {
      do_websocket_operation::<GetFederatedInstances>(context, id, op, data).await
    }

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
  context: ContextData<LemmyContext>,
  id: ConnectionId,
  op: UserOperation,
  data: Value,
) -> result::Result<String, LemmyError>
where
  Data: Perform + SendActivity<Response = <Data as Perform>::Response> + Send,
  for<'de> Data: Deserialize<'de>,
{
  let parsed_data: WithAuth<Data> = serde_json::from_value(data)?;
  let data = parsed_data.data;
  let auth = parsed_data.auth;
  let res = data
    .perform(
      &web::Data::new(context.deref().clone()),
      auth.clone(),
      Some(id),
    )
    .await?;
  SendActivity::send_activity(&data, auth, &res, &context).await?;
  serialize_websocket_message(&op, &res)
}
