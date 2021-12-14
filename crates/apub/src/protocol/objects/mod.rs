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
  #[serde(skip_serializing_if = "Option::is_none")]
  pub shared_inbox: Option<Url>,
}

#[cfg(test)]
mod tests {
  use crate::{
    context::WithContext,
    objects::tests::file_to_json_object,
    protocol::{
      objects::{chat_message::ChatMessage, group::Group, note::Note, page::Page, person::Person},
      tests::test_parse_lemmy_item,
    },
  };

  #[actix_rt::test]
  async fn test_parse_object() {
    test_parse_lemmy_item::<Person>("assets/lemmy/objects/person.json");
    test_parse_lemmy_item::<Group>("assets/lemmy/objects/group.json");
    test_parse_lemmy_item::<Page>("assets/lemmy/objects/page.json");
    test_parse_lemmy_item::<Note>("assets/lemmy/objects/note.json");
    test_parse_lemmy_item::<ChatMessage>("assets/lemmy/objects/chat_message.json");

    file_to_json_object::<WithContext<Person>>("assets/pleroma/objects/person.json");
    file_to_json_object::<WithContext<Note>>("assets/pleroma/objects/note.json");
    file_to_json_object::<WithContext<ChatMessage>>("assets/pleroma/objects/chat_message.json");

    file_to_json_object::<WithContext<Person>>("assets/smithereen/objects/person.json");
    file_to_json_object::<Note>("assets/smithereen/objects/note.json");

    file_to_json_object::<Person>("assets/mastodon/objects/person.json");
    file_to_json_object::<Note>("assets/mastodon/objects/note.json");
  }
}
