use crate::{
  chat_server::ChatServer,
  messages::{Connect, Disconnect, StandardMessage, WsMessage},
  LemmyContext,
};
use actix::prelude::*;
use actix_web::{web, Error, HttpRequest, HttpResponse};
use actix_web_actors::ws;
use lemmy_utils::{rate_limit::RateLimit, utils::get_ip, ConnectionId, IpAddr};
use std::time::{Duration, Instant};
use tracing::{debug, error, info};

/// How often heartbeat pings are sent
const HEARTBEAT_INTERVAL: Duration = Duration::from_secs(5);
/// How long before lack of client response causes a timeout
const CLIENT_TIMEOUT: Duration = Duration::from_secs(10);

/// Entry point for our route
pub async fn chat_route(
  req: HttpRequest,
  stream: web::Payload,
  context: web::Data<LemmyContext>,
  rate_limiter: web::Data<RateLimit>,
) -> Result<HttpResponse, Error> {
  ws::start(
    WsSession {
      cs_addr: context.chat_server().to_owned(),
      id: 0,
      hb: Instant::now(),
      ip: get_ip(&req.connection_info()),
      rate_limiter: rate_limiter.as_ref().to_owned(),
    },
    &req,
    stream,
  )
}

struct WsSession {
  cs_addr: Addr<ChatServer>,
  /// unique session id
  id: ConnectionId,
  ip: IpAddr,
  /// Client must send ping at least once per 10 seconds (CLIENT_TIMEOUT),
  /// otherwise we drop connection.
  hb: Instant,
  /// A rate limiter for websocket joins
  rate_limiter: RateLimit,
}

impl Actor for WsSession {
  type Context = ws::WebsocketContext<Self>;

  /// Method is called on actor start.
  /// We register ws session with ChatServer
  fn started(&mut self, ctx: &mut Self::Context) {
    // we'll start heartbeat process on session start.
    self.hb(ctx);

    // register self in chat server. `AsyncContext::wait` register
    // future within context, but context waits until this future resolves
    // before processing any other events.
    // across all routes within application
    let addr = ctx.address();

    if !self.rate_limit_check(ctx) {
      return;
    }

    self
      .cs_addr
      .send(Connect {
        addr: addr.recipient(),
        ip: self.ip.to_owned(),
      })
      .into_actor(self)
      .then(|res, act, ctx| {
        match res {
          Ok(res) => act.id = res,
          // something is wrong with chat server
          _ => ctx.stop(),
        }
        actix::fut::ready(())
      })
      .wait(ctx);
  }

  fn stopping(&mut self, _ctx: &mut Self::Context) -> Running {
    // notify chat server
    self.cs_addr.do_send(Disconnect {
      id: self.id,
      ip: self.ip.to_owned(),
    });
    Running::Stop
  }
}

/// Handle messages from chat server, we simply send it to peer websocket
/// These are room messages, IE sent to others in the room
impl Handler<WsMessage> for WsSession {
  type Result = ();

  fn handle(&mut self, msg: WsMessage, ctx: &mut Self::Context) {
    ctx.text(msg.0);
  }
}

/// WebSocket message handler
impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for WsSession {
  fn handle(&mut self, result: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
    if !self.rate_limit_check(ctx) {
      return;
    }

    let message = match result {
      Ok(m) => m,
      Err(e) => {
        error!("{}", e);
        return;
      }
    };
    match message {
      ws::Message::Ping(msg) => {
        self.hb = Instant::now();
        ctx.pong(&msg);
      }
      ws::Message::Pong(_) => {
        self.hb = Instant::now();
      }
      ws::Message::Text(text) => {
        let m = text.trim().to_owned();

        self
          .cs_addr
          .send(StandardMessage {
            id: self.id,
            msg: m,
          })
          .into_actor(self)
          .then(|res, _, ctx| {
            match res {
              Ok(Ok(res)) => ctx.text(res),
              Ok(Err(_)) => {}
              Err(e) => error!("{}", &e),
            }
            actix::fut::ready(())
          })
          .spawn(ctx);
      }
      ws::Message::Binary(_bin) => info!("Unexpected binary"),
      ws::Message::Close(_) => {
        ctx.stop();
      }
      _ => {}
    }
  }
}

impl WsSession {
  /// helper method that sends ping to client every second.
  ///
  /// also this method checks heartbeats from client
  fn hb(&self, ctx: &mut ws::WebsocketContext<Self>) {
    ctx.run_interval(HEARTBEAT_INTERVAL, |act, ctx| {
      // check client heartbeats
      if Instant::now().duration_since(act.hb) > CLIENT_TIMEOUT {
        // heartbeat timed out
        debug!("Websocket Client heartbeat failed, disconnecting!");

        // notify chat server
        act.cs_addr.do_send(Disconnect {
          id: act.id,
          ip: act.ip.to_owned(),
        });

        // stop actor
        ctx.stop();

        // don't try to send a ping
        return;
      }

      ctx.ping(b"");
    });
  }

  /// Check the rate limit, and stop the ctx if it fails
  fn rate_limit_check(&mut self, ctx: &mut ws::WebsocketContext<Self>) -> bool {
    let check = self.rate_limiter.message().check(self.ip.to_owned());
    if !check {
      debug!("Websocket join with IP: {} has been rate limited.", self.ip);
      ctx.stop()
    }
    check
  }
}
