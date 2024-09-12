use crate::{
  newtypes::{DbUrl, InstanceId, SiteId},
  schema::site,
  source::{
    actor_language::SiteLanguage,
    site::{Site, SiteInsertForm, SiteUpdateForm},
  },
  traits::Crud,
  utils::{get_conn, DbPool},
};
use diesel::{dsl::insert_into, result::Error, ExpressionMethods, OptionalExtension, QueryDsl};
use diesel_async::RunQueryDsl;
use lemmy_utils::{error::LemmyResult, LemmyErrorType};
use url::Url;

#[async_trait]
impl Crud for Site {
  type InsertForm = SiteInsertForm;
  type UpdateForm = SiteUpdateForm;
  type IdType = SiteId;

  /// Use SiteView::read_local, or Site::read_from_apub_id instead
  async fn read(_pool: &mut DbPool<'_>, _site_id: SiteId) -> Result<Option<Self>, Error> {
    Err(Error::NotFound)
  }

  async fn create(pool: &mut DbPool<'_>, form: &Self::InsertForm) -> Result<Self, Error> {
    let is_new_site = match &form.actor_id {
      Some(id_) => Site::read_from_apub_id(pool, id_).await?.is_none(),
      None => true,
    };
    let conn = &mut get_conn(pool).await?;

    // Can't do separate insert/update commands because InsertForm/UpdateForm aren't convertible
    let site_ = insert_into(site::table)
      .values(form)
      .on_conflict(site::actor_id)
      .do_update()
      .set(form)
      .get_result::<Self>(conn)
      .await?;

    // initialize languages if site is newly created
    if is_new_site {
      // initialize with all languages
      SiteLanguage::update(pool, vec![], &site_).await?;
    }
    Ok(site_)
  }

  async fn update(
    pool: &mut DbPool<'_>,
    site_id: SiteId,
    new_site: &Self::UpdateForm,
  ) -> Result<Self, Error> {
    let conn = &mut get_conn(pool).await?;
    diesel::update(site::table.find(site_id))
      .set(new_site)
      .get_result::<Self>(conn)
      .await
  }
}

impl Site {
  pub async fn read_from_instance_id(
    pool: &mut DbPool<'_>,
    _instance_id: InstanceId,
  ) -> Result<Option<Self>, Error> {
    let conn = &mut get_conn(pool).await?;
    site::table
      .filter(site::instance_id.eq(_instance_id))
      .first(conn)
      .await
      .optional()
  }
  pub async fn read_from_apub_id(
    pool: &mut DbPool<'_>,
    object_id: &DbUrl,
  ) -> Result<Option<Self>, Error> {
    let conn = &mut get_conn(pool).await?;

    site::table
      .filter(site::actor_id.eq(object_id))
      .first(conn)
      .await
      .optional()
  }

  pub async fn read_remote_sites(pool: &mut DbPool<'_>) -> Result<Vec<Self>, Error> {
    let conn = &mut get_conn(pool).await?;
    site::table
      .order_by(site::id)
      .offset(1)
      .get_results::<Self>(conn)
      .await
  }

  /// Instance actor is at the root path, so we simply need to clear the path and other unnecessary
  /// parts of the url.
  pub fn instance_actor_id_from_url(mut url: Url) -> Url {
    url.set_fragment(None);
    url.set_path("");
    url.set_query(None);
    url
  }

  pub async fn read_local(pool: &mut DbPool<'_>) -> LemmyResult<Self> {
    let conn = &mut get_conn(pool).await?;

    Ok(
      site::table
        .filter(site::private_key.is_not_null())
        .first(conn)
        .await
        .optional()?
        .ok_or(LemmyErrorType::LocalSiteNotSetup)?,
    )
  }
}
