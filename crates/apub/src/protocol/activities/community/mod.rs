pub mod add_mod;
pub mod announce;
pub mod block_user;
pub mod remove_mod;
pub mod report;
pub mod undo_block_user;
pub mod update;

#[cfg(test)]
mod tests {
  use crate::protocol::{
    activities::community::{
      add_mod::AddMod,
      block_user::BlockUserFromCommunity,
      remove_mod::RemoveMod,
      report::Report,
      undo_block_user::UndoBlockUserFromCommunity,
      update::UpdateCommunity,
    },
    tests::test_parse_lemmy_item,
  };
  use activitystreams::activity::Announce;
  use serial_test::serial;

  #[actix_rt::test]
  #[serial]
  async fn test_parse_lemmy_community() {
    test_parse_lemmy_item::<Announce>(
      "assets/lemmy/activities/community/announce_create_page.json",
    );

    test_parse_lemmy_item::<AddMod>("assets/lemmy/activities/community/add_mod.json");
    test_parse_lemmy_item::<RemoveMod>("assets/lemmy/activities/community/remove_mod.json");

    test_parse_lemmy_item::<BlockUserFromCommunity>(
      "assets/lemmy/activities/community/block_user.json",
    );
    test_parse_lemmy_item::<UndoBlockUserFromCommunity>(
      "assets/lemmy/activities/community/undo_block_user.json",
    );

    test_parse_lemmy_item::<UpdateCommunity>(
      "assets/lemmy/activities/community/update_community.json",
    );

    test_parse_lemmy_item::<Report>("assets/lemmy/activities/community/report_page.json");
  }
}
