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

  #[actix_rt::test]
  async fn test_parse_create_or_update() {
    test_parse_lemmy_item::<CreateOrUpdatePost>(
      "assets/lemmy/activities/create_or_update/create_page.json",
    );
    test_parse_lemmy_item::<CreateOrUpdatePost>(
      "assets/lemmy/activities/create_or_update/update_page.json",
    );
    test_parse_lemmy_item::<CreateOrUpdateComment>(
      "assets/lemmy/activities/create_or_update/create_note.json",
    );

    file_to_json_object::<CreateOrUpdateComment>("assets/pleroma/activities/create_note.json");
    file_to_json_object::<CreateOrUpdateComment>("assets/smithereen/activities/create_note.json");
    file_to_json_object::<CreateOrUpdateComment>("assets/mastodon/activities/create_note.json");
  }
}
