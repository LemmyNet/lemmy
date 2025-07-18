use chrono::{DateTime, Utc};
#[cfg(feature = "full")]
use lemmy_db_schema_file::schema::history_status;
use serde::{Deserialize, Serialize};

#[derive(PartialEq, Eq, Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "full", derive(Queryable, Selectable, Identifiable))]
#[cfg_attr(feature = "full", diesel(table_name = history_status))]
#[cfg_attr(feature = "full", diesel(check_for_backend(diesel::pg::Pg)))]
/// A status of history filling used for background updates.
pub struct HistoryStatus {
  pub id: i32,
  pub source: String,
  pub dest: String,
  pub last_scanned_id: Option<i32>,
  pub last_scanned_timestamp: Option<DateTime<Utc>>,
}

#[derive(Clone, Default)]
#[cfg_attr(feature = "full", derive(Insertable, AsChangeset))]
#[cfg_attr(feature = "full", diesel(table_name = history_status))]
pub struct HistoryStatusInsertForm {
  pub source: String,
  pub dest: String,
  pub last_scanned_id: Option<i32>,
  pub last_scanned_timestamp: Option<DateTime<Utc>>,
}

#[derive(Clone, Default)]
#[cfg_attr(feature = "full", derive(Insertable, AsChangeset))]
#[cfg_attr(feature = "full", diesel(table_name = history_status))]
pub struct HistoryStatusUpdateForm {
  pub last_scanned_id: Option<Option<i32>>,
  pub last_scanned_timestamp: Option<Option<DateTime<Utc>>>,
}
