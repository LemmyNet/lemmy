use crate::newtypes::{CommunityId, LanguageId, LocalUserId, SiteId};
#[cfg(feature = "full")]
use crate::schema::local_user_language;
use serde::{Deserialize, Serialize};

#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "full", derive(Queryable, Selectable, Identifiable))]
#[cfg_attr(feature = "full", diesel(table_name = local_user_language))]
#[cfg_attr(feature = "full", diesel(primary_key(local_user_id, language_id)))]
#[cfg_attr(feature = "full", diesel(check_for_backend(diesel::pg::Pg)))]
pub struct LocalUserLanguage {
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

#[cfg(feature = "full")]
use crate::schema::community_language;

#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "full", derive(Queryable, Selectable, Identifiable))]
#[cfg_attr(feature = "full", diesel(table_name = community_language))]
#[cfg_attr(feature = "full", diesel(primary_key(community_id, language_id)))]
#[cfg_attr(feature = "full", diesel(check_for_backend(diesel::pg::Pg)))]
pub struct CommunityLanguage {
  pub community_id: CommunityId,
  pub language_id: LanguageId,
}

#[derive(Clone)]
#[cfg_attr(feature = "full", derive(Insertable, AsChangeset))]
#[cfg_attr(feature = "full", diesel(table_name = community_language))]
pub struct CommunityLanguageForm {
  pub community_id: CommunityId,
  pub language_id: LanguageId,
}

#[cfg(feature = "full")]
use crate::schema::site_language;

#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "full", derive(Queryable, Selectable, Identifiable))]
#[cfg_attr(feature = "full", diesel(table_name = site_language))]
#[cfg_attr(feature = "full", diesel(primary_key(site_id, language_id)))]
#[cfg_attr(feature = "full", diesel(check_for_backend(diesel::pg::Pg)))]
pub struct SiteLanguage {
  pub site_id: SiteId,
  pub language_id: LanguageId,
}

#[derive(Clone, Debug)]
#[cfg_attr(feature = "full", derive(Insertable, AsChangeset))]
#[cfg_attr(feature = "full", diesel(table_name = site_language))]
pub struct SiteLanguageForm {
  pub site_id: SiteId,
  pub language_id: LanguageId,
}
