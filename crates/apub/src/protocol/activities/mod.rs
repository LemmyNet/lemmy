use serde::{Deserialize, Serialize};
use strum_macros::Display;

pub mod block;
pub mod community;
pub mod create_or_update;
pub mod deletion;
pub mod following;
pub mod voting;

#[derive(Clone, Debug, Display, Deserialize, Serialize, PartialEq, Eq)]
pub enum CreateOrUpdateType {
  Create,
  Update,
}

#[cfg(test)]
mod tests {
  use crate::protocol::{
    activities::{
      community::{announce::AnnounceActivity, report::Report},
      create_or_update::{note::CreateOrUpdateNote, page::CreateOrUpdatePage},
      deletion::delete::Delete,
      following::{accept::AcceptFollow, follow::Follow, undo_follow::UndoFollow},
      voting::{undo_vote::UndoVote, vote::Vote},
    },
    tests::test_json,
  };
  use lemmy_utils::error::LemmyResult;

  #[test]
  fn test_parse_smithereen_activities() -> LemmyResult<()> {
    test_json::<CreateOrUpdateNote>("assets/smithereen/activities/create_note.json")?;
    Ok(())
  }

  #[test]
  fn test_parse_pleroma_activities() -> LemmyResult<()> {
    test_json::<CreateOrUpdateNote>("assets/pleroma/activities/create_note.json")?;
    test_json::<Delete>("assets/pleroma/activities/delete.json")?;
    test_json::<Follow>("assets/pleroma/activities/follow.json")?;
    Ok(())
  }

  #[test]
  fn test_parse_mastodon_activities() -> LemmyResult<()> {
    test_json::<CreateOrUpdateNote>("assets/mastodon/activities/create_note.json")?;
    test_json::<Delete>("assets/mastodon/activities/delete.json")?;
    test_json::<Follow>("assets/mastodon/activities/follow.json")?;
    test_json::<UndoFollow>("assets/mastodon/activities/undo_follow.json")?;
    test_json::<Vote>("assets/mastodon/activities/like_page.json")?;
    test_json::<UndoVote>("assets/mastodon/activities/undo_like_page.json")?;
    Ok(())
  }

  #[test]
  fn test_parse_lotide_activities() -> LemmyResult<()> {
    test_json::<Follow>("assets/lotide/activities/follow.json")?;
    test_json::<CreateOrUpdatePage>("assets/lotide/activities/create_page.json")?;
    test_json::<CreateOrUpdatePage>("assets/lotide/activities/create_page_image.json")?;
    test_json::<CreateOrUpdateNote>("assets/lotide/activities/create_note_reply.json")?;
    Ok(())
  }

  #[test]
  fn test_parse_friendica_activities() -> LemmyResult<()> {
    test_json::<CreateOrUpdatePage>("assets/friendica/activities/create_page_1.json")?;
    test_json::<CreateOrUpdatePage>("assets/friendica/activities/create_page_2.json")?;
    test_json::<CreateOrUpdateNote>("assets/friendica/activities/create_note.json")?;
    test_json::<CreateOrUpdateNote>("assets/friendica/activities/update_note.json")?;
    test_json::<Delete>("assets/friendica/activities/delete.json")?;
    test_json::<Vote>("assets/friendica/activities/like_page.json")?;
    test_json::<Vote>("assets/friendica/activities/dislike_page.json")?;
    test_json::<UndoVote>("assets/friendica/activities/undo_dislike_page.json")?;
    Ok(())
  }

  #[test]
  fn test_parse_gnusocial_activities() -> LemmyResult<()> {
    test_json::<CreateOrUpdatePage>("assets/gnusocial/activities/create_page.json")?;
    test_json::<CreateOrUpdateNote>("assets/gnusocial/activities/create_note.json")?;
    test_json::<Vote>("assets/gnusocial/activities/like_note.json")?;
    Ok(())
  }

  #[test]
  fn test_parse_peertube_activities() -> LemmyResult<()> {
    test_json::<AnnounceActivity>("assets/peertube/activities/announce_video.json")?;
    Ok(())
  }

  #[test]
  fn test_parse_mbin_activities() -> LemmyResult<()> {
    test_json::<AcceptFollow>("assets/mbin/activities/accept.json")?;
    test_json::<Report>("assets/mbin/activities/flag.json")?;
    Ok(())
  }
}
