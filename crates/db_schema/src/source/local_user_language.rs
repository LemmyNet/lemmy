use crate::newtypes::{LanguageId, LocalUserId, LocalUserLanguageId};
#[cfg(feature = "full")]
use crate::schema::local_user_language;
use serde::{Deserialize, Serialize};

#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "full", derive(Queryable, Selectable, Identifiable))]
#[cfg_attr(feature = "full", diesel(table_name = local_user_language))]
#[cfg_attr(feature = "full", diesel(check_for_backend(diesel::pg::Pg)))]
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
