extern crate server;
#[macro_use] extern crate diesel_migrations;

use std::time::{Instant, Duration};
use std::env;
use server::actix::*;
use server::actix_web::server::HttpServer;
use server::actix_web::{ws, App, Error, HttpRequest, HttpResponse, fs::NamedFile, fs};

use server::websocket_server::server::*;
use server::establish_connection;

embed_migrations!();

/// How often heartbeat pings are sent
const HEARTBEAT_INTERVAL: Duration = Duration::from_secs(5);
/// How long before lack of client response causes a timeout
const CLIENT_TIMEOUT: Duration = Duration::from_secs(10);

/// This is our websocket route state, this state is shared with all route
/// instances via `HttpContext::state()`
struct WsChatSessionState {
  addr: Addr<ChatServer>,
}

/// Entry point for our route
fn chat_route(req: &HttpRequest<WsChatSessionState>) -> Result<HttpResponse, Error> {
  ws::start(
    req,
    WSSession {
      id: 0,
      hb: Instant::now()
    },
    )
}

struct WSSession {
  /// unique session id
  id: usize,
  /// Client must send ping at least once per 10 seconds (CLIENT_TIMEOUT),
  /// otherwise we drop connection.
  hb: Instant
}

impl Actor for WSSession {
  type Context = ws::WebsocketContext<Self, WsChatSessionState>;

  /// Method is called on actor start.
  /// We register ws session with ChatServer
  fn started(&mut self, ctx: &mut Self::Context) {
    // we'll start heartbeat process on session start.
    self.hb(ctx);

    // register self in chat server. `AsyncContext::wait` register
    // future within context, but context waits until this future resolves
    // before processing any other events.
    // HttpContext::state() is instance of WsChatSessionState, state is shared
    // across all routes within application
    let addr = ctx.address();
    ctx.state()
      .addr
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
        fut::ok(())
      })
    .wait(ctx);
  }

  fn stopping(&mut self, ctx: &mut Self::Context) -> Running {
    // notify chat server
    ctx.state().addr.do_send(Disconnect { id: self.id });
    Running::Stop
  }
}

/// Handle messages from chat server, we simply send it to peer websocket
impl Handler<WSMessage> for WSSession {
  type Result = ();

  fn handle(&mut self, msg: WSMessage, ctx: &mut Self::Context) {
    println!("id: {} msg: {}", self.id, msg.0);
    ctx.text(msg.0);
  }
}

/// WebSocket message handler
impl StreamHandler<ws::Message, ws::ProtocolError> for WSSession {
  fn handle(&mut self, msg: ws::Message, ctx: &mut Self::Context) {
    println!("WEBSOCKET MESSAGE: {:?} from id: {}", msg, self.id);
    match msg {
      ws::Message::Ping(msg) => {
        self.hb = Instant::now();
        ctx.pong(&msg);
      }
      ws::Message::Pong(_) => {
        self.hb = Instant::now();
      }
      ws::Message::Text(text) => {
        let m = text.trim().to_owned();
        
        ctx.state()
          .addr
          .send(StandardMessage {
            id: self.id,
            msg: m
          })
        .into_actor(self)
          .then(|res, _, ctx| {
            match res {
              Ok(res) => ctx.text(res),
              Err(e) => {
                eprintln!("{}", &e);
                // ctx.text(e);
              }
            }
            // Ok(res) => ctx.text(res),
            // // something is wrong with chat server
            // _ => ctx.stop(),
            fut::ok(())
          })
        .wait(ctx);

        // we check for /sss type of messages
        // if m.starts_with('/') {
        //     let v: Vec<&str> = m.splitn(2, ' ').collect();
        //     match v[0] {
        //         "/list" => {
        //             // Send ListRooms message to chat server and wait for
        //             // response
        //             println!("List rooms");
        //             ctx.state()
        //                 .addr
        //                 .send(ListRooms)
        //                 .into_actor(self)
        //                 .then(|res, _, ctx| {
        //                     match res {
        //                         Ok(rooms) => {
        //                             for room in rooms {
        //                                 ctx.text(room);
        //                             }
        //                         }
        //                         _ => println!("Something is wrong"),
        //                     }
        //                     fut::ok(())
        //                 })
        //                 .wait(ctx)
        // .wait(ctx) pauses all events in context,
        // so actor wont receive any new messages until it get list
        // of rooms back
        // }
        // "/join" => {
        //     if v.len() == 2 {
        //         self.room = v[1].to_owned();
        //         ctx.state().addr.do_send(Join {
        //             id: self.id,
        //             name: self.room.clone(),
        //         });

        //         ctx.text("joined");
        //     } else {
        //         ctx.text("!!! room name is required");
        //     }
        // }
        // "/name" => {
        //     if v.len() == 2 {
        //         self.name = Some(v[1].to_owned());
        //     } else {
        //         ctx.text("!!! name is required");
        //     }
        // }
        // _ => ctx.text(format!("!!! unknown command: {:?}", m)),
        // }
        // } else {
        // let msg = if let Some(ref name) = self.name {
        //     format!("{}: {}", name, m)
        // } else {
        //     m.to_owned()
        // };
        // send message to chat server
        // ctx.state().addr.do_send(ClientMessage {
        // id: self.id,
        // msg: msg,
        // room: self.room.clone(),
        // })
        // }
      }
      ws::Message::Binary(_bin) => println!("Unexpected binary"),
      ws::Message::Close(_) => {
        ctx.stop();
      },
    }
  }
}

impl WSSession {
  /// helper method that sends ping to client every second.
  ///
  /// also this method checks heartbeats from client
  fn hb(&self, ctx: &mut ws::WebsocketContext<Self, WsChatSessionState>) {
    ctx.run_interval(HEARTBEAT_INTERVAL, |act, ctx| {
      // check client heartbeats
      if Instant::now().duration_since(act.hb) > CLIENT_TIMEOUT {
        // heartbeat timed out
        println!("Websocket Client heartbeat failed, disconnecting!");

        // notify chat server
        ctx.state()
          .addr
          .do_send(Disconnect { id: act.id });

        // stop actor
        ctx.stop();

        // don't try to send a ping
        return;
      }

      ctx.ping("");
    });
  }
}

fn main() {
  let _ = env_logger::init();
  let sys = actix::System::new("lemmy");

  // Run the migrations from code
  let conn = establish_connection();
  embedded_migrations::run(&conn).unwrap();

  // Start chat server actor in separate thread
  let server = Arbiter::start(|_| ChatServer::default());

  // Create Http server with websocket support
  HttpServer::new(move || {
    // Websocket sessions state
    let state = WsChatSessionState {
      addr: server.clone(),
    };

    App::with_state(state)
      // redirect to websocket.html
      // .resource("/", |r| r.method(http::Method::GET).f(|_| {
      // HttpResponse::Found()
      // .header("LOCATION", "/static/websocket.html")
      // .finish()
      // }))
      .resource("/service/ws", |r| r.route().f(chat_route))
      // static resources
      .resource("/", |r| r.route().f(index))
      .handler(
        "/static",
        fs::StaticFiles::new(front_end_dir()).unwrap()
      )
      .finish()
  }).bind("0.0.0.0:8536")
  .unwrap()
    .start();

  println!("Started http server: 0.0.0.0:8536");
  let _ = sys.run();
}

fn index(_req: &HttpRequest<WsChatSessionState>) -> Result<NamedFile, actix_web::error::Error> {
  Ok(NamedFile::open(front_end_dir() + "/index.html")?)
}

fn front_end_dir() -> String {
  env::var("LEMMY_FRONT_END_DIR").unwrap_or("../ui/dist".to_string())
}
