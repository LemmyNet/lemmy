use crate::structs::TaglineView;
use diesel::{result::Error, ExpressionMethods, QueryDsl};
use diesel_async::RunQueryDsl;
use lemmy_db_schema::{
  newtypes::LocalSiteId,
  schema::tagline,
  source::tagline::Tagline,
  utils::{get_conn, limit_and_offset, DbPool},
};

impl TaglineView {
  pub async fn list(
    pool: &mut DbPool<'_>,
    for_local_site_id: LocalSiteId,
    page: Option<i64>,
    limit: Option<i64>,
  ) -> Result<Vec<Self>, Error> {
    let conn = &mut get_conn(pool).await?;
    let (limit, offset) = limit_and_offset(page, limit)?;
    let taglines = tagline::table
      .filter(tagline::local_site_id.eq(for_local_site_id))
      .order(tagline::id)
      .select(tagline::all_columns)
      .limit(limit)
      .offset(offset)
      .load::<Tagline>(conn)
      .await?;

    let mut result = Vec::new();
    for tagline in &taglines {
      result.push(TaglineView {
        tagline: tagline.clone(),
      });
    }

    Ok(result)
  }
}
