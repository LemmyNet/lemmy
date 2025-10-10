use crate::{
  newtypes::{DbUrl, InstanceId, SiteId},
  source::{
    actor_language::SiteLanguage,
    site::{Site, SiteInsertForm, SiteUpdateForm},
  },
  traits::Crud,
  utils::{functions::lower, get_conn, DbPool},
};
use diesel::{dsl::insert_into, ExpressionMethods, OptionalExtension, QueryDsl};
use diesel_async::RunQueryDsl;
use lemmy_db_schema_file::schema::{local_site, site};
use lemmy_utils::error::{LemmyErrorExt, LemmyErrorType, LemmyResult};
use url::Url;

impl Crud for Site {
  type InsertForm = SiteInsertForm;
  type UpdateForm = SiteUpdateForm;
  type IdType = SiteId;

  /// Use SiteView::read_local, or Site::read_from_apub_id instead
  async fn read(_pool: &mut DbPool<'_>, _site_id: SiteId) -> LemmyResult<Self> {
    Err(LemmyErrorType::NotFound.into())
  }

  async fn create(pool: &mut DbPool<'_>, form: &Self::InsertForm) -> LemmyResult<Self> {
    let is_new_site = match &form.ap_id {
      Some(id) => Site::read_from_apub_id(pool, id).await?.is_none(),
      None => true,
    };
    let conn = &mut get_conn(pool).await?;

    // Can't do separate insert/update commands because InsertForm/UpdateForm aren't convertible
    let site = insert_into(site::table)
      .values(form)
      .on_conflict(site::ap_id)
      .do_update()
      .set(form)
      .get_result::<Self>(conn)
      .await?;

    // initialize languages if site is newly created
    if is_new_site {
      // initialize with all languages
      SiteLanguage::update(pool, vec![], &site).await?;
    }
    Ok(site)
  }

  async fn update(
    pool: &mut DbPool<'_>,
    site_id: SiteId,
    new_site: &Self::UpdateForm,
  ) -> LemmyResult<Self> {
    let conn = &mut get_conn(pool).await?;
    diesel::update(site::table.find(site_id))
      .set(new_site)
      .get_result::<Self>(conn)
      .await
      .with_lemmy_type(LemmyErrorType::CouldntUpdate)
  }
}

impl Site {
  pub async fn read_from_instance_id(
    pool: &mut DbPool<'_>,
    instance_id: InstanceId,
  ) -> LemmyResult<Self> {
    let conn = &mut get_conn(pool).await?;
    site::table
      .filter(site::instance_id.eq(instance_id))
      .first(conn)
      .await
      .with_lemmy_type(LemmyErrorType::NotFound)
  }
  pub async fn read_from_apub_id(
    pool: &mut DbPool<'_>,
    object_id: &DbUrl,
  ) -> LemmyResult<Option<Self>> {
    let conn = &mut get_conn(pool).await?;

    site::table
      .filter(lower(site::ap_id).eq(object_id.to_lowercase()))
      .first(conn)
      .await
      .optional()
      .with_lemmy_type(LemmyErrorType::NotFound)
  }

  pub async fn read_remote_sites(pool: &mut DbPool<'_>) -> LemmyResult<Vec<Self>> {
    let conn = &mut get_conn(pool).await?;
    site::table
      .order_by(site::id)
      .offset(1)
      .get_results::<Self>(conn)
      .await
      .with_lemmy_type(LemmyErrorType::NotFound)
  }

  /// Instance actor is at the root path, so we simply need to clear the path and other unnecessary
  /// parts of the url.
  pub fn instance_ap_id_from_url(mut url: Url) -> Url {
    url.set_fragment(None);
    url.set_path("");
    url.set_query(None);
    url
  }

  pub async fn read_local(pool: &mut DbPool<'_>) -> LemmyResult<Self> {
    let conn = &mut get_conn(pool).await?;

    site::table
      .inner_join(local_site::table)
      .select(site::all_columns)
      .first(conn)
      .await
      .with_lemmy_type(LemmyErrorType::LocalSiteNotSetup)
  }
}
