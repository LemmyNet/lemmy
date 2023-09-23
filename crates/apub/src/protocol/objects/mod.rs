use lemmy_db_schema::{
  impls::actor_language::UNDETERMINED_ID, newtypes::LanguageId, source::language::Language,
  utils::DbPool,
};
use lemmy_utils::error::LemmyError;
use serde::{Deserialize, Serialize};
use url::Url;

pub(crate) mod chat_message;
pub(crate) mod group;
pub(crate) mod instance;
pub(crate) mod note;
pub(crate) mod page;
pub(crate) mod person;
pub(crate) mod tombstone;

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Endpoints {
  pub shared_inbox: Url,
}

/// As specified in https://schema.org/Language
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct LanguageTag {
  pub(crate) identifier: String,
  pub(crate) name: String,
}

impl LanguageTag {
  pub(crate) async fn new_single(
    lang: LanguageId,
    pool: &mut DbPool<'_>,
  ) -> Result<Option<LanguageTag>, LemmyError> {
    let lang = Language::read_from_id(pool, lang).await?;

    // undetermined
    if lang.id == UNDETERMINED_ID {
      Ok(None)
    } else {
      Ok(Some(LanguageTag {
        identifier: lang.code,
        name: lang.name,
      }))
    }
  }

  pub(crate) async fn new_multiple(
    lang_ids: Vec<LanguageId>,
    pool: &mut DbPool<'_>,
  ) -> Result<Vec<LanguageTag>, LemmyError> {
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
    lang: Option<Self>,
    pool: &mut DbPool<'_>,
  ) -> Result<Option<LanguageId>, LemmyError> {
    let identifier = lang.map(|l| l.identifier);
    let language = Language::read_id_from_code(pool, identifier.as_deref()).await?;

    Ok(language)
  }

  pub(crate) async fn to_language_id_multiple(
    langs: Vec<Self>,
    pool: &mut DbPool<'_>,
  ) -> Result<Vec<LanguageId>, LemmyError> {
    let mut language_ids = Vec::new();

    for l in langs {
      let id = l.identifier;
      language_ids.push(Language::read_id_from_code(pool, Some(&id)).await?);
    }

    Ok(language_ids.into_iter().flatten().collect())
  }
}

#[cfg(test)]
mod tests {
  #![allow(clippy::unwrap_used)]
  #![allow(clippy::indexing_slicing)]

  use crate::protocol::{
    objects::{
      chat_message::ChatMessage, group::Group, instance::Instance, note::Note, page::Page,
      person::Person, tombstone::Tombstone,
    },
    tests::{test_json, test_parse_lemmy_item},
  };

  #[test]
  fn test_parse_objects_lemmy() {
    test_parse_lemmy_item::<Instance>("assets/lemmy/objects/instance.json").unwrap();
    test_parse_lemmy_item::<Group>("assets/lemmy/objects/group.json").unwrap();
    test_parse_lemmy_item::<Person>("assets/lemmy/objects/person.json").unwrap();
    test_parse_lemmy_item::<Page>("assets/lemmy/objects/page.json").unwrap();
    test_parse_lemmy_item::<Note>("assets/lemmy/objects/note.json").unwrap();
    test_parse_lemmy_item::<ChatMessage>("assets/lemmy/objects/chat_message.json").unwrap();
    test_parse_lemmy_item::<Tombstone>("assets/lemmy/objects/tombstone.json").unwrap();
  }

  #[test]
  fn test_parse_objects_pleroma() {
    test_json::<Person>("assets/pleroma/objects/person.json").unwrap();
    test_json::<Note>("assets/pleroma/objects/note.json").unwrap();
    test_json::<ChatMessage>("assets/pleroma/objects/chat_message.json").unwrap();
  }

  #[test]
  fn test_parse_objects_smithereen() {
    test_json::<Person>("assets/smithereen/objects/person.json").unwrap();
    test_json::<Note>("assets/smithereen/objects/note.json").unwrap();
  }

  #[test]
  fn test_parse_objects_mastodon() {
    test_json::<Person>("assets/mastodon/objects/person.json").unwrap();
    test_json::<Note>("assets/mastodon/objects/note.json").unwrap();
    test_json::<Page>("assets/mastodon/objects/page.json").unwrap();
  }

  #[test]
  fn test_parse_objects_lotide() {
    test_json::<Group>("assets/lotide/objects/group.json").unwrap();
    test_json::<Person>("assets/lotide/objects/person.json").unwrap();
    test_json::<Note>("assets/lotide/objects/note.json").unwrap();
    test_json::<Page>("assets/lotide/objects/page.json").unwrap();
    test_json::<Tombstone>("assets/lotide/objects/tombstone.json").unwrap();
  }

  #[test]
  fn test_parse_object_friendica() {
    test_json::<Person>("assets/friendica/objects/person_1.json").unwrap();
    test_json::<Person>("assets/friendica/objects/person_2.json").unwrap();
    test_json::<Page>("assets/friendica/objects/page_1.json").unwrap();
    test_json::<Page>("assets/friendica/objects/page_2.json").unwrap();
    test_json::<Note>("assets/friendica/objects/note_1.json").unwrap();
    test_json::<Note>("assets/friendica/objects/note_2.json").unwrap();
  }

  #[test]
  fn test_parse_object_gnusocial() {
    test_json::<Person>("assets/gnusocial/objects/person.json").unwrap();
    test_json::<Group>("assets/gnusocial/objects/group.json").unwrap();
    test_json::<Page>("assets/gnusocial/objects/page.json").unwrap();
    test_json::<Note>("assets/gnusocial/objects/note.json").unwrap();
  }

  #[test]
  fn test_parse_object_peertube() {
    test_json::<Person>("assets/peertube/objects/person.json").unwrap();
    test_json::<Group>("assets/peertube/objects/group.json").unwrap();
    test_json::<Page>("assets/peertube/objects/video.json").unwrap();
    test_json::<Note>("assets/peertube/objects/note.json").unwrap();
  }

  #[test]
  fn test_parse_object_mobilizon() {
    test_json::<Group>("assets/mobilizon/objects/group.json").unwrap();
    test_json::<Page>("assets/mobilizon/objects/event.json").unwrap();
    test_json::<Person>("assets/mobilizon/objects/person.json").unwrap();
  }
}
