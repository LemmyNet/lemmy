use crate::{
  newtypes::{LocalSiteId, TaglineId},
  schema::tagline::dsl::{local_site_id, published, tagline},
  source::tagline::{Tagline, TaglineInsertForm, TaglineUpdateForm},
  traits::Crud,
  utils::{get_conn, limit_and_offset, DbPool},
};
use diesel::{insert_into, result::Error, ExpressionMethods, OptionalExtension, QueryDsl};
use diesel_async::RunQueryDsl;

#[async_trait]
impl Crud for Tagline {
  type InsertForm = TaglineInsertForm;
  type UpdateForm = TaglineUpdateForm;
  type IdType = TaglineId;

  async fn create(pool: &mut DbPool<'_>, form: &Self::InsertForm) -> Result<Self, Error> {
    let conn = &mut get_conn(pool).await?;
    insert_into(tagline)
      .values(form)
      .get_result::<Self>(conn)
      .await
  }

  async fn update(
    pool: &mut DbPool<'_>,
    tagline_id: TaglineId,
    new_tagline: &Self::UpdateForm,
  ) -> Result<Self, Error> {
    let conn = &mut get_conn(pool).await?;
    diesel::update(tagline.find(tagline_id))
      .set(new_tagline)
      .get_result::<Self>(conn)
      .await
  }

  async fn delete(pool: &mut DbPool<'_>, id: Self::IdType) -> Result<usize, Error> {
    let conn = &mut get_conn(pool).await?;
    diesel::delete(tagline.find(id)).execute(conn).await
  }
}

impl Tagline {
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
