pub mod note;
pub(crate) mod note_wrapper;
pub mod page;
pub mod private_message;

#[cfg(test)]
mod tests {
  use super::note_wrapper::{CreateOrUpdateNoteWrapper, NoteWrapper};
  use crate::protocol::activities::create_or_update::{
    note::CreateOrUpdateNote,
    page::CreateOrUpdatePage,
    private_message::CreateOrUpdatePrivateMessage,
  };
  use lemmy_apub_objects::utils::test::test_parse_lemmy_item;
  use lemmy_utils::error::LemmyResult;

  #[test]
  fn test_parse_lemmy_create_or_update() -> LemmyResult<()> {
    test_parse_lemmy_item::<CreateOrUpdatePage>(
      "assets/lemmy/activities/create_or_update/create_page.json",
    )?;
    test_parse_lemmy_item::<CreateOrUpdatePage>(
      "assets/lemmy/activities/create_or_update/update_page.json",
    )?;
    test_parse_lemmy_item::<CreateOrUpdateNote>(
      "assets/lemmy/activities/create_or_update/create_comment.json",
    )?;
    test_parse_lemmy_item::<CreateOrUpdatePrivateMessage>(
      "assets/lemmy/activities/create_or_update/create_private_message.json",
    )?;
    test_parse_lemmy_item::<CreateOrUpdateNoteWrapper>(
      "assets/lemmy/activities/create_or_update/create_comment.json",
    )?;
    test_parse_lemmy_item::<CreateOrUpdateNoteWrapper>(
      "assets/lemmy/activities/create_or_update/create_private_message.json",
    )?;
    test_parse_lemmy_item::<NoteWrapper>("assets/lemmy/objects/comment.json")?;
    test_parse_lemmy_item::<NoteWrapper>("assets/lemmy/objects/private_message.json")?;
    Ok(())
  }
}
