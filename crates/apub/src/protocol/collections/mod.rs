pub(crate) mod empty_outbox;
pub(crate) mod group_followers;
pub(crate) mod group_moderators;
pub(crate) mod group_outbox;

#[cfg(test)]
mod tests {
  use crate::protocol::{
    collections::{
      empty_outbox::EmptyOutbox,
      group_followers::GroupFollowers,
      group_moderators::GroupModerators,
      group_outbox::GroupOutbox,
    },
    tests::test_parse_lemmy_item,
  };

  #[test]
  fn test_parse_lemmy_collections() {
    test_parse_lemmy_item::<GroupFollowers>("assets/lemmy/collections/group_followers.json")
      .unwrap();
    let outbox =
      test_parse_lemmy_item::<GroupOutbox>("assets/lemmy/collections/group_outbox.json").unwrap();
    assert_eq!(outbox.ordered_items.len() as i32, outbox.total_items);
    test_parse_lemmy_item::<GroupModerators>("assets/lemmy/collections/group_moderators.json")
      .unwrap();
    test_parse_lemmy_item::<EmptyOutbox>("assets/lemmy/collections/person_outbox.json").unwrap();
  }
}
