use crate::{
  newtypes::{LocalSiteId, TaglineId},
  schema::tagline::dsl::{local_site_id, published, tagline},
  source::tagline::{Tagline, TaglineInsertForm, TaglineUpdateForm},
  utils::{get_conn, limit_and_offset, DbPool},
};
use diesel::{insert_into, result::Error, ExpressionMethods, OptionalExtension, QueryDsl};
use diesel_async::RunQueryDsl;

impl Tagline {
  pub async fn create(pool: &mut DbPool<'_>, form: &TaglineInsertForm) -> Result<Self, Error> {
    let conn = &mut get_conn(pool).await?;
    insert_into(tagline)
      .values(form)
      .get_result::<Self>(conn)
      .await
  }

  pub async fn update(
    pool: &mut DbPool<'_>,
    tagline_id: TaglineId,
    form: &TaglineUpdateForm,
  ) -> Result<Self, Error> {
    let conn = &mut get_conn(pool).await?;
    diesel::update(tagline.find(tagline_id))
      .set(form)
      .get_result::<Self>(conn)
      .await
  }

  pub async fn delete(pool: &mut DbPool<'_>, tagline_id: TaglineId) -> Result<usize, Error> {
    let conn = &mut get_conn(pool).await?;
    diesel::delete(tagline.find(tagline_id)).execute(conn).await
  }

  pub async fn get_all(
    pool: &mut DbPool<'_>,
    for_local_site_id: LocalSiteId,
  ) -> Result<Vec<Self>, Error> {
    let conn = &mut get_conn(pool).await?;
    tagline
      .filter(local_site_id.eq(for_local_site_id))
      .get_results::<Self>(conn)
      .await
  }

  pub async fn list(
    pool: &mut DbPool<'_>,
    for_local_site_id: LocalSiteId,
    page: Option<i64>,
    limit: Option<i64>,
  ) -> Result<Vec<Self>, Error> {
    let conn = &mut get_conn(pool).await?;
    let (limit, offset) = limit_and_offset(page, limit)?;
    tagline
      .order(published.desc())
      .offset(offset)
      .limit(limit)
      .filter(local_site_id.eq(for_local_site_id))
      .get_results::<Self>(conn)
      .await
  }

  pub async fn get_random(
    pool: &mut DbPool<'_>,
    for_local_site_id: LocalSiteId,
  ) -> Result<Option<Self>, Error> {
    let conn = &mut get_conn(pool).await?;
    sql_function!(fn random() -> Text);
    tagline
      .order(random())
      .limit(1)
      .filter(local_site_id.eq(for_local_site_id))
      .first::<Self>(conn)
      .await
      .optional()
  }
}
