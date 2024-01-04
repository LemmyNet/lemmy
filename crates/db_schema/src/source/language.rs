use crate::newtypes::LanguageId;
#[cfg(feature = "full")]
use crate::schema::language;
use serde::{Deserialize, Serialize};
#[cfg(feature = "full")]
use ts_rs::TS;

#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "full", derive(Queryable, Selectable, Identifiable, TS))]
#[cfg_attr(feature = "full", diesel(table_name = language))]
#[cfg_attr(feature = "full", diesel(check_for_backend(diesel::pg::Pg)))]
#[cfg_attr(feature = "full", ts(export))]
/// A language.
pub struct Language {
  pub id: LanguageId,
  pub code: String,
  pub name: String,
}
