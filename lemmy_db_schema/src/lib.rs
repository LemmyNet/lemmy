#[macro_use]
extern crate diesel;

use chrono::NaiveDateTime;

pub mod schema;
pub mod source;

pub fn naive_now() -> NaiveDateTime {
  chrono::prelude::Utc::now().naive_utc()
}
