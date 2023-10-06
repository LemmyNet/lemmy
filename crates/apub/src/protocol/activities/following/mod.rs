pub(crate) mod accept;
pub mod follow;
pub mod undo_follow;

#[cfg(test)]
mod tests {
  #![allow(clippy::unwrap_used)]
  #![allow(clippy::indexing_slicing)]

  use crate::protocol::{
    activities::following::{accept::AcceptFollow, follow::Follow, undo_follow::UndoFollow},
    tests::test_parse_lemmy_item,
  };

  #[test]
  fn test_parse_lemmy_accept_follow() {
    test_parse_lemmy_item::<Follow>("assets/lemmy/activities/following/follow.json").unwrap();
    test_parse_lemmy_item::<AcceptFollow>("assets/lemmy/activities/following/accept.json").unwrap();
    test_parse_lemmy_item::<UndoFollow>("assets/lemmy/activities/following/undo_follow.json")
      .unwrap();
  }
}
