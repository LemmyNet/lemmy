use crate::{schema::local_site::dsl::*, source::local_site::*};
use diesel::{dsl::*, result::Error, *};

impl LocalSite {
  pub fn create(conn: &mut PgConnection, form: &LocalSiteInsertForm) -> Result<Self, Error> {
    insert_into(local_site)
      .values(form)
      .get_result::<Self>(conn)
  }
  pub fn read(conn: &mut PgConnection) -> Result<Self, Error> {
    local_site.first::<Self>(conn)
  }
  pub fn update(conn: &mut PgConnection, form: &LocalSiteUpdateForm) -> Result<Self, Error> {
    diesel::update(local_site)
      .set(form)
      .get_result::<Self>(conn)
  }
  pub fn delete(conn: &mut PgConnection) -> Result<usize, Error> {
    diesel::delete(local_site).execute(conn)
  }
}
