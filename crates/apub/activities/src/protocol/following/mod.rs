pub(crate) mod accept;
pub mod follow;
pub(crate) mod reject;
pub mod undo_follow;

#[cfg(test)]
mod tests {
  use crate::protocol::following::{accept::AcceptFollow, follow::Follow, undo_follow::UndoFollow};
  use lemmy_apub_objects::utils::test::test_parse_lemmy_item;
  use lemmy_utils::error::LemmyResult;

  #[test]
  fn test_parse_lemmy_accept_follow() -> LemmyResult<()> {
    test_parse_lemmy_item::<Follow>("../apub/assets/lemmy/activities/following/follow.json")?;
    test_parse_lemmy_item::<AcceptFollow>("../apub/assets/lemmy/activities/following/accept.json")?;
    test_parse_lemmy_item::<UndoFollow>(
      "../apub/assets/lemmy/activities/following/undo_follow.json",
    )?;
    Ok(())
  }
}
