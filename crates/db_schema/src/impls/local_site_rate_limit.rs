use crate::{schema::local_site_rate_limit, source::local_site_rate_limit::*};
use diesel::{dsl::*, result::Error, *};

impl LocalSiteRateLimit {
  pub fn read(conn: &mut PgConnection) -> Result<Self, Error> {
    local_site_rate_limit::table.first::<Self>(conn)
  }

  pub fn create(
    conn: &mut PgConnection,
    form: &LocalSiteRateLimitInsertForm,
  ) -> Result<Self, Error> {
    insert_into(local_site_rate_limit::table)
      .values(form)
      .get_result::<Self>(conn)
  }
  pub fn update(
    conn: &mut PgConnection,
    form: &LocalSiteRateLimitUpdateForm,
  ) -> Result<Self, Error> {
    diesel::update(local_site_rate_limit::table)
      .set(form)
      .get_result::<Self>(conn)
  }
}
