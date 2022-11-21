use crate::{
  newtypes::LocalSiteId,
  schema::tagline::dsl::{local_site_id, tagline},
  source::tagline::{Tagline, TaglineForm},
  utils::{get_conn, DbPool},
};
use diesel::{insert_into, result::Error, ExpressionMethods, QueryDsl};
use diesel_async::{AsyncPgConnection, RunQueryDsl};

impl Tagline {
  pub async fn replace(
    pool: &DbPool,
    for_local_site_id: LocalSiteId,
    list_content: Option<Vec<String>>,
  ) -> Result<(), Error> {
    if let Some(list) = list_content {
      let conn = &mut get_conn(pool).await?;
      conn
        .build_transaction()
        .run(|conn| {
          Box::pin(async move {
            Self::clear(conn).await?;

            for item in list {
              let form = TaglineForm {
                local_site_id: for_local_site_id,
                content: item,
                updated: None,
              };
              insert_into(tagline)
                .values(form)
                .get_result::<Self>(conn)
                .await?;
            }
            Ok(())
          }) as _
        })
        .await
    } else {
      Ok(())
    }
  }

  async fn clear(conn: &mut AsyncPgConnection) -> Result<usize, Error> {
    diesel::delete(tagline).execute(conn).await
  }
  pub async fn get_all(pool: &DbPool, for_local_site_id: LocalSiteId) -> Result<Vec<Self>, Error> {
    let conn = &mut get_conn(pool).await?;
    tagline
      .filter(local_site_id.eq(for_local_site_id))
      .get_results::<Self>(conn)
      .await
  }
}
