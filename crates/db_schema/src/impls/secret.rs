use crate::{
  schema::secret::dsl::secret,
  source::secret::Secret,
  utils::{DbPool, DbPoolRef, RunQueryDsl},
};
use diesel::result::Error;

impl Secret {
  /// Initialize the Secrets from the DB.
  /// Warning: You should only call this once.
  pub async fn init(pool: DbPoolRef<'_>) -> Result<Secret, Error> {
    Self::read_secrets(pool).await
  }

  async fn read_secrets(pool: DbPoolRef<'_>) -> Result<Secret, Error> {
    let conn = pool;
    secret.first::<Secret>(conn).await
  }
}
