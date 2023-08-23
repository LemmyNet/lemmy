#[cfg(feature = "full")]
use activitypub_federation::{
  fetch::collection_id::CollectionId,
  fetch::object_id::ObjectId,
  traits::Collection,
  traits::Object,
};
#[cfg(feature = "full")]
use diesel_ltree::Ltree;
use lemmy_proc_macros::id_newtype;
use serde::{Deserialize, Serialize};
use std::{
  fmt,
  fmt::{Display, Formatter},
  ops::Deref,
};
#[cfg(feature = "full")]
use ts_rs::TS;
use url::Url;

// The post id.
id_newtype!(PostId + ts + public + display);

// The person id.
id_newtype!(PersonId + public + ts);

// The comment id.
id_newtype!(CommentId + public + ts + display);

// The community id.
id_newtype!(CommunityId + public + ts);

// The local user id.
id_newtype!(LocalUserId + public + ts);

// The private message id.
id_newtype!(PrivateMessageId + ts + display);

// The person mention id.
id_newtype!(PersonMentionId + ts);

// The person block id.
id_newtype!(PersonBlockId + ts);

// The community block id.
id_newtype!(CommunityBlockId + ts);

// The comment report id.
id_newtype!(CommentReportId + ts);

// The post report id.
id_newtype!(PostReportId + ts);

// The private message report id.
id_newtype!(PrivateMessageReportId + ts);

// The site id.
id_newtype!(SiteId + ts);

// The language id.
id_newtype!(LanguageId + public + ts);

id_newtype!(LocalUserLanguageId + public);

id_newtype!(SiteLanguageId + public);

id_newtype!(CommunityLanguageId + public);

// The comment reply id.
id_newtype!(CommentReplyId + ts);

// The instance id.
id_newtype!(InstanceId + ts);

// The local site id.
id_newtype!(LocalSiteId + ts);

// The custom emoji id.
id_newtype!(CustomEmojiId + ts);

#[cfg(feature = "full")]
#[derive(Serialize, Deserialize)]
#[serde(remote = "Ltree")]
/// Do remote derivation for the Ltree struct
pub struct LtreeDef(pub String);

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
impl TS for DbUrl {
  fn name() -> String {
    "string".to_string()
  }
  fn dependencies() -> Vec<ts_rs::Dependency> {
    Vec::new()
  }
  fn transparent() -> bool {
    true
  }
}
