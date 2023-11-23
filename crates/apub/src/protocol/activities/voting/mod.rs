pub mod undo_vote;
pub mod vote;

#[cfg(test)]
mod tests {
  use crate::protocol::{
    activities::voting::{undo_vote::UndoVote, vote::Vote},
    tests::test_parse_lemmy_item,
  };
  use lemmy_utils::error::LemmyResult;

  #[test]
  fn test_parse_lemmy_voting() -> LemmyResult<()> {
    test_parse_lemmy_item::<Vote>("assets/lemmy/activities/voting/like_note.json")?;
    test_parse_lemmy_item::<Vote>("assets/lemmy/activities/voting/dislike_page.json")?;

    test_parse_lemmy_item::<UndoVote>("assets/lemmy/activities/voting/undo_like_note.json")?;
    test_parse_lemmy_item::<UndoVote>("assets/lemmy/activities/voting/undo_dislike_page.json")?;
    Ok(())
  }
}
