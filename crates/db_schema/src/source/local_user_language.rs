use crate::newtypes::{LanguageId, LocalUserId, LocalUserLanguageId};
use serde::{Deserialize, Serialize};

#[cfg(feature = "full")]
use crate::schema::local_user_language;

#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "full", derive(Queryable, Identifiable))]
#[cfg_attr(feature = "full", diesel(table_name = local_user_language))]
pub struct LocalUserLanguage {
  #[serde(skip)]
  pub id: LocalUserLanguageId,
  pub local_user_id: LocalUserId,
  pub language_id: LanguageId,
}

#[derive(Clone)]
#[cfg_attr(feature = "full", derive(Insertable, AsChangeset))]
#[cfg_attr(feature = "full", diesel(table_name = local_user_language))]
pub struct LocalUserLanguageForm {
  pub local_user_id: LocalUserId,
  pub language_id: LanguageId,
}
