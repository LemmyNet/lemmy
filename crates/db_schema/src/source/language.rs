use crate::newtypes::LanguageId;
#[cfg(feature = "full")]
use crate::schema::language;
use serde::{Deserialize, Serialize};

#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "full", derive(Queryable, Identifiable))]
#[cfg_attr(feature = "full", diesel(table_name = language))]
pub struct Language {
  pub id: LanguageId,
  pub code: String,
  pub name: String,
}
