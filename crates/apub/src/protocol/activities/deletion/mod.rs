pub mod delete;
pub mod undo_delete;

#[cfg(test)]
mod tests {
  use crate::{
    context::WithContext,
    objects::tests::file_to_json_object,
    protocol::{
      activities::deletion::{delete::Delete, undo_delete::UndoDelete},
      tests::test_parse_lemmy_item,
    },
  };

  #[actix_rt::test]
  async fn test_parse_deletion() {
    test_parse_lemmy_item::<Delete>("assets/lemmy/activities/deletion/remove_note.json").unwrap();
    test_parse_lemmy_item::<Delete>("assets/lemmy/activities/deletion/delete_page.json").unwrap();

    test_parse_lemmy_item::<UndoDelete>("assets/lemmy/activities/deletion/undo_remove_note.json")
      .unwrap();
    test_parse_lemmy_item::<UndoDelete>("assets/lemmy/activities/deletion/undo_delete_page.json")
      .unwrap();
    test_parse_lemmy_item::<Delete>("assets/lemmy/activities/deletion/delete_private_message.json")
      .unwrap();
    test_parse_lemmy_item::<UndoDelete>(
      "assets/lemmy/activities/deletion/undo_delete_private_message.json",
    )
    .unwrap();

    file_to_json_object::<WithContext<Delete>>("assets/pleroma/activities/delete.json").unwrap();
    file_to_json_object::<WithContext<Delete>>("assets/mastodon/activities/delete.json").unwrap();
  }
}
