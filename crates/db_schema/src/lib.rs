#[macro_use]
extern crate diesel;

use chrono::NaiveDateTime;
use diesel::{
  backend::Backend,
  deserialize::FromSql,
  serialize::{Output, ToSql},
  sql_types::Text,
};
use serde::Serialize;
use std::{
  fmt::{Display, Formatter},
  io::Write,
};

pub mod schema;
pub mod source;

#[repr(transparent)]
#[derive(Clone, PartialEq, Serialize, Debug, AsExpression, FromSqlRow)]
#[sql_type = "Text"]
pub struct Url(url::Url);

impl<DB: Backend> ToSql<Text, DB> for Url
where
  String: ToSql<Text, DB>,
{
  fn to_sql<W: Write>(&self, out: &mut Output<W, DB>) -> diesel::serialize::Result {
    self.0.to_string().to_sql(out)
  }
}

impl<DB: Backend> FromSql<Text, DB> for Url
where
  String: FromSql<Text, DB>,
{
  fn from_sql(bytes: Option<&DB::RawValue>) -> diesel::deserialize::Result<Self> {
    let str = String::from_sql(bytes)?;
    Ok(Url(url::Url::parse(&str)?))
  }
}

impl Url {
  pub fn into_inner(self) -> url::Url {
    self.0
  }
}

impl Display for Url {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    self.to_owned().into_inner().fmt(f)
  }
}

impl From<Url> for url::Url {
  fn from(url: Url) -> Self {
    url.0
  }
}

impl From<url::Url> for Url {
  fn from(url: url::Url) -> Self {
    Url(url)
  }
}

// TODO: can probably move this back to lemmy_db_queries
pub fn naive_now() -> NaiveDateTime {
  chrono::prelude::Utc::now().naive_utc()
}
