pub(crate) mod empty_outbox;
pub(crate) mod group_featured;
pub(crate) mod group_followers;
pub(crate) mod group_moderators;
pub(crate) mod group_outbox;

#[cfg(test)]
mod tests {
  use crate::protocol::{
    collections::{
      empty_outbox::EmptyOutbox,
      group_featured::GroupFeatured,
      group_followers::GroupFollowers,
      group_moderators::GroupModerators,
      group_outbox::GroupOutbox,
    },
    tests::{test_json, test_parse_lemmy_item},
  };
  use pretty_assertions::assert_eq;
  use lemmy_utils::error::LemmyResult;

  #[test]
  fn test_parse_lemmy_collections() -> LemmyResult<()> {
    test_parse_lemmy_item::<GroupFollowers>("assets/lemmy/collections/group_followers.json")?;
    let outbox =
      test_parse_lemmy_item::<GroupOutbox>("assets/lemmy/collections/group_outbox.json")?;
    assert_eq!(outbox.ordered_items.len() as i32, outbox.total_items);
    test_parse_lemmy_item::<GroupFeatured>("assets/lemmy/collections/group_featured_posts.json")?;
    test_parse_lemmy_item::<GroupModerators>("assets/lemmy/collections/group_moderators.json")?;
    test_parse_lemmy_item::<EmptyOutbox>("assets/lemmy/collections/person_outbox.json")?;
    Ok(())
  }

  #[test]
  fn test_parse_mastodon_collections() -> LemmyResult<()> {
    test_json::<GroupFeatured>("assets/mastodon/collections/featured.json")?;
    Ok(())
  }
}
