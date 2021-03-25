use activitystreams::unparsed::UnparsedMutExt;
use activitystreams_ext::UnparsedExtension;
use lemmy_utils::LemmyError;
use serde::{Deserialize, Serialize};

/// Activitystreams extension to allow (de)serializing additional Person field
/// `also_known_as` (used for Matrix profile link).
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PersonExtension {
  pub matrix_user_id: Option<String>,
}

impl PersonExtension {
  pub fn new(matrix_user_id: Option<String>) -> Result<PersonExtension, LemmyError> {
    Ok(PersonExtension { matrix_user_id })
  }
}

impl<U> UnparsedExtension<U> for PersonExtension
where
  U: UnparsedMutExt,
{
  type Error = serde_json::Error;

  fn try_from_unparsed(unparsed_mut: &mut U) -> Result<Self, Self::Error> {
    Ok(PersonExtension {
      matrix_user_id: unparsed_mut.remove("matrix_user_id")?,
    })
  }

  fn try_into_unparsed(self, unparsed_mut: &mut U) -> Result<(), Self::Error> {
    unparsed_mut.insert("matrix_user_id", self.matrix_user_id)?;
    Ok(())
  }
}
