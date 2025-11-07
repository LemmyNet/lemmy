use crate::source::secret::Secret;
use diesel_async::RunQueryDsl;
use lemmy_db_schema_file::schema::secret::dsl::secret;
use lemmy_diesel_utils::connection::{DbPool, get_conn};
use lemmy_utils::error::{LemmyErrorExt, LemmyErrorType, LemmyResult};

impl Secret {
  /// Initialize the Secrets from the DB.
  /// Warning: You should only call this once.
  pub async fn init(pool: &mut DbPool<'_>) -> LemmyResult<Secret> {
    Self::read_secrets(pool).await
  }

  async fn read_secrets(pool: &mut DbPool<'_>) -> LemmyResult<Self> {
    let conn = &mut get_conn(pool).await?;
    secret
      .first(conn)
      .await
      .with_lemmy_type(LemmyErrorType::NotFound)
  }
}
