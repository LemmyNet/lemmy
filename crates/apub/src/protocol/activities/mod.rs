use serde::{Deserialize, Serialize};
use strum_macros::Display;

pub mod block;
pub mod community;
pub mod create_or_update;
pub mod deletion;
pub mod following;
pub mod voting;

#[derive(Clone, Debug, Display, Deserialize, Serialize, PartialEq)]
pub enum CreateOrUpdateType {
  Create,
  Update,
}

#[cfg(test)]
mod tests {
  use crate::protocol::{
    activities::{
      community::announce::AnnounceActivity,
      create_or_update::{comment::CreateOrUpdateComment, post::CreateOrUpdatePost},
      deletion::delete::Delete,
      following::{follow::FollowCommunity, undo_follow::UndoFollowCommunity},
      voting::{undo_vote::UndoVote, vote::Vote},
    },
    tests::test_json,
  };

  #[test]
  fn test_parse_smithereen_activities() {
    test_json::<CreateOrUpdateComment>("assets/smithereen/activities/create_note.json").unwrap();
  }

  #[test]
  fn test_parse_pleroma_activities() {
    test_json::<CreateOrUpdateComment>("assets/pleroma/activities/create_note.json").unwrap();
    test_json::<Delete>("assets/pleroma/activities/delete.json").unwrap();
    test_json::<FollowCommunity>("assets/pleroma/activities/follow.json").unwrap();
  }

  #[test]
  fn test_parse_mastodon_activities() {
    test_json::<CreateOrUpdateComment>("assets/mastodon/activities/create_note.json").unwrap();
    test_json::<Delete>("assets/mastodon/activities/delete.json").unwrap();
    test_json::<FollowCommunity>("assets/mastodon/activities/follow.json").unwrap();
    test_json::<UndoFollowCommunity>("assets/mastodon/activities/undo_follow.json").unwrap();
    test_json::<Vote>("assets/mastodon/activities/like_page.json").unwrap();
    test_json::<UndoVote>("assets/mastodon/activities/undo_like_page.json").unwrap();
  }

  #[test]
  fn test_parse_lotide_activities() {
    test_json::<CreateOrUpdatePost>("assets/lotide/activities/create_page.json").unwrap();
    test_json::<CreateOrUpdateComment>("assets/lotide/activities/create_note_reply.json").unwrap();
  }

  #[test]
  fn test_parse_friendica_activities() {
    test_json::<CreateOrUpdatePost>("assets/friendica/activities/create_page_1.json").unwrap();
    test_json::<CreateOrUpdatePost>("assets/friendica/activities/create_page_2.json").unwrap();
    test_json::<CreateOrUpdateComment>("assets/friendica/activities/create_note.json").unwrap();
    test_json::<CreateOrUpdateComment>("assets/friendica/activities/update_note.json").unwrap();
    test_json::<Delete>("assets/friendica/activities/delete.json").unwrap();
    test_json::<Vote>("assets/friendica/activities/like_page.json").unwrap();
    test_json::<Vote>("assets/friendica/activities/dislike_page.json").unwrap();
    test_json::<UndoVote>("assets/friendica/activities/undo_dislike_page.json").unwrap();
  }

  #[test]
  fn test_parse_gnusocial_activities() {
    test_json::<CreateOrUpdatePost>("assets/gnusocial/activities/create_page.json").unwrap();
    test_json::<CreateOrUpdateComment>("assets/gnusocial/activities/create_note.json").unwrap();
    test_json::<Vote>("assets/gnusocial/activities/like_note.json").unwrap();
  }

  #[test]
  fn test_parse_peertube_activities() {
    test_json::<AnnounceActivity>("assets/peertube/activities/announce_video.json").unwrap();
  }
}
