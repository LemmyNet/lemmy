use diesel::{
  backend::Backend,
  deserialize::FromSql,
  serialize::{Output, ToSql},
  sql_types::Text,
};
use lemmy_apub_lib::{object_id::ObjectId, traits::ApubObject};
use serde::{Deserialize, Serialize};
use std::{
  fmt,
  fmt::{Display, Formatter},
  io::Write,
};
use url::Url;

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

#[derive(Debug, Copy, Clone, Hash, Eq, PartialEq, Serialize, Deserialize, DieselNewType)]
pub struct PersonBlockId(i32);

#[derive(Debug, Copy, Clone, Hash, Eq, PartialEq, Serialize, Deserialize, DieselNewType)]
pub struct CommunityBlockId(i32);

#[derive(Debug, Copy, Clone, Hash, Eq, PartialEq, Serialize, Deserialize, DieselNewType)]
pub struct CommentReportId(i32);

#[derive(Debug, Copy, Clone, Hash, Eq, PartialEq, Serialize, Deserialize, DieselNewType)]
pub struct PostReportId(i32);

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

impl Display for DbUrl {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    self.to_owned().0.fmt(f)
  }
}

// the project doesnt compile with From
#[allow(clippy::from_over_into)]
impl Into<DbUrl> for Url {
  fn into(self) -> DbUrl {
    DbUrl(self)
  }
}
#[allow(clippy::from_over_into)]
impl Into<Url> for DbUrl {
  fn into(self) -> Url {
    self.0
  }
}

impl<Kind> From<ObjectId<Kind>> for DbUrl
where
  Kind: ApubObject + Send + 'static,
  for<'de2> <Kind as ApubObject>::ApubType: serde::Deserialize<'de2>,
{
  fn from(id: ObjectId<Kind>) -> Self {
    DbUrl(id.into())
  }
}
