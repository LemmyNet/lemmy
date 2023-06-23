use crate::{
  newtypes::SiteRoleId,
  schema::site_role::dsl::site_role,
  source::site_role::SiteRole,
  traits::Crud,
  utils::{get_conn, DbPool},
};
use diesel::{result::Error, QueryDsl};
use diesel_async::RunQueryDsl;

#[async_trait]
impl Crud for SiteRole {
  type InsertForm = SiteRole;
  type UpdateForm = SiteRole;
  type IdType = SiteRoleId;

  async fn read(pool: &DbPool, site_role_id: SiteRoleId) -> Result<Self, Error> {
    let conn = &mut get_conn(pool).await?;
    site_role.find(site_role_id).first::<Self>(conn).await
  }

  async fn delete(pool: &DbPool, site_role_id: SiteRoleId) -> Result<usize, Error> {
    let conn = &mut get_conn(pool).await?;
    diesel::delete(site_role.find(site_role_id))
      .execute(conn)
      .await
  }

  async fn create(pool: &DbPool, form: &Self::InsertForm) -> Result<Self, Error> {
    let conn = &mut get_conn(pool).await?;

    diesel::insert_into(site_role)
      .values(form)
      .get_result::<Self>(conn)
      .await
  }

  async fn update(
    pool: &DbPool,
    site_role_id: SiteRoleId,
    form: &Self::UpdateForm,
  ) -> Result<Self, Error> {
    let conn = &mut get_conn(pool).await?;

    diesel::update(site_role.find(site_role_id))
      .set(form)
      .get_result::<Self>(conn)
      .await
  }
}

impl SiteRole {
  pub async fn read_all(pool: &DbPool) -> Result<Vec<Self>, Error> {
    let conn = &mut get_conn(pool).await?;
    site_role
      .get_results::<Self>(conn)
      .await
      .map(|results| results.into_iter().collect::<Vec<Self>>())
  }
}
