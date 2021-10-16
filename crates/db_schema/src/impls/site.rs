use crate::{naive_now, newtypes::PersonId, source::site::*, traits::Crud};
use diesel::{dsl::*, result::Error, *};

impl Crud for Site {
  type Form = SiteForm;
  type IdType = i32;
  fn read(conn: &PgConnection, _site_id: i32) -> Result<Self, Error> {
    use crate::schema::site::dsl::*;
    site.first::<Self>(conn)
  }

  fn create(conn: &PgConnection, new_site: &SiteForm) -> Result<Self, Error> {
    use crate::schema::site::dsl::*;
    insert_into(site).values(new_site).get_result::<Self>(conn)
  }

  fn update(conn: &PgConnection, site_id: i32, new_site: &SiteForm) -> Result<Self, Error> {
    use crate::schema::site::dsl::*;
    diesel::update(site.find(site_id))
      .set(new_site)
      .get_result::<Self>(conn)
  }
  fn delete(conn: &PgConnection, site_id: i32) -> Result<usize, Error> {
    use crate::schema::site::dsl::*;
    diesel::delete(site.find(site_id)).execute(conn)
  }
}

impl Site {
  pub fn transfer(conn: &PgConnection, new_creator_id: PersonId) -> Result<Site, Error> {
    use crate::schema::site::dsl::*;
    diesel::update(site.find(1))
      .set((creator_id.eq(new_creator_id), updated.eq(naive_now())))
      .get_result::<Self>(conn)
  }

  pub fn read_simple(conn: &PgConnection) -> Result<Self, Error> {
    use crate::schema::site::dsl::*;
    site.first::<Self>(conn)
  }
}
