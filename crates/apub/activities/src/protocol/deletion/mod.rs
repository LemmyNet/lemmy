pub mod delete;
pub mod delete_user;
pub mod undo_delete;

#[cfg(test)]
mod tests {
  use crate::protocol::deletion::{
    delete::Delete,
    delete_user::DeleteUser,
    undo_delete::UndoDelete,
  };
  use lemmy_apub_objects::utils::test::test_parse_lemmy_item;
  use lemmy_utils::error::LemmyResult;

  #[test]
  fn test_parse_lemmy_deletion() -> LemmyResult<()> {
    test_parse_lemmy_item::<Delete>("../apub/assets/lemmy/activities/deletion/remove_note.json")?;
    test_parse_lemmy_item::<Delete>("../apub/assets/lemmy/activities/deletion/delete_page.json")?;

    test_parse_lemmy_item::<UndoDelete>(
      "../apub/assets/lemmy/activities/deletion/undo_remove_note.json",
    )?;
    test_parse_lemmy_item::<UndoDelete>(
      "../apub/assets/lemmy/activities/deletion/undo_delete_page.json",
    )?;
    test_parse_lemmy_item::<Delete>(
      "../apub/assets/lemmy/activities/deletion/delete_private_message.json",
    )?;
    test_parse_lemmy_item::<UndoDelete>(
      "../apub/assets/lemmy/activities/deletion/undo_delete_private_message.json",
    )?;

    test_parse_lemmy_item::<DeleteUser>(
      "../apub/assets/lemmy/activities/deletion/delete_user.json",
    )?;
    Ok(())
  }
}
