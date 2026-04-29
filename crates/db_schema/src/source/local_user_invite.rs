use crate::newtypes::{InvitationId, LocalUserId};
use chrono::{DateTime, Utc};
#[cfg(feature = "full")]
use i_love_jesus::CursorKeysModule;
use lemmy_db_schema_file::schema::local_user_invite;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, PartialEq, Eq, Debug, Clone)]
#[cfg_attr(
  feature = "full",
  derive(Queryable, Selectable, Identifiable, CursorKeysModule)
)]
#[cfg_attr(feature = "full", diesel(table_name = local_user_invite))]
#[cfg_attr(feature = "full", diesel(check_for_backend(diesel::pg::Pg)))]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "full", cursor_keys_module(name = invitation_keys))]
pub struct LocalUserInvite {
  #[serde(skip_serializing)]
  pub id: InvitationId,
  pub token: String,
  #[serde(skip_serializing)]
  pub local_user_id: LocalUserId,
  pub max_uses: Option<i32>,
  pub uses_count: i32,
  pub expires_at: Option<DateTime<Utc>>,
  pub published_at: DateTime<Utc>,
}

#[cfg_attr(feature = "full", derive(Insertable))]
#[cfg_attr(feature = "full", diesel(table_name = local_user_invite))]
pub struct LocalUserInviteInsertForm {
  pub token: String,
  pub local_user_id: LocalUserId,
  pub max_uses: Option<i32>,
  pub expires_at: Option<DateTime<Utc>>,
}

#[cfg_attr(feature = "full", derive(AsChangeset))]
#[cfg_attr(feature = "full", diesel(table_name = local_user_invite))]
pub struct LocalUserInviteUpdateForm {
  pub uses_count: Option<i32>,
}
