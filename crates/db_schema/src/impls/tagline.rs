use crate::{
  newtypes::{PaginationCursor, TaglineId},
  source::tagline::{tagline_keys as key, Tagline, TaglineInsertForm, TaglineUpdateForm},
  traits::Crud,
  utils::{functions::random, get_conn, limit_fetch, paginate, DbPool},
};
use diesel::{insert_into, QueryDsl};
use diesel_async::RunQueryDsl;
use i_love_jesus::SortDirection;
use lemmy_db_schema_file::schema::tagline;
use lemmy_utils::error::{LemmyErrorExt, LemmyErrorType, LemmyResult};

impl Crud for Tagline {
  type InsertForm = TaglineInsertForm;
  type UpdateForm = TaglineUpdateForm;
  type IdType = TaglineId;

  async fn create(pool: &mut DbPool<'_>, form: &Self::InsertForm) -> LemmyResult<Self> {
    let conn = &mut get_conn(pool).await?;
    insert_into(tagline::table)
      .values(form)
      .get_result::<Self>(conn)
      .await
      .with_lemmy_type(LemmyErrorType::CouldntCreate)
  }

  async fn update(
    pool: &mut DbPool<'_>,
    tagline_id: TaglineId,
    form: &Self::UpdateForm,
  ) -> LemmyResult<Self> {
    let conn = &mut get_conn(pool).await?;
    diesel::update(tagline::table.find(tagline_id))
      .set(form)
      .get_result::<Self>(conn)
      .await
      .with_lemmy_type(LemmyErrorType::CouldntUpdate)
  }
}

impl Tagline {
  pub async fn list(
    pool: &mut DbPool<'_>,
    cursor_data: Option<Tagline>,
    page_back: Option<bool>,
    limit: Option<i64>,
  ) -> LemmyResult<Vec<Self>> {
    let conn = &mut get_conn(pool).await?;
    let limit = limit_fetch(limit)?;
    let query = tagline::table.limit(limit).into_boxed();
    let paginated_query = paginate(query, SortDirection::Desc, cursor_data, None, page_back)
      .then_order_by(key::published_at)
      .then_order_by(key::id);

    paginated_query
      .load::<Self>(conn)
      .await
      .with_lemmy_type(LemmyErrorType::NotFound)
  }

  pub async fn get_random(pool: &mut DbPool<'_>) -> LemmyResult<Self> {
    let conn = &mut get_conn(pool).await?;
    tagline::table
      .order(random())
      .limit(1)
      .first::<Self>(conn)
      .await
      .with_lemmy_type(LemmyErrorType::NotFound)
  }

  pub fn to_cursor(&self) -> PaginationCursor {
    PaginationCursor::new_single('T', self.id.0)
  }

  pub async fn from_cursor(cursor: &PaginationCursor, pool: &mut DbPool<'_>) -> LemmyResult<Self> {
    let [(_, id)] = cursor.prefixes_and_ids()?;
    Self::read(pool, TaglineId(id)).await
  }
}
