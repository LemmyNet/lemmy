#[macro_use]
extern crate diesel;
extern crate dotenv;

use diesel::*;
use diesel::pg::PgConnection;
use diesel::result::Error;
use dotenv::dotenv;
use std::env;

pub mod schema;
pub mod models;
pub mod actions;

// pub trait Likeable;
pub trait Crud<T> {
  fn create(conn: &PgConnection, form: T) -> Result<Self, Error> where Self: Sized;
  fn read(conn: &PgConnection, id: i32) -> Self;
  fn update(conn: &PgConnection, id: i32, form: T) -> Self;
  fn delete(conn: &PgConnection, id: i32) -> usize;
}

pub trait Followable<T> {
  fn follow(conn: &PgConnection, form: T) -> Result<Self, Error> where Self: Sized;
  fn ignore(conn: &PgConnection, form: T) -> usize;
}

pub trait Joinable<T> {
  fn join(conn: &PgConnection, form: T) -> Result<Self, Error> where Self: Sized;
  fn leave(conn: &PgConnection, form: T) -> usize;
}


pub fn establish_connection() -> PgConnection {
  dotenv().ok();

  let database_url = env::var("DATABASE_URL")
    .expect("DATABASE_URL must be set");
  PgConnection::establish(&database_url)
    .expect(&format!("Error connecting to {}", database_url))
}

