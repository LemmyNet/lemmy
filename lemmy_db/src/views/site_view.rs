use crate::{
  schema::{site, user_},
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
    let (site, creator) = site::table
      .inner_join(user_::table)
      .first::<(Site, User_)>(conn)?;

    Ok(SiteView {
      site,
      creator: creator.to_safe(),
    })
  }
}
