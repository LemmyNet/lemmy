use crate::newtypes::{
  CommunityId,
  CommunityLanguageId,
  LanguageId,
  LocalUserId,
  LocalUserLanguageId,
  SiteId,
  SiteLanguageId,
};
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

#[cfg(feature = "full")]
use crate::schema::community_language;

#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "full", derive(Queryable, Identifiable))]
#[cfg_attr(feature = "full", diesel(table_name = community_language))]
pub struct CommunityLanguage {
  #[serde(skip)]
  pub id: CommunityLanguageId,
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
#[cfg_attr(feature = "full", derive(Queryable, Identifiable))]
#[cfg_attr(feature = "full", diesel(table_name = site_language))]
pub struct SiteLanguage {
  #[serde(skip)]
  pub id: SiteLanguageId,
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
