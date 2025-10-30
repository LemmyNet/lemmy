use crate::newtypes::{CommentId, CommunityId, InstanceId, ModlogId, PersonId, PostId};
use chrono::{DateTime, Utc};
#[cfg(feature = "full")]
use i_love_jesus::CursorKeysModule;
use lemmy_db_schema_file::enums::ModlogKind;
#[cfg(feature = "full")]
use lemmy_db_schema_file::schema::modlog;
use serde::{Deserialize, Serialize};

#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
#[cfg_attr(
  feature = "full",
  derive(Queryable, Selectable, Identifiable, CursorKeysModule)
)]
#[cfg_attr(feature = "full", diesel(table_name = modlog))]
#[cfg_attr(feature = "full", diesel(check_for_backend(diesel::pg::Pg)))]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
#[cfg_attr(feature = "full", cursor_keys_module(name = modlog_keys))]
pub struct Modlog {
  pub id: ModlogId,
  pub kind: ModlogKind,
  pub is_revert: bool,
  #[serde(skip)]
  pub mod_id: PersonId,
  pub reason: Option<String>,
  #[serde(skip)]
  pub target_person_id: Option<PersonId>,
  #[serde(skip)]
  pub target_community_id: Option<CommunityId>,
  #[serde(skip)]
  pub target_post_id: Option<PostId>,
  #[serde(skip)]
  pub target_comment_id: Option<CommentId>,
  #[serde(skip)]
  pub target_instance_id: Option<InstanceId>,
  pub expires_at: Option<DateTime<Utc>>,
  pub published_at: DateTime<Utc>,
}

#[derive(derive_new::new)]
#[cfg_attr(feature = "full", derive(Insertable))]
#[cfg_attr(feature = "full", diesel(table_name = modlog))]
pub struct ModlogInsertForm<'a> {
  pub(crate) kind: ModlogKind,
  pub(crate) is_revert: bool,
  pub(crate) mod_id: PersonId,
  #[new(default)]
  pub(crate) reason: Option<&'a str>,
  #[new(default)]
  pub(crate) target_person_id: Option<PersonId>,
  #[new(default)]
  pub(crate) target_community_id: Option<CommunityId>,
  #[new(default)]
  pub(crate) target_post_id: Option<PostId>,
  #[new(default)]
  pub(crate) target_comment_id: Option<CommentId>,
  #[new(default)]
  pub(crate) target_instance_id: Option<InstanceId>,
  #[new(default)]
  pub(crate) expires_at: Option<DateTime<Utc>>,
}
