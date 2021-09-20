use diesel::{result::Error, *};
use lemmy_db_schema::source::secrets::Secrets;

pub trait Secrets_ {
  fn read(conn: &PgConnection) -> Result<String, Error>;
}

impl Secrets_ for Secrets {
  fn read(conn: &PgConnection) -> Result<String, Error> {
    use lemmy_db_schema::schema::secrets::dsl::*;
    secrets.first::<Self>(conn).map(|s| s.jwt_secret)
  }
}
