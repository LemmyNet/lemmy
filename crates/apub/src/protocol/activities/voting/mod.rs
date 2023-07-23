pub mod undo_vote;
pub mod vote;

#[cfg(test)]
mod tests {
  #![allow(clippy::unwrap_used)]
  #![allow(clippy::indexing_slicing)]

  use crate::protocol::{
    activities::voting::{undo_vote::UndoVote, vote::Vote},
    tests::test_parse_lemmy_item,
  };

  #[test]
  fn test_parse_lemmy_voting() {
    test_parse_lemmy_item::<Vote>("assets/lemmy/activities/voting/like_note.json").unwrap();
    test_parse_lemmy_item::<Vote>("assets/lemmy/activities/voting/dislike_page.json").unwrap();

    test_parse_lemmy_item::<UndoVote>("assets/lemmy/activities/voting/undo_like_note.json")
      .unwrap();
    test_parse_lemmy_item::<UndoVote>("assets/lemmy/activities/voting/undo_dislike_page.json")
      .unwrap();
  }
}
