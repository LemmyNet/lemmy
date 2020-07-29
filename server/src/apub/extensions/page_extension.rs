use activitystreams_ext::UnparsedExtension;
use activitystreams_new::unparsed::UnparsedMutExt;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PageExtension {
  pub comments_enabled: bool,
  pub sensitive: bool,
  pub stickied: bool,
}

impl<U> UnparsedExtension<U> for PageExtension
where
  U: UnparsedMutExt,
{
  type Error = serde_json::Error;

  fn try_from_unparsed(unparsed_mut: &mut U) -> Result<Self, Self::Error> {
    Ok(PageExtension {
      comments_enabled: unparsed_mut.remove("commentsEnabled")?,
      sensitive: unparsed_mut.remove("sensitive")?,
      stickied: unparsed_mut.remove("stickied")?,
    })
  }

  fn try_into_unparsed(self, unparsed_mut: &mut U) -> Result<(), Self::Error> {
    unparsed_mut.insert("commentsEnabled", self.comments_enabled)?;
    unparsed_mut.insert("sensitive", self.sensitive)?;
    unparsed_mut.insert("stickied", self.stickied)?;
    Ok(())
  }
}
