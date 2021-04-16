#[macro_use]
extern crate diesel;

#[macro_use]
extern crate diesel_derive_newtype;

use chrono::NaiveDateTime;
use diesel::{
  backend::Backend,
  deserialize::FromSql,
  serialize::{Output, ToSql},
  sql_types::Text,
};
use language_tags::LanguageTag;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::{
  fmt,
  fmt::{Display, Formatter},
  io::Write,
};
use url::Url;

pub mod schema;
pub mod source;

#[derive(
  Debug, Copy, Clone, Hash, Eq, PartialEq, Default, Serialize, Deserialize, DieselNewType,
)]
pub struct PostId(pub i32);

impl fmt::Display for PostId {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "{}", self.0)
  }
}

#[derive(
  Debug, Copy, Clone, Hash, Eq, PartialEq, Default, Serialize, Deserialize, DieselNewType,
)]
pub struct PersonId(pub i32);

#[derive(Debug, Copy, Clone, Hash, Eq, PartialEq, Serialize, Deserialize, DieselNewType)]
pub struct CommentId(pub i32);

impl fmt::Display for CommentId {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "{}", self.0)
  }
}

#[derive(
  Debug, Copy, Clone, Hash, Eq, PartialEq, Default, Serialize, Deserialize, DieselNewType,
)]
pub struct CommunityId(pub i32);

#[derive(Debug, Copy, Clone, Hash, Eq, PartialEq, Serialize, Deserialize, DieselNewType)]
pub struct LocalUserId(pub i32);

#[derive(Debug, Copy, Clone, Hash, Eq, PartialEq, Serialize, Deserialize, DieselNewType)]
pub struct PrivateMessageId(i32);

impl fmt::Display for PrivateMessageId {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "{}", self.0)
  }
}

#[derive(Debug, Copy, Clone, Hash, Eq, PartialEq, Serialize, Deserialize, DieselNewType)]
pub struct PersonMentionId(i32);

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

#[repr(transparent)]
#[derive(Clone, PartialEq, Debug, AsExpression, FromSqlRow)]
#[sql_type = "Text"]
pub struct DbLanguage(LanguageTag);

impl<DB: Backend> ToSql<Text, DB> for DbLanguage
where
  String: ToSql<Text, DB>,
{
  fn to_sql<W: Write>(&self, out: &mut Output<W, DB>) -> diesel::serialize::Result {
    self.0.to_string().to_sql(out)
  }
}

// TODO: hopefully the crate will add serde support
//       https://github.com/pyfisch/rust-language-tags/issues/22
impl Serialize for DbLanguage {
  fn serialize<S>(&self, _serializer: S) -> Result<<S as Serializer>::Ok, <S as Serializer>::Error>
  where
    S: Serializer,
  {
    todo!()
  }
}

impl<'de> Deserialize<'de> for DbLanguage {
  fn deserialize<D>(_deserializer: D) -> Result<Self, <D as Deserializer<'de>>::Error>
  where
    D: Deserializer<'de>,
  {
    todo!()
  }
}

impl<DB: Backend> FromSql<Text, DB> for DbLanguage
where
  String: FromSql<Text, DB>,
{
  fn from_sql(bytes: Option<&DB::RawValue>) -> diesel::deserialize::Result<Self> {
    let str = String::from_sql(bytes)?;
    Ok(DbLanguage(LanguageTag::parse(&str)?))
  }
}

impl DbLanguage {
  pub fn into_inner(self) -> LanguageTag {
    self.0
  }
}

impl Display for DbLanguage {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    self.to_owned().into_inner().fmt(f)
  }
}

impl From<DbLanguage> for LanguageTag {
  fn from(url: DbLanguage) -> Self {
    url.0
  }
}

impl From<LanguageTag> for DbLanguage {
  fn from(lang: LanguageTag) -> Self {
    DbLanguage(lang)
  }
}
