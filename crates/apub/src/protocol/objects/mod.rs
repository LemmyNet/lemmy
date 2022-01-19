use serde::{Deserialize, Serialize};
use url::Url;

pub(crate) mod chat_message;
pub(crate) mod group;
pub(crate) mod note;
pub(crate) mod page;
pub(crate) mod person;
pub(crate) mod tombstone;

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Endpoints {
  pub shared_inbox: Url,
}

#[cfg(test)]
mod tests {
  use crate::{
    context::WithContext,
    objects::tests::file_to_json_object,
    protocol::{
      objects::{
        chat_message::ChatMessage,
        group::Group,
        note::Note,
        page::Page,
        person::Person,
        tombstone::Tombstone,
      },
      tests::test_parse_lemmy_item,
    },
  };

  #[actix_rt::test]
  async fn test_parse_object_lemmy() {
    test_parse_lemmy_item::<Person>("assets/lemmy/objects/person.json").unwrap();
    test_parse_lemmy_item::<Group>("assets/lemmy/objects/group.json").unwrap();
    test_parse_lemmy_item::<Page>("assets/lemmy/objects/page.json").unwrap();
    test_parse_lemmy_item::<Note>("assets/lemmy/objects/note.json").unwrap();
    test_parse_lemmy_item::<ChatMessage>("assets/lemmy/objects/chat_message.json").unwrap();
    test_parse_lemmy_item::<Tombstone>("assets/lemmy/objects/tombstone.json").unwrap();
  }

  #[actix_rt::test]
  async fn test_parse_object_pleroma() {
    file_to_json_object::<WithContext<Person>>("assets/pleroma/objects/person.json").unwrap();
    file_to_json_object::<WithContext<Note>>("assets/pleroma/objects/note.json").unwrap();
    file_to_json_object::<WithContext<ChatMessage>>("assets/pleroma/objects/chat_message.json")
      .unwrap();
  }

  #[actix_rt::test]
  async fn test_parse_object_smithereen() {
    file_to_json_object::<WithContext<Person>>("assets/smithereen/objects/person.json").unwrap();
    file_to_json_object::<Note>("assets/smithereen/objects/note.json").unwrap();
  }

  #[actix_rt::test]
  async fn test_parse_object_mastodon() {
    file_to_json_object::<WithContext<Person>>("assets/mastodon/objects/person.json").unwrap();
    file_to_json_object::<WithContext<Note>>("assets/mastodon/objects/note.json").unwrap();
  }

  #[actix_rt::test]
  async fn test_parse_object_lotide() {
    file_to_json_object::<WithContext<Group>>("assets/lotide/objects/group.json").unwrap();
    file_to_json_object::<WithContext<Person>>("assets/lotide/objects/person.json").unwrap();
    file_to_json_object::<WithContext<Note>>("assets/lotide/objects/note.json").unwrap();
    file_to_json_object::<WithContext<Page>>("assets/lotide/objects/page.json").unwrap();
    file_to_json_object::<WithContext<Tombstone>>("assets/lotide/objects/tombstone.json").unwrap();
  }
}
