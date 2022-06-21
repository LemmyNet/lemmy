use crate::newtypes::LanguageId;
use serde::{Deserialize, Serialize};

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "full", derive(Queryable, Identifiable))]
#[cfg_attr(feature = "full", table_name = "language")]
pub struct Language {
  pub id: LanguageId,
  pub code: String,
  pub name: String,
}
