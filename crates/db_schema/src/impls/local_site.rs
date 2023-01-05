use crate::{
  schema::local_site::dsl::local_site,
  source::local_site::{
    LocalSite,
    LocalSiteInsertForm,
    LocalSiteUpdateForm,
    RegistrationMode,
    RegistrationModeType,
  },
  utils::{get_conn, DbPool},
};
use diesel::{
  deserialize,
  deserialize::FromSql,
  dsl::insert_into,
  pg::{Pg, PgValue},
  result::Error,
  serialize,
  serialize::{IsNull, Output, ToSql},
};
use diesel_async::RunQueryDsl;
use std::io::Write;

impl LocalSite {
  pub async fn create(pool: &DbPool, form: &LocalSiteInsertForm) -> Result<Self, Error> {
    let conn = &mut get_conn(pool).await?;
    insert_into(local_site)
      .values(form)
      .get_result::<Self>(conn)
      .await
  }
  pub async fn read(pool: &DbPool) -> Result<Self, Error> {
    let conn = &mut get_conn(pool).await?;
    local_site.first::<Self>(conn).await
  }
  pub async fn update(pool: &DbPool, form: &LocalSiteUpdateForm) -> Result<Self, Error> {
    let conn = &mut get_conn(pool).await?;
    diesel::update(local_site)
      .set(form)
      .get_result::<Self>(conn)
      .await
  }
  pub async fn delete(pool: &DbPool) -> Result<usize, Error> {
    let conn = &mut get_conn(pool).await?;
    diesel::delete(local_site).execute(conn).await
  }
}

impl ToSql<RegistrationModeType, Pg> for RegistrationMode {
  fn to_sql<'b>(&'b self, out: &mut Output<'b, '_, Pg>) -> serialize::Result {
    match *self {
      RegistrationMode::Closed => out.write_all(b"closed")?,
      RegistrationMode::RequireApplication => out.write_all(b"require_application")?,
      RegistrationMode::Open => out.write_all(b"open")?,
    }
    Ok(IsNull::No)
  }
}

impl FromSql<RegistrationModeType, Pg> for RegistrationMode {
  fn from_sql(bytes: PgValue<'_>) -> deserialize::Result<Self> {
    match bytes.as_bytes() {
      b"closed" => Ok(RegistrationMode::Closed),
      b"require_application" => Ok(RegistrationMode::RequireApplication),
      b"open" => Ok(RegistrationMode::Open),
      _ => Err("Unrecognized enum variant".into()),
    }
  }
}
