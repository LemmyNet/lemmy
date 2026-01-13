use crate::newtypes::LanguageId;
use derive_aliases::derive;
#[cfg(feature = "full")]
use lemmy_db_schema_file::schema::language;
use serde::{Deserialize, Serialize};

#[derive(..ApiStruct)]
#[cfg_attr(feature = "full", derive(..SqlStruct))]
#[cfg_attr(feature = "full", diesel(table_name = language))]
#[cfg_attr(feature = "full", diesel(check_for_backend(diesel::pg::Pg)))]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
/// A language.
pub struct Language {
  pub id: LanguageId,
  pub code: String,
  pub name: String,
}
