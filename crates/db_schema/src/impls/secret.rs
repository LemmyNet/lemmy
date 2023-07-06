use crate::{
  schema::secret::dsl::secret,
  source::secret::Secret,
  utils::{get_conn, DbPool},
};
use diesel::result::Error;
use diesel_async::RunQueryDsl;

impl Secret {
  /// Initialize the Secrets from the DB.
  /// Warning: You should only call this once.
  pub async fn init(pool: &mut DbPool<'_>) -> Result<Secret, Error> {
    Self::read_secrets(pool).await
  }

  async fn read_secrets(pool: &mut DbPool<'_>) -> Result<Secret, Error> {
    let conn = &mut get_conn(pool).await?;
    secret.first::<Secret>(conn).await
  }
}
