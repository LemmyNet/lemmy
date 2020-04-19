use crate::api::{Oper, Perform};
use crate::db::site_view::SiteView;
use crate::rate_limit::{rate_limiter::RateLimiter, RateLimitInfo};
use crate::websocket::{server::ChatServer, WebsocketInfo};
use crate::{get_ip, markdown_to_html, version, Settings};
use actix::prelude::*;
use actix_files::NamedFile;
use actix_web::{body::Body, web::Query, *};
use actix_web_actors::ws;
use chrono::{DateTime, NaiveDateTime, Utc};
use diesel::{
  r2d2::{ConnectionManager, Pool},
  PgConnection,
};
use failure::Error;
use log::{error, info};
use regex::Regex;
use rss::{CategoryBuilder, ChannelBuilder, GuidBuilder, Item, ItemBuilder};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::str::FromStr;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use strum::ParseError;

pub type DbPoolParam = web::Data<Pool<ConnectionManager<PgConnection>>>;
pub type RateLimitParam = web::Data<Arc<Mutex<RateLimiter>>>;
pub type ChatServerParam = web::Data<Addr<ChatServer>>;

pub mod api;
pub mod federation;
pub mod feeds;
pub mod index;
pub mod nodeinfo;
pub mod webfinger;
pub mod websocket;
