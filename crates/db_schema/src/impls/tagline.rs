use crate::{
  newtypes::LocalSiteId,
  schema::tagline::dsl::{local_site_id, tagline},
  source::tagline::{Tagline, TaglineForm},
  utils::DbConn,
};
use diesel::{insert_into, result::Error, ExpressionMethods, QueryDsl};
use diesel_async::RunQueryDsl;

impl Tagline {
  pub async fn replace(
    mut conn: impl DbConn,
    for_local_site_id: LocalSiteId,
    list_content: Option<Vec<String>>,
  ) -> Result<Vec<Self>, Error> {
    if let Some(list) = list_content {
      conn
        .build_transaction()
        .run(|conn| {
          Box::pin(async move {
            Self::clear(&mut *conn).await?;

            for item in list {
              let form = TaglineForm {
                local_site_id: for_local_site_id,
                content: item,
                updated: None,
              };
              insert_into(tagline)
                .values(form)
                .get_result::<Self>(&mut *conn)
                .await?;
            }
            Self::get_all_conn(&mut *conn, for_local_site_id).await
          }) as _
        })
        .await
    } else {
      Self::get_all_conn(&mut *conn, for_local_site_id).await
    }
  }

  async fn clear(mut conn: impl DbConn) -> Result<usize, Error> {
    diesel::delete(tagline).execute(&mut *conn).await
  }

  async fn get_all_conn(
    mut conn: impl DbConn,
    for_local_site_id: LocalSiteId,
  ) -> Result<Vec<Self>, Error> {
    tagline
      .filter(local_site_id.eq(for_local_site_id))
      .get_results::<Self>(&mut *conn)
      .await
  }
  pub async fn get_all(
    mut conn: impl DbConn,
    for_local_site_id: LocalSiteId,
  ) -> Result<Vec<Self>, Error> {
    Self::get_all_conn(&mut *conn, for_local_site_id).await
  }
}
