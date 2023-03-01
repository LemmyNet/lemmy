use actix_web::{web, Error, HttpRequest, HttpResponse};
use actix_web_actors::ws;
use actix_ws::{MessageStream, Session};
use futures::stream::StreamExt;
use lemmy_api::Perform;
use lemmy_api_common::{
  comment::{
    CreateComment,
    CreateCommentLike,
    CreateCommentReport,
    DeleteComment,
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
    serialize_websocket_message,
    structs::{CommunityJoin, ModJoin, PostJoin, UserJoin},
    UserOperation,
    UserOperationApub,
    UserOperationCrud,
  },
};
use lemmy_api_crud::PerformCrud;
use lemmy_apub::{api::PerformApub, SendActivity};
use lemmy_utils::{error::LemmyError, rate_limit::RateLimitCell, ConnectionId, IpAddr};
use serde::Deserialize;
use serde_json::Value;
use std::{
  result,
  str::FromStr,
  sync::{Arc, Mutex},
  time::{Duration, Instant},
};
use tracing::{debug, error, info};

/// Entry point for our route
pub async fn websocket(
  req: HttpRequest,
  body: web::Payload,
  context: web::Data<LemmyContext>,
  rate_limiter: web::Data<RateLimitCell>,
) -> Result<HttpResponse, Error> {
  let (response, session, stream) = actix_ws::handle(&req, body)?;

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
    session.close(None).await.map_err(LemmyError::from)?;
    return Ok(response);
  }

  let connection_id = context.chat_server().handle_connect(session.clone())?;
  info!("{} joined", &client_ip);

  let alive = Arc::new(Mutex::new(Instant::now()));
  heartbeat(session.clone(), alive.clone());

  actix_rt::spawn(handle_messages(
    stream,
    client_ip,
    session,
    connection_id,
    alive,
    rate_limiter,
    context,
  ));

  Ok(response)
}

async fn handle_messages(
  mut stream: MessageStream,
  client_ip: IpAddr,
  mut session: Session,
  connection_id: ConnectionId,
  alive: Arc<Mutex<Instant>>,
  rate_limiter: web::Data<RateLimitCell>,
  context: web::Data<LemmyContext>,
) -> Result<(), LemmyError> {
  while let Some(Ok(msg)) = stream.next().await {
    match msg {
      ws::Message::Ping(bytes) => {
        if session.pong(&bytes).await.is_err() {
          break;
        }
      }
      ws::Message::Pong(_) => {
        let mut lock = alive
          .lock()
          .expect("Failed to acquire websocket heartbeat alive lock");
        *lock = Instant::now();
      }
      ws::Message::Text(text) => {
        let msg = text.trim().to_string();
        let executed = parse_json_message(
          msg,
          client_ip.clone(),
          connection_id,
          rate_limiter.get_ref(),
          context.get_ref().clone(),
        )
        .await;

        let res = executed.unwrap_or_else(|e| {
          error!("Error during message handling {}", e);
          e.to_json()
            .unwrap_or_else(|_| String::from(r#"{"error":"failed to serialize json"}"#))
        });
        session.text(res).await?;
      }
      ws::Message::Close(_) => {
        session.close(None).await?;
        context.chat_server().handle_disconnect(&connection_id)?;
        break;
      }
      ws::Message::Binary(_) => info!("Unexpected binary"),
      _ => {}
    }
  }
  Ok(())
}

fn heartbeat(mut session: Session, alive: Arc<Mutex<Instant>>) {
  actix_rt::spawn(async move {
    let mut interval = actix_rt::time::interval(Duration::from_secs(5));
    loop {
      if session.ping(b"").await.is_err() {
        break;
      }

      let duration_since = {
        let alive_lock = alive
          .lock()
          .expect("Failed to acquire websocket heartbeat alive lock");
        Instant::now().duration_since(*alive_lock)
      };
      if duration_since > Duration::from_secs(10) {
        let _ = session.close(None).await;
        break;
      }
      interval.tick().await;
    }
  });
}

async fn parse_json_message(
  msg: String,
  ip: IpAddr,
  connection_id: ConnectionId,
  rate_limiter: &RateLimitCell,
  context: LemmyContext,
) -> Result<String, LemmyError> {
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
  context: LemmyContext,
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
  }
}

async fn do_websocket_operation_crud<'a, 'b, Data>(
  context: LemmyContext,
  id: ConnectionId,
  op: UserOperationCrud,
  data: Value,
) -> result::Result<String, LemmyError>
where
  Data: PerformCrud + SendActivity<Response = <Data as PerformCrud>::Response>,
  for<'de> Data: Deserialize<'de>,
{
  let parsed_data: Data = serde_json::from_value(data)?;
  let res = parsed_data
    .perform(&web::Data::new(context.clone()), Some(id))
    .await?;
  SendActivity::send_activity(&parsed_data, &res, &context).await?;
  serialize_websocket_message(&op, &res)
}

pub async fn match_websocket_operation_apub(
  context: LemmyContext,
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
  context: LemmyContext,
  id: ConnectionId,
  op: UserOperationApub,
  data: Value,
) -> result::Result<String, LemmyError>
where
  Data: PerformApub + SendActivity<Response = <Data as PerformApub>::Response>,
  for<'de> Data: Deserialize<'de>,
{
  let parsed_data: Data = serde_json::from_value(data)?;
  let res = parsed_data
    .perform(&web::Data::new(context.clone()), Some(id))
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
  Data: Perform + SendActivity<Response = <Data as Perform>::Response>,
  for<'de> Data: Deserialize<'de>,
{
  let parsed_data: Data = serde_json::from_value(data)?;
  let res = parsed_data
    .perform(&web::Data::new(context.clone()), Some(id))
    .await?;
  SendActivity::send_activity(&parsed_data, &res, &context).await?;
  serialize_websocket_message(&op, &res)
}
