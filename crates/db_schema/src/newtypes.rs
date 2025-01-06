#[cfg(feature = "full")]
use activitypub_federation::{
  fetch::collection_id::CollectionId,
  fetch::object_id::ObjectId,
  traits::Collection,
  traits::Object,
};
#[cfg(feature = "full")]
use diesel::{
  backend::Backend,
  deserialize::FromSql,
  pg::Pg,
  serialize::{Output, ToSql},
  sql_types::Text,
};
#[cfg(feature = "full")]
use diesel_ltree::Ltree;
use serde::{Deserialize, Serialize};
use std::{
  fmt,
  fmt::{Display, Formatter},
  ops::Deref,
};
#[cfg(feature = "full")]
use ts_rs::TS;
use url::Url;

#[derive(Debug, Copy, Clone, Hash, Eq, PartialEq, Default, Serialize, Deserialize)]
#[cfg_attr(feature = "full", derive(DieselNewType, TS))]
#[cfg_attr(feature = "full", ts(export))]
/// The post id.
pub struct PostId(pub i32);

impl fmt::Display for PostId {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "{}", self.0)
  }
}

#[derive(Debug, Copy, Clone, Hash, Eq, PartialEq, Default, Serialize, Deserialize)]
#[cfg_attr(feature = "full", derive(DieselNewType, TS))]
#[cfg_attr(feature = "full", ts(export))]
/// The person id.
pub struct PersonId(pub i32);

#[derive(Debug, Copy, Clone, Hash, Eq, PartialEq, Serialize, Deserialize, Default)]
#[cfg_attr(feature = "full", derive(DieselNewType, TS))]
#[cfg_attr(feature = "full", ts(export))]
/// The comment id.
pub struct CommentId(pub i32);

impl fmt::Display for CommentId {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "{}", self.0)
  }
}

#[derive(Debug, Copy, Clone, Hash, Eq, PartialEq, Default, Serialize, Deserialize)]
#[cfg_attr(feature = "full", derive(DieselNewType, TS))]
#[cfg_attr(feature = "full", ts(export))]
/// The community id.
pub struct CommunityId(pub i32);

#[derive(Debug, Copy, Clone, Hash, Eq, PartialEq, Default, Serialize, Deserialize)]
#[cfg_attr(feature = "full", derive(DieselNewType, TS))]
#[cfg_attr(feature = "full", ts(export))]
/// The local user id.
pub struct LocalUserId(pub i32);

#[derive(Debug, Copy, Clone, Hash, Eq, PartialEq, Serialize, Deserialize, Default)]
#[cfg_attr(feature = "full", derive(DieselNewType, TS))]
#[cfg_attr(feature = "full", ts(export))]
/// The private message id.
pub struct PrivateMessageId(i32);

impl fmt::Display for PrivateMessageId {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "{}", self.0)
  }
}

#[derive(Debug, Copy, Clone, Hash, Eq, PartialEq, Serialize, Deserialize, Default)]
#[cfg_attr(feature = "full", derive(DieselNewType, TS))]
#[cfg_attr(feature = "full", ts(export))]
/// The person mention id.
pub struct PersonMentionId(i32);

#[derive(Debug, Copy, Clone, Hash, Eq, PartialEq, Serialize, Deserialize, Default)]
#[cfg_attr(feature = "full", derive(DieselNewType, TS))]
#[cfg_attr(feature = "full", ts(export))]
/// The comment report id.
pub struct CommentReportId(pub i32);

#[derive(Debug, Copy, Clone, Hash, Eq, PartialEq, Serialize, Deserialize, Default)]
#[cfg_attr(feature = "full", derive(DieselNewType, TS))]
#[cfg_attr(feature = "full", ts(export))]
/// The post report id.
pub struct PostReportId(pub i32);

#[derive(Debug, Copy, Clone, Hash, Eq, PartialEq, Serialize, Deserialize, Default)]
#[cfg_attr(feature = "full", derive(DieselNewType, TS))]
#[cfg_attr(feature = "full", ts(export))]
/// The private message report id.
pub struct PrivateMessageReportId(pub i32);

#[derive(Debug, Copy, Clone, Hash, Eq, PartialEq, Serialize, Deserialize, Default)]
#[cfg_attr(feature = "full", derive(DieselNewType, TS))]
#[cfg_attr(feature = "full", ts(export))]
/// The site id.
pub struct SiteId(pub i32);

#[derive(Debug, Copy, Clone, Hash, Eq, PartialEq, Serialize, Deserialize, Default)]
#[cfg_attr(feature = "full", derive(DieselNewType, TS))]
#[cfg_attr(feature = "full", ts(export))]
/// The language id.
pub struct LanguageId(pub i32);

#[derive(Debug, Copy, Clone, Hash, Eq, PartialEq, Serialize, Deserialize, Default)]
#[cfg_attr(feature = "full", derive(DieselNewType, TS))]
#[cfg_attr(feature = "full", ts(export))]
/// The comment reply id.
pub struct CommentReplyId(i32);

#[derive(
  Debug, Copy, Clone, Hash, Eq, PartialEq, Serialize, Deserialize, Default, Ord, PartialOrd,
)]
#[cfg_attr(feature = "full", derive(DieselNewType, TS))]
#[cfg_attr(feature = "full", ts(export))]
/// The instance id.
pub struct InstanceId(pub i32);

#[derive(
  Debug, Copy, Clone, Hash, Eq, PartialEq, Serialize, Deserialize, Default, PartialOrd, Ord,
)]
#[cfg_attr(feature = "full", derive(DieselNewType, TS))]
#[cfg_attr(feature = "full", ts(export))]
pub struct ActivityId(pub i64);

#[derive(Debug, Copy, Clone, Hash, Eq, PartialEq, Serialize, Deserialize, Default)]
#[cfg_attr(feature = "full", derive(DieselNewType, TS))]
#[cfg_attr(feature = "full", ts(export))]
/// The local site id.
pub struct LocalSiteId(i32);

#[derive(Debug, Copy, Clone, Hash, Eq, PartialEq, Serialize, Deserialize, Default)]
#[cfg_attr(feature = "full", derive(DieselNewType, TS))]
#[cfg_attr(feature = "full", ts(export))]
/// The custom emoji id.
pub struct CustomEmojiId(i32);

#[derive(Debug, Copy, Clone, Hash, Eq, PartialEq, Serialize, Deserialize, Default)]
#[cfg_attr(feature = "full", derive(DieselNewType, TS))]
#[cfg_attr(feature = "full", ts(export))]
/// The tagline id.
pub struct TaglineId(i32);

#[derive(Debug, Copy, Clone, Hash, Eq, PartialEq, Serialize, Deserialize, Default)]
#[cfg_attr(feature = "full", derive(DieselNewType, TS))]
#[cfg_attr(feature = "full", ts(export))]
/// The registration application id.
pub struct RegistrationApplicationId(i32);

#[derive(Debug, Copy, Clone, Hash, Eq, PartialEq, Serialize, Deserialize, Default)]
#[cfg_attr(feature = "full", derive(DieselNewType, TS))]
#[cfg_attr(feature = "full", ts(export))]
/// The oauth provider id.
pub struct OAuthProviderId(pub i32);

#[cfg(feature = "full")]
#[derive(Serialize, Deserialize)]
#[serde(remote = "Ltree")]
/// Do remote derivation for the Ltree struct
pub struct LtreeDef(pub String);

#[repr(transparent)]
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize, Debug, Hash)]
#[cfg_attr(feature = "full", derive(AsExpression, FromSqlRow, TS))]
#[cfg_attr(feature = "full", diesel(sql_type = diesel::sql_types::Text))]
#[cfg_attr(feature = "full", ts(export))]
pub struct DbUrl(pub(crate) Box<Url>);

#[derive(Debug, Copy, Clone, Hash, Eq, PartialEq, Default)]
#[cfg_attr(feature = "full", derive(DieselNewType))]
/// The report combined id
pub struct ReportCombinedId(i32);

#[derive(Debug, Copy, Clone, Hash, Eq, PartialEq, Default)]
#[cfg_attr(feature = "full", derive(DieselNewType))]
/// The person content combined id
pub struct PersonContentCombinedId(i32);

#[derive(Debug, Copy, Clone, Hash, Eq, PartialEq, Default)]
#[cfg_attr(feature = "full", derive(DieselNewType))]
/// The person saved combined id
pub struct PersonSavedCombinedId(i32);

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

impl InstanceId {
  pub fn inner(self) -> i32 {
    self.0
  }
}

#[derive(Debug, Copy, Clone, Hash, Eq, PartialEq, Default, Serialize, Deserialize)]
#[cfg_attr(feature = "full", derive(DieselNewType, TS))]
#[cfg_attr(feature = "full", ts(export))]
/// The internal tag id.
pub struct TagId(pub i32);
