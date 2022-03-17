pub mod add_mod;
pub mod announce;
pub mod remove_mod;
pub mod report;
pub mod update;

#[cfg(test)]
mod tests {
  use crate::protocol::{
    activities::community::{
      add_mod::AddMod,
      announce::AnnounceActivity,
      remove_mod::RemoveMod,
      report::Report,
      update::UpdateCommunity,
    },
    tests::test_parse_lemmy_item,
  };

  #[test]
  fn test_parse_lemmy_community_activities() {
    test_parse_lemmy_item::<AnnounceActivity>(
      "assets/lemmy/activities/community/announce_create_page.json",
    )
    .unwrap();

    test_parse_lemmy_item::<AddMod>("assets/lemmy/activities/community/add_mod.json").unwrap();
    test_parse_lemmy_item::<RemoveMod>("assets/lemmy/activities/community/remove_mod.json")
      .unwrap();

    test_parse_lemmy_item::<UpdateCommunity>(
      "assets/lemmy/activities/community/update_community.json",
    )
    .unwrap();

    test_parse_lemmy_item::<Report>("assets/lemmy/activities/community/report_page.json").unwrap();
  }
}
