pub(crate) mod accept;
pub mod follow;
pub mod undo_follow;

#[cfg(test)]
mod tests {
  use crate::{
    context::WithContext,
    objects::tests::file_to_json_object,
    protocol::{
      activities::following::{
        accept::AcceptFollowCommunity,
        follow::FollowCommunity,
        undo_follow::UndoFollowCommunity,
      },
      tests::test_parse_lemmy_item,
    },
  };

  #[actix_rt::test]
  async fn test_parse_lemmy_accept_follow() {
    test_parse_lemmy_item::<FollowCommunity>("assets/lemmy/activities/following/follow.json");
    test_parse_lemmy_item::<AcceptFollowCommunity>("assets/lemmy/activities/following/accept.json");
    test_parse_lemmy_item::<UndoFollowCommunity>(
      "assets/lemmy/activities/following/undo_follow.json",
    );

    file_to_json_object::<WithContext<FollowCommunity>>("assets/pleroma/activities/follow.json");
  }
}
