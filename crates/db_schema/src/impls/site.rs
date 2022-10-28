use crate::{
  newtypes::{DbUrl, SiteId},
  source::{actor_language::SiteLanguage, site::*},
  traits::Crud,
};
use diesel::{dsl::*, result::Error, *};
use url::Url;

impl Crud for Site {
  type InsertForm = SiteInsertForm;
  type UpdateForm = SiteUpdateForm;
  type IdType = SiteId;
  fn read(conn: &mut PgConnection, _site_id: SiteId) -> Result<Self, Error> {
    use crate::schema::site::dsl::*;
    site.first::<Self>(conn)
  }

  fn create(conn: &mut PgConnection, form: &Self::InsertForm) -> Result<Self, Error> {
    use crate::schema::site::dsl::*;

    let site_ = insert_into(site)
      .values(form)
      .on_conflict(actor_id)
      .do_update()
      .set(form)
      .get_result::<Self>(conn)?;

    // initialize with all languages
    SiteLanguage::update(conn, vec![], &site_)?;
    Ok(site_)
  }

  fn update(
    conn: &mut PgConnection,
    site_id: SiteId,
    new_site: &Self::UpdateForm,
  ) -> Result<Self, Error> {
    use crate::schema::site::dsl::*;
    diesel::update(site.find(site_id))
      .set(new_site)
      .get_result::<Self>(conn)
  }

  fn delete(conn: &mut PgConnection, site_id: SiteId) -> Result<usize, Error> {
    use crate::schema::site::dsl::*;
    diesel::delete(site.find(site_id)).execute(conn)
  }
}

impl Site {
  pub fn read_from_apub_id(conn: &mut PgConnection, object_id: Url) -> Result<Option<Self>, Error> {
    use crate::schema::site::dsl::*;
    let object_id: DbUrl = object_id.into();
    Ok(
      site
        .filter(actor_id.eq(object_id))
        .first::<Site>(conn)
        .ok()
        .map(Into::into),
    )
  }

  // TODO this needs fixed
  pub fn read_remote_sites(conn: &mut PgConnection) -> Result<Vec<Self>, Error> {
    use crate::schema::site::dsl::*;
    site.order_by(id).offset(1).get_results::<Self>(conn)
  }
}
