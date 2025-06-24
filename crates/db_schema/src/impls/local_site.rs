use crate::{
  source::local_site::{LocalSite, LocalSiteInsertForm, LocalSiteUpdateForm},
  utils::{get_conn, DbPool},
};
use diesel::dsl::insert_into;
use diesel_async::RunQueryDsl;
use lemmy_db_schema_file::schema::local_site;
use lemmy_utils::error::{LemmyErrorExt, LemmyErrorType, LemmyResult};

impl LocalSite {
  pub async fn create(pool: &mut DbPool<'_>, form: &LocalSiteInsertForm) -> LemmyResult<Self> {
    let conn = &mut get_conn(pool).await?;
    insert_into(local_site::table)
      .values(form)
      .get_result::<Self>(conn)
      .await
      .with_lemmy_type(LemmyErrorType::CouldntCreateSite)
  }

  pub async fn update(pool: &mut DbPool<'_>, form: &LocalSiteUpdateForm) -> LemmyResult<Self> {
    let conn = &mut get_conn(pool).await?;
    diesel::update(local_site::table)
      .set(form)
      .get_result::<Self>(conn)
      .await
      .with_lemmy_type(LemmyErrorType::CouldntUpdateSite)
  }

  pub async fn delete(pool: &mut DbPool<'_>) -> LemmyResult<usize> {
    let conn = &mut get_conn(pool).await?;
    diesel::delete(local_site::table)
      .execute(conn)
      .await
      .with_lemmy_type(LemmyErrorType::Deleted)
  }
}
