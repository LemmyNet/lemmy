use crate::{
  source::secret::Secret,
  utils::{get_conn, DbPool},
};
use diesel::result::Error;
use diesel_async::RunQueryDsl;
use lemmy_db_schema_file::schema::secret::dsl::secret;

impl Secret {
  /// Initialize the Secrets from the DB.
  /// Warning: You should only call this once.
  pub async fn init(pool: &mut DbPool<'_>) -> Result<Secret, Error> {
    Self::read_secrets(pool).await
  }

  async fn read_secrets(pool: &mut DbPool<'_>) -> Result<Self, Error> {
    let conn = &mut get_conn(pool).await?;
    secret.first(conn).await
  }
}
