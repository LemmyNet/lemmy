pub mod api;
pub mod federation;
pub mod feeds;
pub mod index;
pub mod nodeinfo;
pub mod webfinger;
pub mod websocket;

use crate::{rate_limit::rate_limiter::RateLimiter, websocket::server::ChatServer};
use actix::prelude::*;
use actix_web::*;
use diesel::{
  r2d2::{ConnectionManager, Pool},
  PgConnection,
};
use std::sync::{Arc, Mutex};

pub type DbPoolParam = web::Data<Pool<ConnectionManager<PgConnection>>>;
pub type RateLimitParam = web::Data<Arc<Mutex<RateLimiter>>>;
pub type ChatServerParam = web::Data<Addr<ChatServer>>;
