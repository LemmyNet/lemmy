use crate::{
  newtypes::{LocalSiteId, TaglineId},
  schema::tagline::dsl::{local_site_id, published, tagline},
  source::tagline::{Tagline, TaglineInsertForm, TaglineUpdateForm},
  utils::{get_conn, limit_and_offset, DbPool},
};
use diesel::{insert_into, result::Error, ExpressionMethods, QueryDsl};
use diesel_async::{AsyncPgConnection, RunQueryDsl};

impl Tagline {
  pub async fn replace(
    pool: &mut DbPool<'_>,
    for_local_site_id: LocalSiteId,
    list_content: Option<Vec<String>>,
  ) -> Result<Vec<Self>, Error> {
    let conn = &mut get_conn(pool).await?;
    if let Some(list) = list_content {
      conn
        .build_transaction()
        .run(|conn| {
          Box::pin(async move {
            Self::clear(conn).await?;

            for item in list {
              let form = TaglineInsertForm {
                local_site_id: for_local_site_id,
                content: item,
                updated: None,
              };
              insert_into(tagline)
                .values(form)
                .get_result::<Self>(conn)
                .await?;
            }
            Self::get_all(&mut conn.into(), for_local_site_id).await
          }) as _
        })
        .await
    } else {
      Self::get_all(&mut conn.into(), for_local_site_id).await
    }
  }

  pub async fn create(
    pool: &mut DbPool<'_>,
    form: &TaglineInsertForm,
  ) -> Result<Self, Error> {
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

  async fn clear(conn: &mut AsyncPgConnection) -> Result<usize, Error> {
    diesel::delete(tagline).execute(conn).await
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
}
