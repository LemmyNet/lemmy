use crate::{
  collections::community_moderators::ApubCommunityModerators,
  fetcher::UserOrCommunity,
  objects::person::ApubPerson,
};
use activitypub_federation::fetch::{collection_id::CollectionId, object_id::ObjectId};
use lemmy_db_schema::{
  impls::actor_language::UNDETERMINED_ID,
  newtypes::{DbUrl, LanguageId},
  source::language::Language,
  utils::DbPool,
};
use lemmy_utils::error::LemmyResult;
use serde::{Deserialize, Serialize};
use std::ops::Deref;
use url::Url;

pub(crate) mod instance;
pub(crate) mod note;
pub(crate) mod private_message;
pub(crate) mod tombstone;

#[cfg(test)]
mod tests {
  use crate::protocol::{
    activities::create_or_update::note_wrapper::NoteWrapper,
    objects::{
      group::Group,
      instance::Instance,
      note::Note,
      page::Page,
      person::Person,
      private_message::PrivateMessage,
      tombstone::Tombstone,
    },
    tests::{test_json, test_parse_lemmy_item},
  };
  use lemmy_utils::error::LemmyResult;

  #[test]
  fn test_parse_objects_lemmy() -> LemmyResult<()> {
    test_parse_lemmy_item::<Instance>("assets/lemmy/objects/instance.json")?;
    test_parse_lemmy_item::<Group>("assets/lemmy/objects/group.json")?;
    test_parse_lemmy_item::<Person>("assets/lemmy/objects/person.json")?;
    test_parse_lemmy_item::<Page>("assets/lemmy/objects/page.json")?;
    test_parse_lemmy_item::<Note>("assets/lemmy/objects/comment.json")?;
    test_parse_lemmy_item::<PrivateMessage>("assets/lemmy/objects/private_message.json")?;
    test_parse_lemmy_item::<NoteWrapper>("assets/lemmy/objects/comment.json")?;
    test_parse_lemmy_item::<NoteWrapper>("assets/lemmy/objects/private_message.json")?;
    test_parse_lemmy_item::<Tombstone>("assets/lemmy/objects/tombstone.json")?;
    Ok(())
  }

  #[test]
  fn test_parse_objects_pleroma() -> LemmyResult<()> {
    test_json::<Person>("assets/pleroma/objects/person.json")?;
    test_json::<Note>("assets/pleroma/objects/note.json")?;
    Ok(())
  }

  #[test]
  fn test_parse_objects_smithereen() -> LemmyResult<()> {
    test_json::<Person>("assets/smithereen/objects/person.json")?;
    test_json::<Note>("assets/smithereen/objects/note.json")?;
    Ok(())
  }

  #[test]
  fn test_parse_objects_mastodon() -> LemmyResult<()> {
    test_json::<Person>("assets/mastodon/objects/person.json")?;
    test_json::<Note>("assets/mastodon/objects/note_1.json")?;
    test_json::<Note>("assets/mastodon/objects/note_2.json")?;
    test_json::<Page>("assets/mastodon/objects/page.json")?;
    Ok(())
  }

  #[test]
  fn test_parse_objects_lotide() -> LemmyResult<()> {
    test_json::<Group>("assets/lotide/objects/group.json")?;
    test_json::<Person>("assets/lotide/objects/person.json")?;
    test_json::<Note>("assets/lotide/objects/note.json")?;
    test_json::<Page>("assets/lotide/objects/page.json")?;
    test_json::<Tombstone>("assets/lotide/objects/tombstone.json")?;
    Ok(())
  }

  #[test]
  fn test_parse_object_friendica() -> LemmyResult<()> {
    test_json::<Person>("assets/friendica/objects/person_1.json")?;
    test_json::<Person>("assets/friendica/objects/person_2.json")?;
    test_json::<Page>("assets/friendica/objects/page_1.json")?;
    test_json::<Page>("assets/friendica/objects/page_2.json")?;
    test_json::<Note>("assets/friendica/objects/note_1.json")?;
    test_json::<Note>("assets/friendica/objects/note_2.json")?;
    Ok(())
  }

  #[test]
  fn test_parse_object_gnusocial() -> LemmyResult<()> {
    test_json::<Person>("assets/gnusocial/objects/person.json")?;
    test_json::<Group>("assets/gnusocial/objects/group.json")?;
    test_json::<Page>("assets/gnusocial/objects/page.json")?;
    test_json::<Note>("assets/gnusocial/objects/note.json")?;
    Ok(())
  }

  #[test]
  fn test_parse_object_peertube() -> LemmyResult<()> {
    test_json::<Person>("assets/peertube/objects/person.json")?;
    test_json::<Group>("assets/peertube/objects/group.json")?;
    test_json::<Page>("assets/peertube/objects/video.json")?;
    test_json::<Note>("assets/peertube/objects/note.json")?;
    Ok(())
  }

  #[test]
  fn test_parse_object_mobilizon() -> LemmyResult<()> {
    test_json::<Group>("assets/mobilizon/objects/group.json")?;
    test_json::<Page>("assets/mobilizon/objects/event.json")?;
    test_json::<Person>("assets/mobilizon/objects/person.json")?;
    Ok(())
  }

  #[test]
  fn test_parse_object_discourse() -> LemmyResult<()> {
    test_json::<Group>("assets/discourse/objects/group.json")?;
    test_json::<Page>("assets/discourse/objects/page.json")?;
    test_json::<Person>("assets/discourse/objects/person.json")?;
    Ok(())
  }

  #[test]
  fn test_parse_object_nodebb() -> LemmyResult<()> {
    test_json::<Group>("assets/nodebb/objects/group.json")?;
    test_json::<Page>("assets/nodebb/objects/page.json")?;
    test_json::<Person>("assets/nodebb/objects/person.json")?;
    Ok(())
  }

  #[test]
  fn test_parse_object_wordpress() -> LemmyResult<()> {
    test_json::<Group>("assets/wordpress/objects/group.json")?;
    test_json::<Page>("assets/wordpress/objects/page.json")?;
    test_json::<Person>("assets/wordpress/objects/person.json")?;
    test_json::<Note>("assets/wordpress/objects/note.json")?;
    Ok(())
  }
}
