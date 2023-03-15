#[cfg(feature = "full")]
use activitypub_federation::{
  fetch::collection_id::CollectionId,
  fetch::object_id::ObjectId,
  traits::ApubCollection,
  traits::ApubObject,
};
#[cfg(feature = "full")]
use diesel_ltree::Ltree;
use serde::{Deserialize, Serialize};
use std::{
  fmt,
  fmt::{Display, Formatter},
  ops::Deref,
};
use url::Url;

#[derive(Debug, Copy, Clone, Hash, Eq, PartialEq, Default, Serialize, Deserialize)]
#[cfg_attr(feature = "full", derive(DieselNewType))]
pub struct PostId(pub i32);

impl fmt::Display for PostId {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "{}", self.0)
  }
}

#[derive(Debug, Copy, Clone, Hash, Eq, PartialEq, Default, Serialize, Deserialize)]
#[cfg_attr(feature = "full", derive(DieselNewType))]
pub struct PersonId(pub i32);

#[derive(Debug, Copy, Clone, Hash, Eq, PartialEq, Serialize, Deserialize, Default)]
#[cfg_attr(feature = "full", derive(DieselNewType))]
pub struct CommentId(pub i32);

impl fmt::Display for CommentId {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "{}", self.0)
  }
}

#[derive(Debug, Copy, Clone, Hash, Eq, PartialEq, Default, Serialize, Deserialize)]
#[cfg_attr(feature = "full", derive(DieselNewType))]
pub struct CommunityId(pub i32);

#[derive(Debug, Copy, Clone, Hash, Eq, PartialEq, Default, Serialize, Deserialize)]
#[cfg_attr(feature = "full", derive(DieselNewType))]
pub struct LocalUserId(pub i32);

#[derive(Debug, Copy, Clone, Hash, Eq, PartialEq, Serialize, Deserialize, Default)]
#[cfg_attr(feature = "full", derive(DieselNewType))]
pub struct PrivateMessageId(i32);

impl fmt::Display for PrivateMessageId {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "{}", self.0)
  }
}

#[derive(Debug, Copy, Clone, Hash, Eq, PartialEq, Serialize, Deserialize, Default)]
#[cfg_attr(feature = "full", derive(DieselNewType))]
pub struct PersonMentionId(i32);

#[derive(Debug, Copy, Clone, Hash, Eq, PartialEq, Serialize, Deserialize, Default)]
#[cfg_attr(feature = "full", derive(DieselNewType))]
pub struct PersonBlockId(i32);

#[derive(Debug, Copy, Clone, Hash, Eq, PartialEq, Serialize, Deserialize, Default)]
#[cfg_attr(feature = "full", derive(DieselNewType))]
pub struct CommunityBlockId(i32);

#[derive(Debug, Copy, Clone, Hash, Eq, PartialEq, Serialize, Deserialize, Default)]
#[cfg_attr(feature = "full", derive(DieselNewType))]
pub struct CommentReportId(i32);

#[derive(Debug, Copy, Clone, Hash, Eq, PartialEq, Serialize, Deserialize, Default)]
#[cfg_attr(feature = "full", derive(DieselNewType))]
pub struct PostReportId(i32);

#[derive(Debug, Copy, Clone, Hash, Eq, PartialEq, Serialize, Deserialize, Default)]
#[cfg_attr(feature = "full", derive(DieselNewType))]
pub struct PrivateMessageReportId(i32);

#[derive(Debug, Copy, Clone, Hash, Eq, PartialEq, Serialize, Deserialize, Default)]
#[cfg_attr(feature = "full", derive(DieselNewType))]
pub struct SiteId(i32);

#[derive(Debug, Copy, Clone, Hash, Eq, PartialEq, Serialize, Deserialize, Default)]
#[cfg_attr(feature = "full", derive(DieselNewType))]
pub struct LanguageId(pub i32);

#[derive(Debug, Copy, Clone, Hash, Eq, PartialEq, Serialize, Deserialize, Default)]
#[cfg_attr(feature = "full", derive(DieselNewType))]
pub struct LocalUserLanguageId(pub i32);

#[derive(Debug, Copy, Clone, Hash, Eq, PartialEq, Serialize, Deserialize, Default)]
#[cfg_attr(feature = "full", derive(DieselNewType))]
pub struct SiteLanguageId(pub i32);

#[derive(Debug, Copy, Clone, Hash, Eq, PartialEq, Serialize, Deserialize, Default)]
#[cfg_attr(feature = "full", derive(DieselNewType))]
pub struct CommunityLanguageId(pub i32);

#[derive(Debug, Copy, Clone, Hash, Eq, PartialEq, Serialize, Deserialize, Default)]
#[cfg_attr(feature = "full", derive(DieselNewType))]
pub struct CommentReplyId(i32);

#[derive(Debug, Copy, Clone, Hash, Eq, PartialEq, Serialize, Deserialize, Default)]
#[cfg_attr(feature = "full", derive(DieselNewType))]
pub struct InstanceId(i32);

#[derive(Debug, Copy, Clone, Hash, Eq, PartialEq, Serialize, Deserialize, Default)]
#[cfg_attr(feature = "full", derive(DieselNewType))]
pub struct LocalSiteId(i32);

#[derive(Debug, Copy, Clone, Hash, Eq, PartialEq, Serialize, Deserialize, Default)]
#[cfg_attr(feature = "full", derive(DieselNewType))]
pub struct CustomEmojiId(i32);

#[repr(transparent)]
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize, Debug)]
#[cfg_attr(feature = "full", derive(AsExpression, FromSqlRow))]
#[cfg_attr(feature = "full", diesel(sql_type = diesel::sql_types::Text))]
pub struct DbUrl(pub(crate) Box<Url>);

impl DbUrl {
  pub fn inner(&self) -> &Url {
    &self.0
  }
}

#[cfg(feature = "full")]
#[derive(Serialize, Deserialize)]
#[serde(remote = "Ltree")]
/// Do remote derivation for the Ltree struct
pub struct LtreeDef(pub String);

impl Display for DbUrl {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    self.clone().0.fmt(f)
  }
}

// the project doesnt compile with From
#[allow(clippy::from_over_into)]
impl Into<DbUrl> for Url {
  fn into(self) -> DbUrl {
    DbUrl(Box::new(self))
  }
}
#[allow(clippy::from_over_into)]
impl Into<Url> for DbUrl {
  fn into(self) -> Url {
    *self.0
  }
}

#[cfg(feature = "full")]
impl<T> From<DbUrl> for ObjectId<T>
where
  T: ApubObject + Send + 'static,
  for<'de2> <T as ApubObject>::ApubType: Deserialize<'de2>,
{
  fn from(value: DbUrl) -> Self {
    let url: Url = value.into();
    ObjectId::from(url)
  }
}

#[cfg(feature = "full")]
impl<T> From<DbUrl> for CollectionId<T>
where
  T: ApubCollection + Send + 'static,
  for<'de2> <T as ApubCollection>::ApubType: Deserialize<'de2>,
{
  fn from(value: DbUrl) -> Self {
    let url: Url = value.into();
    CollectionId::from(url)
  }
}

#[cfg(feature = "full")]
impl<T> From<CollectionId<T>> for DbUrl
where
  T: ApubCollection,
  for<'de2> <T as ApubCollection>::ApubType: Deserialize<'de2>,
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
