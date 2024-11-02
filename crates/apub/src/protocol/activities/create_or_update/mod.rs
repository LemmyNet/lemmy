pub mod chat_message;
pub mod note;
pub mod page;

#[cfg(test)]
mod tests {
  use crate::protocol::{
    activities::create_or_update::{
      chat_message::CreateOrUpdateChatMessage,
      note::CreateOrUpdateNote,
      page::CreateOrUpdatePage,
    },
    tests::test_parse_lemmy_item,
  };
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
      "assets/lemmy/activities/create_or_update/create_note.json",
    )?;
    test_parse_lemmy_item::<CreateOrUpdateChatMessage>(
      "assets/lemmy/activities/create_or_update/create_private_message.json",
    )?;
    Ok(())
  }
}
