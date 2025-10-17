use crate::{
  source::modlog::{Modlog, ModlogInsertForm},
  utils::{get_conn, DbPool},
};
use diesel::dsl::insert_into;
use diesel_async::RunQueryDsl;
use lemmy_db_schema_file::schema::modlog;
use lemmy_utils::error::{LemmyErrorExt, LemmyErrorType, LemmyResult};

impl Modlog {
  pub async fn create(pool: &mut DbPool<'_>, form: &[ModlogInsertForm]) -> LemmyResult<Vec<Self>> {
    let conn = &mut get_conn(pool).await?;
    insert_into(modlog::table)
      .values(form)
      .get_results::<Self>(conn)
      .await
      .with_lemmy_type(LemmyErrorType::CouldntCreate)
  }
}
