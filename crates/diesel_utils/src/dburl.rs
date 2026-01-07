#[cfg(feature = "full")]
use activitypub_federation::{
  fetch::{collection_id::CollectionId, object_id::ObjectId},
  traits::{Collection, Object},
};
#[cfg(feature = "full")]
use diesel::{
  backend::Backend,
  deserialize::{FromSql, FromSqlRow},
  expression::AsExpression,
  pg::Pg,
  serialize::{Output, ToSql},
  sql_types::Text,
};
use serde::{Deserialize, Serialize};
use std::{
  fmt::{Display, Formatter},
  ops::Deref,
};
use url::Url;

#[repr(transparent)]
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize, Debug, Hash)]
#[cfg_attr(feature = "full", derive(AsExpression, FromSqlRow))]
#[cfg_attr(feature = "full", diesel(sql_type = diesel::sql_types::Text))]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
pub struct DbUrl(pub Box<Url>);

impl DbUrl {
  pub fn to_lowercase(&self) -> String {
    self.as_str().to_lowercase()
  }
}

impl DbUrl {
  pub fn inner(&self) -> &Url {
    &self.0
  }
}

impl Display for DbUrl {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    self.clone().0.fmt(f)
  }
}

// the project doesn't compile with From
#[expect(clippy::from_over_into)]
impl Into<DbUrl> for Url {
  fn into(self) -> DbUrl {
    DbUrl(Box::new(self))
  }
}
#[expect(clippy::from_over_into)]
impl Into<Url> for DbUrl {
  fn into(self) -> Url {
    *self.0
  }
}

#[cfg(feature = "full")]
impl<T> From<DbUrl> for ObjectId<T>
where
  T: Object + Send + 'static,
  for<'de2> <T as Object>::Kind: Deserialize<'de2>,
{
  fn from(value: DbUrl) -> Self {
    let url: Url = value.into();
    ObjectId::from(url)
  }
}

#[cfg(feature = "full")]
impl<T> From<DbUrl> for CollectionId<T>
where
  T: Collection + Send + 'static,
  for<'de2> <T as Collection>::Kind: Deserialize<'de2>,
{
  fn from(value: DbUrl) -> Self {
    let url: Url = value.into();
    CollectionId::from(url)
  }
}

#[cfg(feature = "full")]
impl<T> From<CollectionId<T>> for DbUrl
where
  T: Collection,
  for<'de2> <T as Collection>::Kind: Deserialize<'de2>,
{
  fn from(value: CollectionId<T>) -> Self {
    let url: Url = value.into();
    url.into()
  }
}

impl Deref for DbUrl {
  type Target = Url;

  fn deref(&self) -> &Self::Target {
    &self.0
  }
}

#[cfg(feature = "full")]
impl ToSql<Text, Pg> for DbUrl {
  fn to_sql(&self, out: &mut Output<Pg>) -> diesel::serialize::Result {
    <std::string::String as ToSql<Text, Pg>>::to_sql(&self.0.to_string(), &mut out.reborrow())
  }
}

#[cfg(feature = "full")]
impl<DB: Backend> FromSql<Text, DB> for DbUrl
where
  String: FromSql<Text, DB>,
{
  fn from_sql(value: DB::RawValue<'_>) -> diesel::deserialize::Result<Self> {
    let str = String::from_sql(value)?;
    Ok(DbUrl(Box::new(Url::parse(&str)?)))
  }
}

#[cfg(feature = "full")]
impl<Kind> From<ObjectId<Kind>> for DbUrl
where
  Kind: Object + Send + 'static,
  for<'de2> <Kind as Object>::Kind: serde::Deserialize<'de2>,
{
  fn from(id: ObjectId<Kind>) -> Self {
    DbUrl(Box::new(id.into()))
  }
}
