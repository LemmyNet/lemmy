pub mod api;
pub mod federation;
pub mod feeds;
pub mod images;
pub mod index;
pub mod nodeinfo;
pub mod webfinger;
pub mod websocket;

use crate::websocket::server::ChatServer;
use actix::prelude::*;
use actix_web::*;
use diesel::{
  r2d2::{ConnectionManager, Pool},
  PgConnection,
};

pub type DbPoolParam = web::Data<Pool<ConnectionManager<PgConnection>>>;
pub type ChatServerParam = web::Data<Addr<ChatServer>>;
