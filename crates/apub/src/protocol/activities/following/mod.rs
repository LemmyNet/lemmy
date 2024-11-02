pub(crate) mod accept;
pub mod follow;
pub mod undo_follow;

#[cfg(test)]
mod tests {
  use crate::protocol::{
    activities::following::{accept::AcceptFollow, follow::Follow, undo_follow::UndoFollow},
    tests::test_parse_lemmy_item,
  };
  use lemmy_utils::error::LemmyResult;

  #[test]
  fn test_parse_lemmy_accept_follow() -> LemmyResult<()> {
    test_parse_lemmy_item::<Follow>("assets/lemmy/activities/following/follow.json")?;
    test_parse_lemmy_item::<AcceptFollow>("assets/lemmy/activities/following/accept.json")?;
    test_parse_lemmy_item::<UndoFollow>("assets/lemmy/activities/following/undo_follow.json")?;
    Ok(())
  }
}
