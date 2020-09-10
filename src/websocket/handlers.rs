use crate::{
  api::Perform,
  websocket::{
    chat_server::{ChatServer, SessionInfo},
    messages::*,
    UserOperation,
  },
  LemmyContext,
};
use actix::{Actor, Context, Handler, ResponseFuture};
use actix_web::web;
use lemmy_db::naive_now;
use lemmy_rate_limit::RateLimit;
use lemmy_utils::{ConnectionId, IPAddr, LemmyError};
use log::{error, info};
use rand::Rng;
use serde::{Deserialize, Serialize};

pub(super) struct Args<'a> {
  pub(super) context: LemmyContext,
  pub(super) rate_limiter: RateLimit,
  pub(super) id: ConnectionId,
  pub(super) ip: IPAddr,
  pub(super) op: UserOperation,
  pub(super) data: &'a str,
}

pub(super) async fn do_user_operation<'a, 'b, Data>(args: Args<'b>) -> Result<String, LemmyError>
where
  for<'de> Data: Deserialize<'de> + 'a,
  Data: Perform,
{
  let Args {
    context,
    rate_limiter,
    id,
    ip,
    op,
    data,
  } = args;

  let data = data.to_string();
  let op2 = op.clone();

  let fut = async move {
    let parsed_data: Data = serde_json::from_str(&data)?;
    let res = parsed_data
      .perform(&web::Data::new(context), Some(id))
      .await?;
    to_json_string(&op, &res)
  };

  match op2 {
    UserOperation::Register => rate_limiter.register().wrap(ip, fut).await,
    UserOperation::CreatePost => rate_limiter.post().wrap(ip, fut).await,
    UserOperation::CreateCommunity => rate_limiter.register().wrap(ip, fut).await,
    _ => rate_limiter.message().wrap(ip, fut).await,
  }
}

/// Make actor from `ChatServer`
impl Actor for ChatServer {
  /// We are going to use simple Context, we just need ability to communicate
  /// with other actors.
  type Context = Context<Self>;
}

/// Handler for Connect message.
///
/// Register new session and assign unique id to this session
impl Handler<Connect> for ChatServer {
  type Result = usize;

  fn handle(&mut self, msg: Connect, _ctx: &mut Context<Self>) -> Self::Result {
    // register session with random id
    let id = self.rng.gen::<usize>();
    info!("{} joined", &msg.ip);

    self.sessions.insert(
      id,
      SessionInfo {
        addr: msg.addr,
        ip: msg.ip,
      },
    );

    id
  }
}

/// Handler for Disconnect message.
impl Handler<Disconnect> for ChatServer {
  type Result = ();

  fn handle(&mut self, msg: Disconnect, _: &mut Context<Self>) {
    // Remove connections from sessions and all 3 scopes
    if self.sessions.remove(&msg.id).is_some() {
      for sessions in self.user_rooms.values_mut() {
        sessions.remove(&msg.id);
      }

      for sessions in self.post_rooms.values_mut() {
        sessions.remove(&msg.id);
      }

      for sessions in self.community_rooms.values_mut() {
        sessions.remove(&msg.id);
      }
    }
  }
}

/// Handler for Message message.
impl Handler<StandardMessage> for ChatServer {
  type Result = ResponseFuture<Result<String, std::convert::Infallible>>;

  fn handle(&mut self, msg: StandardMessage, ctx: &mut Context<Self>) -> Self::Result {
    let fut = self.parse_json_message(msg, ctx);
    Box::pin(async move {
      match fut.await {
        Ok(m) => {
          // info!("Message Sent: {}", m);
          Ok(m)
        }
        Err(e) => {
          error!("Error during message handling {}", e);
          Ok(e.to_string())
        }
      }
    })
  }
}

impl<Response> Handler<SendAllMessage<Response>> for ChatServer
where
  Response: Serialize,
{
  type Result = ();

  fn handle(&mut self, msg: SendAllMessage<Response>, _: &mut Context<Self>) {
    self
      .send_all_message(&msg.op, &msg.response, msg.websocket_id)
      .ok();
  }
}

impl<Response> Handler<SendUserRoomMessage<Response>> for ChatServer
where
  Response: Serialize,
{
  type Result = ();

  fn handle(&mut self, msg: SendUserRoomMessage<Response>, _: &mut Context<Self>) {
    self
      .send_user_room_message(&msg.op, &msg.response, msg.recipient_id, msg.websocket_id)
      .ok();
  }
}

impl<Response> Handler<SendCommunityRoomMessage<Response>> for ChatServer
where
  Response: Serialize,
{
  type Result = ();

  fn handle(&mut self, msg: SendCommunityRoomMessage<Response>, _: &mut Context<Self>) {
    self
      .send_community_room_message(&msg.op, &msg.response, msg.community_id, msg.websocket_id)
      .ok();
  }
}

impl Handler<SendPost> for ChatServer {
  type Result = ();

  fn handle(&mut self, msg: SendPost, _: &mut Context<Self>) {
    self.send_post(&msg.op, &msg.post, msg.websocket_id).ok();
  }
}

impl Handler<SendComment> for ChatServer {
  type Result = ();

  fn handle(&mut self, msg: SendComment, _: &mut Context<Self>) {
    self
      .send_comment(&msg.op, &msg.comment, msg.websocket_id)
      .ok();
  }
}

impl Handler<JoinUserRoom> for ChatServer {
  type Result = ();

  fn handle(&mut self, msg: JoinUserRoom, _: &mut Context<Self>) {
    self.join_user_room(msg.user_id, msg.id).ok();
  }
}

impl Handler<JoinCommunityRoom> for ChatServer {
  type Result = ();

  fn handle(&mut self, msg: JoinCommunityRoom, _: &mut Context<Self>) {
    self.join_community_room(msg.community_id, msg.id).ok();
  }
}

impl Handler<JoinPostRoom> for ChatServer {
  type Result = ();

  fn handle(&mut self, msg: JoinPostRoom, _: &mut Context<Self>) {
    self.join_post_room(msg.post_id, msg.id).ok();
  }
}

impl Handler<GetUsersOnline> for ChatServer {
  type Result = usize;

  fn handle(&mut self, _msg: GetUsersOnline, _: &mut Context<Self>) -> Self::Result {
    self.sessions.len()
  }
}

impl Handler<GetPostUsersOnline> for ChatServer {
  type Result = usize;

  fn handle(&mut self, msg: GetPostUsersOnline, _: &mut Context<Self>) -> Self::Result {
    if let Some(users) = self.post_rooms.get(&msg.post_id) {
      users.len()
    } else {
      0
    }
  }
}

impl Handler<GetCommunityUsersOnline> for ChatServer {
  type Result = usize;

  fn handle(&mut self, msg: GetCommunityUsersOnline, _: &mut Context<Self>) -> Self::Result {
    if let Some(users) = self.community_rooms.get(&msg.community_id) {
      users.len()
    } else {
      0
    }
  }
}

#[derive(Serialize)]
struct WebsocketResponse<T> {
  op: String,
  data: T,
}

pub(super) fn to_json_string<Response>(
  op: &UserOperation,
  data: &Response,
) -> Result<String, LemmyError>
where
  Response: Serialize,
{
  let response = WebsocketResponse {
    op: op.to_string(),
    data,
  };
  Ok(serde_json::to_string(&response)?)
}

impl Handler<CaptchaItem> for ChatServer {
  type Result = ();

  fn handle(&mut self, msg: CaptchaItem, _: &mut Context<Self>) {
    self.captchas.push(msg);
  }
}

impl Handler<CheckCaptcha> for ChatServer {
  type Result = bool;

  fn handle(&mut self, msg: CheckCaptcha, _: &mut Context<Self>) -> Self::Result {
    // Remove all the ones that are past the expire time
    self.captchas.retain(|x| x.expires.gt(&naive_now()));

    let check = self
      .captchas
      .iter()
      .any(|r| r.uuid == msg.uuid && r.answer == msg.answer);

    // Remove this uuid so it can't be re-checked (Checks only work once)
    self.captchas.retain(|x| x.uuid != msg.uuid);

    check
  }
}
