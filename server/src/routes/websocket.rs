use super::*;
use crate::websocket::server::*;

/// How often heartbeat pings are sent
const HEARTBEAT_INTERVAL: Duration = Duration::from_secs(5);
/// How long before lack of client response causes a timeout
const CLIENT_TIMEOUT: Duration = Duration::from_secs(10);

/// Entry point for our route
pub async fn chat_route(
  req: HttpRequest,
  stream: web::Payload,
  chat_server: web::Data<Addr<ChatServer>>,
) -> Result<HttpResponse, Error> {
  ws::start(
    WSSession {
      cs_addr: chat_server.get_ref().to_owned(),
      id: 0,
      hb: Instant::now(),
      ip: get_ip(&req.connection_info()),
    },
    &req,
    stream,
  )
}

struct WSSession {
  cs_addr: Addr<ChatServer>,
  /// unique session id
  id: usize,
  ip: String,
  /// Client must send ping at least once per 10 seconds (CLIENT_TIMEOUT),
  /// otherwise we drop connection.
  hb: Instant,
  // db: Pool<ConnectionManager<PgConnection>>,
}

impl Actor for WSSession {
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
impl Handler<WSMessage> for WSSession {
  type Result = ();

  fn handle(&mut self, msg: WSMessage, ctx: &mut Self::Context) {
    ctx.text(msg.0);
  }
}

/// WebSocket message handler
impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for WSSession {
  fn handle(&mut self, result: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
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
        info!("Message received: {:?} from id: {}", &m, self.id);

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

impl WSSession {
  /// helper method that sends ping to client every second.
  ///
  /// also this method checks heartbeats from client
  fn hb(&self, ctx: &mut ws::WebsocketContext<Self>) {
    ctx.run_interval(HEARTBEAT_INTERVAL, |act, ctx| {
      // check client heartbeats
      if Instant::now().duration_since(act.hb) > CLIENT_TIMEOUT {
        // heartbeat timed out
        error!("Websocket Client heartbeat failed, disconnecting!");

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
}
