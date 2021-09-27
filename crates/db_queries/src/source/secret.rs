use diesel::{result::Error, *};
use lemmy_db_schema::source::secret::Secret;

pub trait Secret_ {
  fn init(conn: &PgConnection) -> Result<Secret, Error>;
}

impl Secret_ for Secret {
  /// Initialize the Secrets from the DB.
  /// Warning: You should only call this once.
  fn init(conn: &PgConnection) -> Result<Secret, Error> {
    read_secrets(conn)
  }
}

fn read_secrets(conn: &PgConnection) -> Result<Secret, Error> {
  use lemmy_db_schema::schema::secret::dsl::*;
  secret.first::<Secret>(conn)
}
