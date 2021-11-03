pub mod comment;
pub mod post;

#[cfg(test)]
mod tests {
  use crate::{
    objects::tests::file_to_json_object,
    protocol::{
      activities::create_or_update::{comment::CreateOrUpdateComment, post::CreateOrUpdatePost},
      tests::test_parse_lemmy_item,
    },
  };
  use serial_test::serial;

  #[actix_rt::test]
  #[serial]
  async fn test_parse_lemmy_create_or_update() {
    test_parse_lemmy_item::<CreateOrUpdatePost>(
      "assets/lemmy/activities/create_or_update/create_page.json",
    );
    test_parse_lemmy_item::<CreateOrUpdatePost>(
      "assets/lemmy/activities/create_or_update/update_page.json",
    );
    test_parse_lemmy_item::<CreateOrUpdateComment>(
      "assets/lemmy/activities/create_or_update/create_note.json",
    );
  }

  #[actix_rt::test]
  #[serial]
  async fn test_parse_pleroma_create_or_update() {
    file_to_json_object::<CreateOrUpdateComment>("assets/pleroma/activities/create_note.json");
  }
}
