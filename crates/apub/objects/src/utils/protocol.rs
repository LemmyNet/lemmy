use crate::objects::{UserOrCommunity, community::ApubCommunity, person::ApubPerson};
use activitypub_federation::{
  config::Data,
  fetch::object_id::ObjectId,
  kinds::object::ImageType,
  protocol::{tombstone::Tombstone, values::MediaTypeMarkdown},
};
use lemmy_api_utils::context::LemmyContext;
use lemmy_db_schema::{
  impls::actor_language::UNDETERMINED_ID,
  newtypes::LanguageId,
  source::language::Language,
};
use lemmy_diesel_utils::{connection::DbPool, dburl::DbUrl};
use lemmy_utils::error::LemmyResult;
use serde::{Deserialize, Serialize};
use std::{future::Future, ops::Deref};
use url::Url;

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Source {
  pub(crate) content: String,
  pub(crate) media_type: MediaTypeMarkdown,
}

impl Source {
  pub(crate) fn new(content: String) -> Self {
    Source {
      content,
      media_type: MediaTypeMarkdown::Markdown,
    }
  }
}

pub trait InCommunity {
  fn community(
    &self,
    context: &Data<LemmyContext>,
  ) -> impl Future<Output = LemmyResult<ApubCommunity>> + Send;
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ImageObject {
  #[serde(rename = "type")]
  kind: ImageType,
  pub url: Url,
}

impl ImageObject {
  pub(crate) fn new(url: DbUrl) -> Self {
    ImageObject {
      kind: ImageType::Image,
      url: url.into(),
    }
  }
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(untagged)]
pub enum AttributedTo {
  Lemmy(PersonOrGroupModerators),
  Peertube(Vec<AttributedToPeertube>),
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub enum PersonOrGroupType {
  Person,
  Group,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct AttributedToPeertube {
  #[serde(rename = "type")]
  pub kind: PersonOrGroupType,
  pub id: ObjectId<UserOrCommunity>,
}

impl AttributedTo {
  pub fn url(self) -> Option<DbUrl> {
    match self {
      AttributedTo::Lemmy(l) => Some(l.moderators().into()),
      AttributedTo::Peertube(_) => None,
    }
  }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct PersonOrGroupModerators(Url);

impl Deref for PersonOrGroupModerators {
  type Target = Url;

  fn deref(&self) -> &Self::Target {
    &self.0
  }
}

impl From<DbUrl> for PersonOrGroupModerators {
  fn from(value: DbUrl) -> Self {
    PersonOrGroupModerators(value.into())
  }
}

impl PersonOrGroupModerators {
  pub(crate) fn creator(&self) -> ObjectId<ApubPerson> {
    self.deref().clone().into()
  }

  pub fn moderators(&self) -> Url {
    self.deref().clone()
  }
}

/// As specified in https://schema.org/Language
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub(crate) struct LanguageTag {
  pub(crate) identifier: String,
  pub(crate) name: String,
}

impl Default for LanguageTag {
  fn default() -> Self {
    LanguageTag {
      identifier: "und".to_string(),
      name: "Undetermined".to_string(),
    }
  }
}

impl LanguageTag {
  pub(crate) async fn new_single(
    lang: LanguageId,
    pool: &mut DbPool<'_>,
  ) -> LemmyResult<LanguageTag> {
    let lang = Language::read_from_id(pool, lang).await?;

    // undetermined
    if lang.id == UNDETERMINED_ID {
      Ok(LanguageTag::default())
    } else {
      Ok(LanguageTag {
        identifier: lang.code,
        name: lang.name,
      })
    }
  }

  pub(crate) async fn new_multiple(
    lang_ids: Vec<LanguageId>,
    pool: &mut DbPool<'_>,
  ) -> LemmyResult<Vec<LanguageTag>> {
    let mut langs = Vec::<Language>::new();

    for l in lang_ids {
      langs.push(Language::read_from_id(pool, l).await?);
    }

    let langs = langs
      .into_iter()
      .map(|l| LanguageTag {
        identifier: l.code,
        name: l.name,
      })
      .collect();
    Ok(langs)
  }

  pub(crate) async fn to_language_id_single(
    lang: Self,
    pool: &mut DbPool<'_>,
  ) -> LemmyResult<LanguageId> {
    Language::read_id_from_code(pool, &lang.identifier).await
  }

  pub(crate) async fn to_language_id_multiple(
    langs: Vec<Self>,
    pool: &mut DbPool<'_>,
  ) -> LemmyResult<Vec<LanguageId>> {
    let mut language_ids = Vec::new();

    for l in langs {
      let id = l.identifier;
      language_ids.push(Language::read_id_from_code(pool, &id).await?);
    }

    Ok(language_ids.into_iter().collect())
  }
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Endpoints {
  pub shared_inbox: Url,
}

pub trait Id {
  fn id(&self) -> &Url;
}

impl Id for Tombstone {
  fn id(&self) -> &Url {
    &self.id
  }
}
