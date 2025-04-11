use crate::{
  source::secret::Secret,
  utils::{get_conn, DbPool},
};
use diesel_async::RunQueryDsl;
use lemmy_db_schema_file::schema::secret::dsl::secret;
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
