pub(crate) mod chat_message;
pub(crate) mod group;
pub(crate) mod note;
pub(crate) mod page;
pub(crate) mod person;
pub(crate) mod tombstone;

#[cfg(test)]
mod tests {
  use crate::protocol::{
    objects::{chat_message::ChatMessage, group::Group, note::Note, page::Page, person::Person},
    tests::test_parse_lemmy_item,
  };
  use serial_test::serial;

  #[actix_rt::test]
  #[serial]
  async fn test_parse_lemmy_object() {
    test_parse_lemmy_item::<Person>("assets/lemmy/objects/person.json");
    test_parse_lemmy_item::<Group>("assets/lemmy/objects/group.json");
    test_parse_lemmy_item::<Page>("assets/lemmy/objects/page.json");
    test_parse_lemmy_item::<Note>("assets/lemmy/objects/note.json");
    test_parse_lemmy_item::<ChatMessage>("assets/lemmy/objects/chat_message.json");
  }
}
