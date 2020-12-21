use crate::ToSafe;
use diesel::{result::Error, *};
use lemmy_db_schema::{
  schema::{site, user_},
  source::{
    site::Site,
    user::{UserSafe, User_},
  },
};
use serde::Serialize;

#[derive(Debug, Serialize, Clone)]
pub struct SiteView {
  pub site: Site,
  pub creator: UserSafe,
}

impl SiteView {
  pub fn read(conn: &PgConnection) -> Result<Self, Error> {
    let (site, creator) = site::table
      .inner_join(user_::table)
      .select((site::all_columns, User_::safe_columns_tuple()))
      .first::<(Site, UserSafe)>(conn)?;

    Ok(SiteView { site, creator })
  }
}
