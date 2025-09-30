pub mod announce;
pub mod collection_add;
pub mod collection_remove;
pub mod lock;
pub mod report;
pub mod resolve_report;
pub mod update;

#[cfg(test)]
mod tests {
  use super::resolve_report::ResolveReport;
  use crate::protocol::community::{
    announce::AnnounceActivity,
    collection_add::CollectionAdd,
    collection_remove::CollectionRemove,
    lock::{LockPageOrNote, UndoLockPageOrNote},
    report::Report,
    update::Update,
  };
  use lemmy_apub_objects::utils::test::test_parse_lemmy_item;
  use lemmy_utils::error::LemmyResult;

  #[test]
  fn test_parse_lemmy_community_activities() -> LemmyResult<()> {
    test_parse_lemmy_item::<AnnounceActivity>(
      "../apub/assets/lemmy/activities/community/announce_create_page.json",
    )?;

    test_parse_lemmy_item::<CollectionAdd>(
      "../apub/assets/lemmy/activities/community/add_mod.json",
    )?;
    test_parse_lemmy_item::<CollectionRemove>(
      "../apub/assets/lemmy/activities/community/remove_mod.json",
    )?;

    test_parse_lemmy_item::<CollectionAdd>(
      "../apub/assets/lemmy/activities/community/add_featured_post.json",
    )?;
    test_parse_lemmy_item::<CollectionRemove>(
      "../apub/assets/lemmy/activities/community/remove_featured_post.json",
    )?;

    test_parse_lemmy_item::<LockPageOrNote>(
      "../apub/assets/lemmy/activities/community/lock_page.json",
    )?;
    test_parse_lemmy_item::<UndoLockPageOrNote>(
      "../apub/assets/lemmy/activities/community/undo_lock_page.json",
    )?;

    test_parse_lemmy_item::<LockPageOrNote>(
      "../apub/assets/lemmy/activities/community/lock_note.json",
    )?;
    test_parse_lemmy_item::<UndoLockPageOrNote>(
      "../apub/assets/lemmy/activities/community/undo_lock_note.json",
    )?;

    test_parse_lemmy_item::<Update>(
      "../apub/assets/lemmy/activities/community/update_community.json",
    )?;

    test_parse_lemmy_item::<Report>("../apub/assets/lemmy/activities/community/report_page.json")?;
    test_parse_lemmy_item::<ResolveReport>(
      "../apub/assets/lemmy/activities/community/resolve_report_page.json",
    )?;

    Ok(())
  }
}
