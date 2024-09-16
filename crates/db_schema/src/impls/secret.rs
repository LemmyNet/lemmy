use crate::{
  diesel::OptionalExtension,
  schema::secret::dsl::secret,
  source::secret::Secret,
  utils::{get_conn, DbPool},
};
use diesel::result::Error;
use diesel_async::RunQueryDsl;

impl Secret {
  /// Initialize the Secrets from the DB.
  /// Warning: You should only call this once.
  pub async fn init(pool: &mut DbPool<'_>) -> Result<Option<Secret>, Error> {
    Self::read_secrets(pool).await
  }

  async fn read_secrets(pool: &mut DbPool<'_>) -> Result<Option<Self>, Error> {
    let conn = &mut get_conn(pool).await?;
    secret.first(conn).await.optional()
  }
}
