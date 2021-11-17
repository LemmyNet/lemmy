pub mod delete;
pub mod undo_delete;

#[cfg(test)]
mod tests {
  use crate::protocol::{
    activities::deletion::{delete::Delete, undo_delete::UndoDelete},
    tests::test_parse_lemmy_item,
  };
  use serial_test::serial;

  #[actix_rt::test]
  #[serial]
  async fn test_parse_lemmy_deletion() {
    test_parse_lemmy_item::<Delete>("assets/lemmy/activities/deletion/remove_note.json");
    test_parse_lemmy_item::<Delete>("assets/lemmy/activities/deletion/delete_page.json");

    test_parse_lemmy_item::<UndoDelete>("assets/lemmy/activities/deletion/undo_remove_note.json");
    test_parse_lemmy_item::<UndoDelete>("assets/lemmy/activities/deletion/undo_delete_page.json");
  }
}
