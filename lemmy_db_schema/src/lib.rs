#[macro_use]
extern crate diesel;

use chrono::NaiveDateTime;

pub mod schema;
pub mod source;

// TODO: can probably move this back to lemmy_db_queries
pub fn naive_now() -> NaiveDateTime {
  chrono::prelude::Utc::now().naive_utc()
}
