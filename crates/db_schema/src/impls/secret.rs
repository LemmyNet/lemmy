use crate::{schema::secret::dsl::secret, source::secret::Secret, utils::DbConn};
use diesel::result::Error;
use diesel_async::RunQueryDsl;

impl Secret {
  /// Initialize the Secrets from the DB.
  /// Warning: You should only call this once.
  pub async fn init(conn: &mut DbConn) -> Result<Secret, Error> {
    Self::read_secrets(conn).await
  }

  async fn read_secrets(conn: &mut DbConn) -> Result<Secret, Error> {
    secret.first::<Secret>(conn).await
  }
}
