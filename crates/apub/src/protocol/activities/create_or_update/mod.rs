pub mod comment;
pub mod post;

#[cfg(test)]
mod tests {
  use crate::{
    context::WithContext,
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
    )
    .unwrap();
    test_parse_lemmy_item::<CreateOrUpdatePost>(
      "assets/lemmy/activities/create_or_update/update_page.json",
    )
    .unwrap();
    test_parse_lemmy_item::<CreateOrUpdateComment>(
      "assets/lemmy/activities/create_or_update/create_note.json",
    )
    .unwrap();

    file_to_json_object::<WithContext<CreateOrUpdateComment>>(
      "assets/pleroma/activities/create_note.json",
    )
    .unwrap();
    file_to_json_object::<WithContext<CreateOrUpdateComment>>(
      "assets/smithereen/activities/create_note.json",
    )
    .unwrap();
    file_to_json_object::<CreateOrUpdateComment>("assets/mastodon/activities/create_note.json")
      .unwrap();

    file_to_json_object::<CreateOrUpdatePost>("assets/lotide/activities/create_page.json").unwrap();
    file_to_json_object::<CreateOrUpdateComment>("assets/lotide/activities/create_note_reply.json")
      .unwrap();
  }
}
