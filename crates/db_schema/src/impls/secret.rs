use crate::source::secret::Secret;
use diesel::{result::Error, *};

impl Secret {
  /// Initialize the Secrets from the DB.
  /// Warning: You should only call this once.
  pub fn init(conn: &PgConnection) -> Result<Secret, Error> {
    read_secrets(conn)
  }
}

fn read_secrets(conn: &PgConnection) -> Result<Secret, Error> {
  use crate::schema::secret::dsl::*;
  secret.first::<Secret>(conn)
}
