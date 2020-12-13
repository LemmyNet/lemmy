use crate::{
  schema::{site, user_},
  source::{
    site::Site,
    user::{UserSafe, User_},
  },
  ToSafe,
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
    let (site, creator) = site::table
      .inner_join(user_::table)
      .select((site::all_columns, User_::safe_columns_tuple()))
      .first::<(Site, UserSafe)>(conn)?;

    Ok(SiteView { site, creator })
  }
}
