use activitystreams::unparsed::UnparsedMutExt;
use activitystreams_ext::UnparsedExtension;
use lemmy_utils::LemmyError;
use serde::{Deserialize, Serialize};

/// Activitystreams extension to allow (de)serializing additional Community field
/// `sensitive` (called 'nsfw' in Lemmy).
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GroupExtension {
  pub sensitive: Option<bool>,
}

impl GroupExtension {
  pub fn new(sensitive: bool) -> Result<GroupExtension, LemmyError> {
    Ok(GroupExtension {
      sensitive: Some(sensitive),
    })
  }
}

impl<U> UnparsedExtension<U> for GroupExtension
where
  U: UnparsedMutExt,
{
  type Error = serde_json::Error;

  fn try_from_unparsed(unparsed_mut: &mut U) -> Result<Self, Self::Error> {
    Ok(GroupExtension {
      sensitive: unparsed_mut.remove("sensitive")?,
    })
  }

  fn try_into_unparsed(self, unparsed_mut: &mut U) -> Result<(), Self::Error> {
    unparsed_mut.insert("sensitive", self.sensitive)?;
    Ok(())
  }
}
