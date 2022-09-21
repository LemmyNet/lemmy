use lemmy_db_schema::source::language::Language;
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

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct LanguageTag {
  pub(crate) identifier: String,
  pub(crate) name: String,
}

impl LanguageTag {
  pub(crate) fn new(lang: Language) -> Option<LanguageTag> {
    // undetermined
    if lang.code == "und" {
      None
    } else {
      Some(LanguageTag {
        identifier: lang.code,
        name: lang.name,
      })
    }
  }
}

#[cfg(test)]
mod tests {
  use crate::protocol::{
    objects::{
      chat_message::ChatMessage,
      group::Group,
      instance::Instance,
      note::Note,
      page::Page,
      person::Person,
      tombstone::Tombstone,
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
}
