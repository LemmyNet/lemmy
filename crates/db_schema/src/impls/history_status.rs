use crate::{
  source::history_status::{HistoryStatus, HistoryStatusInsertForm, HistoryStatusUpdateForm},
  traits::Crud,
  utils::{get_conn, DbPool},
};
use diesel::{insert_into, QueryDsl};
use diesel_async::RunQueryDsl;
use lemmy_db_schema_file::schema::history_status;
use lemmy_utils::error::{LemmyErrorExt, LemmyErrorType, LemmyResult};

impl Crud for HistoryStatus {
  type InsertForm = HistoryStatusInsertForm;
  type UpdateForm = HistoryStatusUpdateForm;
  type IdType = (String, String);

  async fn create(pool: &mut DbPool<'_>, form: &Self::InsertForm) -> LemmyResult<Self> {
    let conn = &mut get_conn(pool).await?;
    insert_into(history_status::table)
      .values(form)
      .get_result::<Self>(conn)
      .await
      .with_lemmy_type(LemmyErrorType::NotFound)
  }

  async fn update(
    pool: &mut DbPool<'_>,
    id: Self::IdType,
    form: &Self::UpdateForm,
  ) -> LemmyResult<Self> {
    let conn = &mut get_conn(pool).await?;
    diesel::update(history_status::table.find((id.0, id.1)))
      .set(form)
      .get_result::<Self>(conn)
      .await
      .with_lemmy_type(LemmyErrorType::NotFound)
  }
}
