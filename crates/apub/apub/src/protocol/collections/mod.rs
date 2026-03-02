pub(crate) mod group_featured;
pub(crate) mod group_followers;
pub(crate) mod group_moderators;
pub(crate) mod group_outbox;
pub mod url_collection;

#[cfg(test)]
#[expect(clippy::as_conversions)]
mod tests {
  use crate::protocol::collections::{
    group_featured::GroupFeatured,
    group_followers::GroupFollowers,
    group_moderators::GroupModerators,
    group_outbox::GroupOutbox,
    url_collection::UrlCollection,
  };
  use lemmy_apub_objects::utils::test::{test_json, test_parse_lemmy_item};
  use lemmy_utils::error::LemmyResult;
  use pretty_assertions::assert_eq;

  #[test]
  fn test_parse_lemmy_collections() -> LemmyResult<()> {
    test_parse_lemmy_item::<GroupFollowers>("assets/lemmy/collections/group_followers.json")?;
    let outbox =
      test_parse_lemmy_item::<GroupOutbox>("assets/lemmy/collections/group_outbox.json")?;
    assert_eq!(outbox.ordered_items.len(), outbox.total_items as usize);
    test_parse_lemmy_item::<GroupFeatured>("assets/lemmy/collections/group_featured_posts.json")?;
    test_parse_lemmy_item::<GroupModerators>("assets/lemmy/collections/group_moderators.json")?;
    test_parse_lemmy_item::<UrlCollection>("assets/lemmy/collections/person_outbox.json")?;
    Ok(())
  }

  #[test]
  fn test_parse_mastodon_collections() -> LemmyResult<()> {
    test_json::<GroupFeatured>("assets/mastodon/collections/featured.json")?;
    Ok(())
  }
}
