pub mod block_user;
pub mod undo_block_user;

#[cfg(test)]
mod tests {
  use crate::protocol::{
    activities::block::{block_user::BlockUser, undo_block_user::UndoBlockUser},
    tests::test_parse_lemmy_item,
  };

  #[test]
  fn test_parse_lemmy_block() {
    test_parse_lemmy_item::<BlockUser>("assets/lemmy/activities/block/block_user.json").unwrap();
    test_parse_lemmy_item::<UndoBlockUser>("assets/lemmy/activities/block/undo_block_user.json")
      .unwrap();
  }
}
