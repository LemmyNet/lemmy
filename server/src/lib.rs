#[macro_use]
extern crate diesel;
extern crate dotenv;

use diesel::prelude::*;
use diesel::pg::PgConnection;
use dotenv::dotenv;
use std::env;

pub mod schema;
pub mod models;
pub mod activitypub;
pub mod actions;

pub fn establish_connection() -> PgConnection {
  dotenv().ok();

  let database_url = env::var("DATABASE_URL")
    .expect("DATABASE_URL must be set");
  PgConnection::establish(&database_url)
    .expect(&format!("Error connecting to {}", database_url))
}

trait Crud {
  fn read(conn: &PgConnection, id: i32) -> Self;
  fn delete(conn: &PgConnection, id: i32) -> usize;
  // fn create<T: Insertable>(conn: &PgConnection, item: T) -> Result<Self, Error> where Self: Sized;
}

#[cfg(test)]
mod tests {
  #[test]
  fn it_works() {
    assert_eq!(2 + 2, 4);
  }

  #[test]
  fn db_fetch() {

  }
}
