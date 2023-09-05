pub mod announce;
pub mod collection_add;
pub mod collection_remove;
pub mod lock_page;
pub mod report;
pub mod update;

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]
    #![allow(clippy::indexing_slicing)]

    use crate::protocol::{
        activities::community::{
            announce::AnnounceActivity,
            collection_add::CollectionAdd,
            collection_remove::CollectionRemove,
            lock_page::{LockPage, UndoLockPage},
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

        test_parse_lemmy_item::<CollectionAdd>("assets/lemmy/activities/community/add_mod.json")
            .unwrap();
        test_parse_lemmy_item::<CollectionRemove>(
            "assets/lemmy/activities/community/remove_mod.json",
        )
        .unwrap();

        test_parse_lemmy_item::<CollectionAdd>(
            "assets/lemmy/activities/community/add_featured_post.json",
        )
        .unwrap();
        test_parse_lemmy_item::<CollectionRemove>(
            "assets/lemmy/activities/community/remove_featured_post.json",
        )
        .unwrap();

        test_parse_lemmy_item::<LockPage>("assets/lemmy/activities/community/lock_page.json")
            .unwrap();
        test_parse_lemmy_item::<UndoLockPage>(
            "assets/lemmy/activities/community/undo_lock_page.json",
        )
        .unwrap();

        test_parse_lemmy_item::<UpdateCommunity>(
            "assets/lemmy/activities/community/update_community.json",
        )
        .unwrap();

        test_parse_lemmy_item::<Report>("assets/lemmy/activities/community/report_page.json")
            .unwrap();
    }
}
