pub(crate) mod accept;
pub mod follow;
pub mod undo_follow;

#[cfg(test)]
mod tests {
  use crate::protocol::{
    activities::following::{
      accept::AcceptFollowCommunity,
      follow::FollowCommunity,
      undo_follow::UndoFollowCommunity,
    },
    tests::test_parse_lemmy_item,
  };

  #[test]
  fn test_parse_lemmy_accept_follow() {
    test_parse_lemmy_item::<FollowCommunity>("assets/lemmy/activities/following/follow.json")
      .unwrap();
    test_parse_lemmy_item::<AcceptFollowCommunity>("assets/lemmy/activities/following/accept.json")
      .unwrap();
    test_parse_lemmy_item::<UndoFollowCommunity>(
      "assets/lemmy/activities/following/undo_follow.json",
    )
    .unwrap();
  }
}
