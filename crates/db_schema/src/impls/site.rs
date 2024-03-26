use crate::{
  newtypes::{DbUrl, InstanceId, SiteId},
  schema::site::dsl::{actor_id, id, instance_id, site},
  source::{
    actor_language::SiteLanguage,
    site::{Site, SiteInsertForm, SitePersonBan, SitePersonBanForm, SiteUpdateForm},
  },
  traits::{Bannable, Crud},
  utils::{get_conn, DbPool},
};
use diesel::{dsl::insert_into, result::Error, ExpressionMethods, OptionalExtension, QueryDsl};
use diesel_async::RunQueryDsl;
use url::Url;

#[async_trait]
impl Crud for Site {
  type InsertForm = SiteInsertForm;
  type UpdateForm = SiteUpdateForm;
  type IdType = SiteId;

  /// Use SiteView::read_local, or Site::read_from_apub_id instead
  async fn read(_pool: &mut DbPool<'_>, _site_id: SiteId) -> Result<Self, Error> {
    unimplemented!()
  }

  async fn create(pool: &mut DbPool<'_>, form: &Self::InsertForm) -> Result<Self, Error> {
    let is_new_site = match &form.actor_id {
      Some(id_) => Site::read_from_apub_id(pool, id_).await?.is_none(),
      None => true,
    };
    let conn = &mut get_conn(pool).await?;

    // Can't do separate insert/update commands because InsertForm/UpdateForm aren't convertible
    let site_ = insert_into(site)
      .values(form)
      .on_conflict(actor_id)
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
    diesel::update(site.find(site_id))
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
    site
      .filter(instance_id.eq(_instance_id))
      .get_result(conn)
      .await
      .optional()
  }
  pub async fn read_from_apub_id(
    pool: &mut DbPool<'_>,
    object_id: &DbUrl,
  ) -> Result<Option<Self>, Error> {
    let conn = &mut get_conn(pool).await?;

    site
      .filter(actor_id.eq(object_id))
      .first::<Site>(conn)
      .await
      .optional()
      .map(Into::into)
  }

  pub async fn read_remote_sites(pool: &mut DbPool<'_>) -> Result<Vec<Self>, Error> {
    let conn = &mut get_conn(pool).await?;
    site.order_by(id).offset(1).get_results::<Self>(conn).await
  }

  /// Instance actor is at the root path, so we simply need to clear the path and other unnecessary
  /// parts of the url.
  pub fn instance_actor_id_from_url(mut url: Url) -> Url {
    url.set_fragment(None);
    url.set_path("");
    url.set_query(None);
    url
  }
}

#[async_trait]
impl Bannable for SitePersonBan {
  type Form = SitePersonBanForm;
  async fn ban(
    pool: &mut DbPool<'_>,
    site_person_ban_form: &SitePersonBanForm,
  ) -> Result<Self, Error> {
    use crate::schema::site_person_ban::dsl::{person_id, site_id, site_person_ban};
    let conn = &mut get_conn(pool).await?;
    insert_into(site_person_ban)
      .values(site_person_ban_form)
      .on_conflict((site_id, person_id))
      .do_update()
      .set(site_person_ban_form)
      .get_result::<Self>(conn)
      .await
  }

  async fn unban(
    pool: &mut DbPool<'_>,
    site_person_ban_form: &SitePersonBanForm,
  ) -> Result<usize, Error> {
    use crate::schema::site_person_ban::dsl::site_person_ban;
    let conn = &mut get_conn(pool).await?;
    diesel::delete(
      site_person_ban.find((site_person_ban_form.person_id, site_person_ban_form.site_id)),
    )
    .execute(conn)
    .await
  }
}
