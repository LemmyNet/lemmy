use crate::{schema::secret::dsl::secret, source::secret::Secret, utils::GetConn};
use diesel::result::Error;
use lemmy_db_schema::utils::RunQueryDsl;

impl Secret {
  /// Initialize the Secrets from the DB.
  /// Warning: You should only call this once.
  pub async fn init(mut conn: impl GetConn) -> Result<Secret, Error> {
    Self::read_secrets(conn).await
  }

  async fn read_secrets(mut conn: impl GetConn) -> Result<Secret, Error> {
    secret.first::<Secret>(conn).await
  }
}
