use crate::{
  newtypes::{DbUrl, SiteId},
  schema::site::dsl::*,
  source::{actor_language::SiteLanguage, site::*},
  traits::Crud,
  utils::{get_conn, DbPool},
};
use diesel::{dsl::*, result::Error, ExpressionMethods, QueryDsl};
use diesel_async::RunQueryDsl;
use url::Url;

#[async_trait]
impl Crud for Site {
  type InsertForm = SiteInsertForm;
  type UpdateForm = SiteUpdateForm;
  type IdType = SiteId;

  async fn read(pool: &DbPool, _site_id: SiteId) -> Result<Self, Error> {
    let conn = &mut get_conn(&pool).await?;
    site.first::<Self>(conn).await
  }

  async fn create(pool: &DbPool, form: &Self::InsertForm) -> Result<Self, Error> {
    let conn = &mut get_conn(&pool).await?;
    let site_ = insert_into(site)
      .values(form)
      .on_conflict(actor_id)
      .do_update()
      .set(form)
      .get_result::<Self>(conn)
      .await?;

    // initialize with all languages
    SiteLanguage::update(pool, vec![], site_.id).await?;
    Ok(site_)
  }

  async fn update(
    pool: &DbPool,
    site_id: SiteId,
    new_site: &Self::UpdateForm,
  ) -> Result<Self, Error> {
    let conn = &mut get_conn(&pool).await?;
    diesel::update(site.find(site_id))
      .set(new_site)
      .get_result::<Self>(conn)
      .await
  }

  async fn delete(pool: &DbPool, site_id: SiteId) -> Result<usize, Error> {
    let conn = &mut get_conn(&pool).await?;
    diesel::delete(site.find(site_id)).execute(conn).await
  }
}

impl Site {
  pub async fn read_from_apub_id(pool: &DbPool, object_id: Url) -> Result<Option<Self>, Error> {
    let conn = &mut get_conn(&pool).await?;
    let object_id: DbUrl = object_id.into();
    Ok(
      site
        .filter(actor_id.eq(object_id))
        .first::<Site>(conn)
        .await
        .ok()
        .map(Into::into),
    )
  }

  // TODO this needs fixed
  pub async fn read_remote_sites(pool: &DbPool) -> Result<Vec<Self>, Error> {
    let conn = &mut get_conn(&pool).await?;
    site.order_by(id).offset(1).get_results::<Self>(conn).await
  }
}
