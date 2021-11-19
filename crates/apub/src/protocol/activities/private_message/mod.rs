pub mod create_or_update;
pub mod delete;
pub mod undo_delete;

#[cfg(test)]
mod tests {
  use crate::protocol::{
    activities::private_message::{
      create_or_update::CreateOrUpdatePrivateMessage,
      delete::DeletePrivateMessage,
      undo_delete::UndoDeletePrivateMessage,
    },
    tests::test_parse_lemmy_item,
  };

  #[actix_rt::test]
  async fn test_parse_lemmy_private_message() {
    test_parse_lemmy_item::<CreateOrUpdatePrivateMessage>(
      "assets/lemmy/activities/private_message/create.json",
    );
    test_parse_lemmy_item::<DeletePrivateMessage>(
      "assets/lemmy/activities/private_message/delete.json",
    );
    test_parse_lemmy_item::<UndoDeletePrivateMessage>(
      "assets/lemmy/activities/private_message/undo_delete.json",
    );
  }
}
