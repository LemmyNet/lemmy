use crate::newtypes::LanguageId;
use serde::{Deserialize, Serialize};

#[cfg(feature = "full")]
use crate::schema::language;

#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "full", derive(Queryable, Identifiable))]
#[cfg_attr(feature = "full", diesel(table_name = language))]
pub struct Language {
  pub id: LanguageId,
  pub code: String,
  pub name: String,
}
