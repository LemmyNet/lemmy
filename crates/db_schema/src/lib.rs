#[macro_use]
extern crate diesel;

use chrono::NaiveDateTime;
use diesel::{
  backend::Backend,
  deserialize::FromSql,
  serialize::{Output, ToSql},
  sql_types::Text,
};
use serde::{Deserialize, Serialize};
use std::{
  fmt::{Display, Formatter},
  io::Write,
};
use url::Url;

pub mod schema;
pub mod source;

#[repr(transparent)]
#[derive(Clone, PartialEq, Serialize, Deserialize, Debug, AsExpression, FromSqlRow)]
#[sql_type = "Text"]
pub struct DbUrl(Url);

impl<DB: Backend> ToSql<Text, DB> for DbUrl
where
  String: ToSql<Text, DB>,
{
  fn to_sql<W: Write>(&self, out: &mut Output<W, DB>) -> diesel::serialize::Result {
    self.0.to_string().to_sql(out)
  }
}

impl<DB: Backend> FromSql<Text, DB> for DbUrl
where
  String: FromSql<Text, DB>,
{
  fn from_sql(bytes: Option<&DB::RawValue>) -> diesel::deserialize::Result<Self> {
    let str = String::from_sql(bytes)?;
    Ok(DbUrl(Url::parse(&str)?))
  }
}

impl DbUrl {
  pub fn into_inner(self) -> Url {
    self.0
  }
}

impl Display for DbUrl {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    self.to_owned().into_inner().fmt(f)
  }
}

impl From<DbUrl> for Url {
  fn from(url: DbUrl) -> Self {
    url.0
  }
}

impl From<Url> for DbUrl {
  fn from(url: Url) -> Self {
    DbUrl(url)
  }
}

// TODO: can probably move this back to lemmy_db_queries
pub fn naive_now() -> NaiveDateTime {
  chrono::prelude::Utc::now().naive_utc()
}
