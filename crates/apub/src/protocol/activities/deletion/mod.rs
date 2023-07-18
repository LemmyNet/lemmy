pub mod delete;
pub mod delete_user;
pub mod undo_delete;

#[cfg(test)]
mod tests {
  #![allow(clippy::unwrap_used)]
  #![allow(clippy::indexing_slicing)]

  use crate::protocol::{
    activities::deletion::{delete::Delete, delete_user::DeleteUser, undo_delete::UndoDelete},
    tests::test_parse_lemmy_item,
  };

  #[test]
  fn test_parse_lemmy_deletion() {
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

    test_parse_lemmy_item::<DeleteUser>("assets/lemmy/activities/deletion/delete_user.json")
      .unwrap();
  }
}
