use crate::{
  schema::secret::dsl::secret,
  source::secret::Secret,
  utils::{DbPool, GetConn},
};
use diesel::result::Error;
use diesel_async::RunQueryDsl;

impl Secret {
  /// Initialize the Secrets from the DB.
  /// Warning: You should only call this once.
  pub async fn init(mut pool: &mut impl GetConn) -> Result<Secret, Error> {
    Self::read_secrets(pool).await
  }

  async fn read_secrets(mut pool: &mut impl GetConn) -> Result<Secret, Error> {
    let conn = &mut *pool.get_conn().await?;
    secret.first::<Secret>(conn).await
  }
}
