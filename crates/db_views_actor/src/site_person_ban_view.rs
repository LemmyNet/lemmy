use crate::structs::SitePersonBanView;
use diesel::{dsl::exists, result::Error, select, ExpressionMethods, QueryDsl};
use diesel_async::RunQueryDsl;
use lemmy_db_schema::{
  newtypes::{PersonId, SiteId},
  schema::site_person_ban,
  utils::{get_conn, DbPool},
};

impl SitePersonBanView {
  pub async fn get(
    pool: &mut DbPool<'_>,
    from_person_id: PersonId,
    from_site_id: SiteId,
  ) -> Result<bool, Error> {
    let conn = &mut get_conn(pool).await?;
    select(exists(
      site_person_ban::table
        .filter(site_person_ban::site_id.eq(from_site_id))
        .filter(site_person_ban::person_id.eq(from_person_id)),
    ))
    .get_result::<bool>(conn)
    .await
  }
}
