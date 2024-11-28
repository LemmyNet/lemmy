use super::comment::ApubComment;
use crate::{
  objects::private_message::ApubPrivateMessage,
  protocol::objects::note_wrapper::NoteWrapper,
};
use activitypub_federation::{config::Data, kinds::public, traits::Object};
use chrono::{DateTime, Utc};
use lemmy_api_common::{context::LemmyContext, LemmyErrorType};
use lemmy_utils::error::{LemmyError, LemmyResult};
use serde_json::{from_value, to_value};
use url::Url;

/// Private messages and public comments are quite awkward in Activitypub, because the json
/// format looks identical. They only way to differentiate them is to check for the presence
/// or absence of `https://www.w3.org/ns/activitystreams#Public` in `to` or `cc` which this
/// wrapper does.
#[derive(Debug)]
pub(crate) struct ApubNote {}

#[async_trait::async_trait]
impl Object for ApubNote {
  type DataType = LemmyContext;
  type Kind = NoteWrapper;
  type Error = LemmyError;

  fn last_refreshed_at(&self) -> Option<DateTime<Utc>> {
    None
  }

  #[tracing::instrument(skip_all)]
  async fn read_from_id(
    _object_id: Url,
    _context: &Data<Self::DataType>,
  ) -> LemmyResult<Option<Self>> {
    Err(LemmyErrorType::Unknown("not implemented".to_string()).into())
  }

  #[tracing::instrument(skip_all)]
  async fn delete(self, _context: &Data<Self::DataType>) -> LemmyResult<()> {
    Err(LemmyErrorType::Unknown("not implemented".to_string()).into())
  }

  async fn verify(
    note: &NoteWrapper,
    expected_domain: &Url,
    context: &Data<LemmyContext>,
  ) -> LemmyResult<()> {
    let val = to_value(note)?;
    if is_public(&note.to, &note.cc) {
      ApubComment::verify(&from_value(val)?, expected_domain, context).await?;
    } else {
      ApubPrivateMessage::verify(&from_value(val)?, expected_domain, context).await?;
    }
    Ok(())
  }

  async fn from_json(note: NoteWrapper, context: &Data<LemmyContext>) -> LemmyResult<ApubNote> {
    let is_public = is_public(&note.to, &note.cc);
    let val = to_value(note)?;
    if is_public {
      ApubComment::from_json(from_value(val)?, context).await?;
    } else {
      ApubPrivateMessage::from_json(from_value(val)?, context).await?;
    }
    Ok(ApubNote {})
  }

  async fn into_json(self, _context: &Data<Self::DataType>) -> LemmyResult<NoteWrapper> {
    Err(LemmyErrorType::Unknown("not implemented".to_string()).into())
  }
}

pub(crate) fn is_public(to: &Option<Vec<Url>>, cc: &Option<Vec<Url>>) -> bool {
  if let Some(to) = to {
    if to.contains(&public()) {
      return true;
    }
  }
  if let Some(cc) = cc {
    if cc.contains(&public()) {
      return true;
    }
  }
  false
}
