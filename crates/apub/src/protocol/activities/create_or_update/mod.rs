pub mod chat_message;
pub mod note;
pub mod page;

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]
    #![allow(clippy::indexing_slicing)]

    use crate::protocol::{
        activities::create_or_update::{
            chat_message::CreateOrUpdateChatMessage, note::CreateOrUpdateNote,
            page::CreateOrUpdatePage,
        },
        tests::test_parse_lemmy_item,
    };

    #[test]
    fn test_parse_lemmy_create_or_update() {
        test_parse_lemmy_item::<CreateOrUpdatePage>(
            "assets/lemmy/activities/create_or_update/create_page.json",
        )
        .unwrap();
        test_parse_lemmy_item::<CreateOrUpdatePage>(
            "assets/lemmy/activities/create_or_update/update_page.json",
        )
        .unwrap();
        test_parse_lemmy_item::<CreateOrUpdateNote>(
            "assets/lemmy/activities/create_or_update/create_note.json",
        )
        .unwrap();
        test_parse_lemmy_item::<CreateOrUpdateChatMessage>(
            "assets/lemmy/activities/create_or_update/create_private_message.json",
        )
        .unwrap();
    }
}
