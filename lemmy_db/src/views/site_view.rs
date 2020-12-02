use crate::{
  schema::{site as site_table, user_},
  site::Site,
  user::{UserSafe, User_},
};
use diesel::{result::Error, *};
use serde::Serialize;

#[derive(Debug, Serialize, Clone)]
pub struct SiteView {
  pub site: Site,
  pub creator: UserSafe,
}

impl SiteView {
  pub fn read(conn: &PgConnection) -> Result<Self, Error> {
    let site_join = site_table::table
      .inner_join(user_::table)
      .first::<(Site, User_)>(conn)?;

    Ok(SiteView {
      site: site_join.0,
      creator: site_join.1.to_safe(),
    })
  }
}
