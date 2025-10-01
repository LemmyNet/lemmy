use activitypub_federation::{config::Data, fetch::fetch_object_http};
use lemmy_api_utils::context::LemmyContext;
use lemmy_apub_objects::utils::protocol::Id;
use lemmy_utils::error::LemmyResult;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use strum::Display;
use url::Url;

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

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(untagged)]
pub enum IdOrNestedObject<Kind: Id> {
  Id(Url),
  NestedObject(Kind),
}

impl<Kind: Id + DeserializeOwned + Send> IdOrNestedObject<Kind> {
  pub(crate) fn id(&self) -> &Url {
    match self {
      IdOrNestedObject::Id(i) => i,
      IdOrNestedObject::NestedObject(n) => n.id(),
    }
  }
  pub async fn object(self, context: &Data<LemmyContext>) -> LemmyResult<Kind> {
    match self {
      // TODO: move IdOrNestedObject struct to library and make fetch_object_http private
      IdOrNestedObject::Id(i) => Ok(fetch_object_http(&i, context).await?.object),
      IdOrNestedObject::NestedObject(o) => Ok(o),
    }
  }
}

#[cfg(test)]
mod tests {
  use crate::protocol::{
    community::{announce::AnnounceActivity, report::Report},
    create_or_update::{note::CreateOrUpdateNote, page::CreateOrUpdatePage},
    deletion::delete::Delete,
    following::{accept::AcceptFollow, follow::Follow, undo_follow::UndoFollow},
    voting::{undo_vote::UndoVote, vote::Vote},
  };
  use lemmy_apub_objects::utils::test::test_json;
  use lemmy_utils::error::LemmyResult;

  #[test]
  fn test_parse_smithereen_activities() -> LemmyResult<()> {
    test_json::<CreateOrUpdateNote>("../apub/assets/smithereen/activities/create_note.json")?;
    Ok(())
  }

  #[test]
  fn test_parse_pleroma_activities() -> LemmyResult<()> {
    test_json::<CreateOrUpdateNote>("../apub/assets/pleroma/activities/create_note.json")?;
    test_json::<Delete>("../apub/assets/pleroma/activities/delete.json")?;
    test_json::<Follow>("../apub/assets/pleroma/activities/follow.json")?;
    Ok(())
  }

  #[test]
  fn test_parse_mastodon_activities() -> LemmyResult<()> {
    test_json::<CreateOrUpdateNote>("../apub/assets/mastodon/activities/create_note.json")?;
    test_json::<Delete>("../apub/assets/mastodon/activities/delete.json")?;
    test_json::<Follow>("../apub/assets/mastodon/activities/follow.json")?;
    test_json::<UndoFollow>("../apub/assets/mastodon/activities/undo_follow.json")?;
    test_json::<Vote>("../apub/assets/mastodon/activities/like_page.json")?;
    test_json::<UndoVote>("../apub/assets/mastodon/activities/undo_like_page.json")?;
    test_json::<Report>("../apub/assets/mastodon/activities/flag.json")?;
    Ok(())
  }

  #[test]
  fn test_parse_lotide_activities() -> LemmyResult<()> {
    test_json::<Follow>("../apub/assets/lotide/activities/follow.json")?;
    test_json::<CreateOrUpdatePage>("../apub/assets/lotide/activities/create_page.json")?;
    test_json::<CreateOrUpdatePage>("../apub/assets/lotide/activities/create_page_image.json")?;
    test_json::<CreateOrUpdateNote>("../apub/assets/lotide/activities/create_note_reply.json")?;
    Ok(())
  }

  #[test]
  fn test_parse_friendica_activities() -> LemmyResult<()> {
    test_json::<CreateOrUpdatePage>("../apub/assets/friendica/activities/create_page_1.json")?;
    test_json::<CreateOrUpdatePage>("../apub/assets/friendica/activities/create_page_2.json")?;
    test_json::<CreateOrUpdateNote>("../apub/assets/friendica/activities/create_note.json")?;
    test_json::<CreateOrUpdateNote>("../apub/assets/friendica/activities/update_note.json")?;
    test_json::<Delete>("../apub/assets/friendica/activities/delete.json")?;
    test_json::<Vote>("../apub/assets/friendica/activities/like_page.json")?;
    test_json::<Vote>("../apub/assets/friendica/activities/dislike_page.json")?;
    test_json::<UndoVote>("../apub/assets/friendica/activities/undo_dislike_page.json")?;
    Ok(())
  }

  #[test]
  fn test_parse_gnusocial_activities() -> LemmyResult<()> {
    test_json::<CreateOrUpdatePage>("../apub/assets/gnusocial/activities/create_page.json")?;
    test_json::<CreateOrUpdateNote>("../apub/assets/gnusocial/activities/create_note.json")?;
    test_json::<Vote>("../apub/assets/gnusocial/activities/like_note.json")?;
    Ok(())
  }

  #[test]
  fn test_parse_peertube_activities() -> LemmyResult<()> {
    test_json::<AnnounceActivity>("../apub/assets/peertube/activities/announce_video.json")?;
    Ok(())
  }

  #[test]
  fn test_parse_mbin_activities() -> LemmyResult<()> {
    test_json::<AcceptFollow>("../apub/assets/mbin/activities/accept.json")?;
    test_json::<Report>("../apub/assets/mbin/activities/flag.json")?;
    Ok(())
  }

  #[test]
  fn test_parse_wordpress_activities() -> LemmyResult<()> {
    test_json::<AnnounceActivity>("../apub/assets/wordpress/activities/announce.json")?;
    Ok(())
  }
}
