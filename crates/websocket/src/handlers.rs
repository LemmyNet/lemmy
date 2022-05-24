use crate::{
  chat_server::{ChatServer, SessionInfo},
  messages::*,
  OperationType,
};
use actix::{Actor, Context, Handler, ResponseFuture};
use lemmy_db_schema::utils::naive_now;
use lemmy_utils::ConnectionId;
use opentelemetry::trace::TraceContextExt;
use rand::Rng;
use serde::Serialize;
use tracing::{error, info};
use tracing_opentelemetry::OpenTelemetrySpanExt;

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
  type Result = ConnectionId;

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

fn root_span() -> tracing::Span {
  let span = tracing::info_span!(
    parent: None,
    "Websocket Request",
    trace_id = tracing::field::Empty,
  );
  {
    let trace_id = span.context().span().span_context().trace_id().to_string();
    span.record("trace_id", &tracing::field::display(trace_id));
  }

  span
}

/// Handler for Message message.
impl Handler<StandardMessage> for ChatServer {
  type Result = ResponseFuture<Result<String, std::convert::Infallible>>;

  fn handle(&mut self, msg: StandardMessage, ctx: &mut Context<Self>) -> Self::Result {
    let fut = self.parse_json_message(msg, ctx);
    let span = root_span();

    use tracing::Instrument;

    Box::pin(
      async move {
        match fut.await {
          Ok(m) => {
            // info!("Message Sent: {}", m);
            Ok(m)
          }
          Err(e) => {
            error!("Error during message handling {}", e);
            Ok(
              e.to_json()
                .unwrap_or_else(|_| String::from(r#"{"error":"failed to serialize json"}"#)),
            )
          }
        }
      }
      .instrument(span),
    )
  }
}

impl<OP, Response> Handler<SendAllMessage<OP, Response>> for ChatServer
where
  OP: OperationType + ToString,
  Response: Serialize,
{
  type Result = ();

  fn handle(&mut self, msg: SendAllMessage<OP, Response>, _: &mut Context<Self>) {
    self
      .send_all_message(&msg.op, &msg.response, msg.websocket_id)
      .ok();
  }
}

impl<OP, Response> Handler<SendUserRoomMessage<OP, Response>> for ChatServer
where
  OP: OperationType + ToString,
  Response: Serialize,
{
  type Result = ();

  fn handle(&mut self, msg: SendUserRoomMessage<OP, Response>, _: &mut Context<Self>) {
    self
      .send_user_room_message(
        &msg.op,
        &msg.response,
        msg.local_recipient_id,
        msg.websocket_id,
      )
      .ok();
  }
}

impl<OP, Response> Handler<SendCommunityRoomMessage<OP, Response>> for ChatServer
where
  OP: OperationType + ToString,
  Response: Serialize,
{
  type Result = ();

  fn handle(&mut self, msg: SendCommunityRoomMessage<OP, Response>, _: &mut Context<Self>) {
    self
      .send_community_room_message(&msg.op, &msg.response, msg.community_id, msg.websocket_id)
      .ok();
  }
}

impl<Response> Handler<SendModRoomMessage<Response>> for ChatServer
where
  Response: Serialize,
{
  type Result = ();

  fn handle(&mut self, msg: SendModRoomMessage<Response>, _: &mut Context<Self>) {
    self
      .send_mod_room_message(&msg.op, &msg.response, msg.community_id, msg.websocket_id)
      .ok();
  }
}

impl<OP> Handler<SendPost<OP>> for ChatServer
where
  OP: OperationType + ToString,
{
  type Result = ();

  fn handle(&mut self, msg: SendPost<OP>, _: &mut Context<Self>) {
    self.send_post(&msg.op, &msg.post, msg.websocket_id).ok();
  }
}

impl<OP> Handler<SendComment<OP>> for ChatServer
where
  OP: OperationType + ToString,
{
  type Result = ();

  fn handle(&mut self, msg: SendComment<OP>, _: &mut Context<Self>) {
    self
      .send_comment(&msg.op, &msg.comment, msg.websocket_id)
      .ok();
  }
}

impl Handler<JoinUserRoom> for ChatServer {
  type Result = ();

  fn handle(&mut self, msg: JoinUserRoom, _: &mut Context<Self>) {
    self.join_user_room(msg.local_user_id, msg.id).ok();
  }
}

impl Handler<JoinCommunityRoom> for ChatServer {
  type Result = ();

  fn handle(&mut self, msg: JoinCommunityRoom, _: &mut Context<Self>) {
    self.join_community_room(msg.community_id, msg.id).ok();
  }
}

impl Handler<JoinModRoom> for ChatServer {
  type Result = ();

  fn handle(&mut self, msg: JoinModRoom, _: &mut Context<Self>) {
    self.join_mod_room(msg.community_id, msg.id).ok();
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
      .any(|r| r.uuid == msg.uuid && r.answer.to_lowercase() == msg.answer.to_lowercase());

    // Remove this uuid so it can't be re-checked (Checks only work once)
    self.captchas.retain(|x| x.uuid != msg.uuid);

    check
  }
}
