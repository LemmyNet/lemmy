use crate::websocket::server::*;
use crate::Settings;
use actix::prelude::*;
use actix_web::web;
use actix_web::*;
use actix_web_actors::ws;
use diesel::r2d2::{ConnectionManager, Pool};
use diesel::PgConnection;
use std::time::{Duration, Instant};

pub fn config(cfg: &mut web::ServiceConfig) {
  // TODO couldn't figure out how to get this method to recieve the other pool
  let settings = Settings::get();
  let manager = ConnectionManager::<PgConnection>::new(&settings.get_database_url());
  let pool = Pool::builder()
    .max_size(settings.database.pool_size)
    .build(manager)
    .unwrap_or_else(|_| panic!("Error connecting to {}", settings.get_database_url()));

  // Start chat server actor in separate thread
  let server = ChatServer::startup(pool).start();
  cfg
    .data(server)
    .service(web::resource("/api/v1/ws").to(chat_route));
}

/// How often heartbeat pings are sent
const HEARTBEAT_INTERVAL: Duration = Duration::from_secs(5);
/// How long before lack of client response causes a timeout
const CLIENT_TIMEOUT: Duration = Duration::from_secs(10);

/// Entry point for our route
async fn chat_route(
  req: HttpRequest,
  stream: web::Payload,
  chat_server: web::Data<Addr<ChatServer>>,
) -> Result<HttpResponse, Error> {
  // TODO not sure if the blocking should be here or not
  ws::start(
    WSSession {
      // db: db.get_ref().clone(),
      cs_addr: chat_server.get_ref().clone(),
      id: 0,
      hb: Instant::now(),
      ip: req
        .connection_info()
        .remote()
        .unwrap_or("127.0.0.1:12345")
        .split(':')
        .next()
        .unwrap_or("127.0.0.1")
        .to_string(),
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
    // println!("id: {} msg: {}", self.id, msg.0);
    ctx.text(msg.0);
  }
}

/// WebSocket message handler
impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for WSSession {
  fn handle(&mut self, result: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
    // println!("WEBSOCKET MESSAGE: {:?} from id: {}", msg, self.id);
    let message = match result {
      Ok(m) => m,
      Err(e) => {
        println!("{}", e);
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
        println!("WEBSOCKET MESSAGE: {:?} from id: {}", &m, self.id);

        self
          .cs_addr
          .send(StandardMessage {
            id: self.id,
            msg: m,
          })
          .into_actor(self)
          .then(|res, _, ctx| {
            match res {
              Ok(res) => ctx.text(res),
              Err(e) => {
                eprintln!("{}", &e);
              }
            }
            actix::fut::ready(())
          })
          .wait(ctx);
      }
      ws::Message::Binary(_bin) => println!("Unexpected binary"),
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
        println!("Websocket Client heartbeat failed, disconnecting!");

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
