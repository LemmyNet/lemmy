use crate::{
  objects::{community::ApubCommunity, person::ApubPerson, post::ApubPost},
  protocol::{ImageObject, SourceCompat},
};
use chrono::{DateTime, FixedOffset};
use itertools::Itertools;
use lemmy_apub_lib::{
  data::Data,
  object_id::ObjectId,
  traits::{ActivityHandler, ApubObject},
  values::MediaTypeHtml,
};
use lemmy_utils::LemmyError;
use lemmy_websocket::LemmyContext;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
use url::Url;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum PageType {
  Page,
  Note,
}

#[skip_serializing_none]
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Page {
  pub(crate) r#type: PageType,
  pub(crate) id: ObjectId<ApubPost>,
  pub(crate) attributed_to: ObjectId<ApubPerson>,
  #[serde(deserialize_with = "crate::deserialize_one_or_many")]
  pub(crate) to: Vec<Url>,
  pub(crate) name: String,

  #[serde(default)]
  #[serde(deserialize_with = "crate::deserialize_one_or_many")]
  pub(crate) cc: Vec<Url>,
  pub(crate) content: Option<String>,
  pub(crate) media_type: Option<MediaTypeHtml>,
  pub(crate) source: Option<SourceCompat>,
  pub(crate) url: Option<Url>,
  pub(crate) image: Option<ImageObject>,
  pub(crate) comments_enabled: Option<bool>,
  pub(crate) sensitive: Option<bool>,
  pub(crate) stickied: Option<bool>,
  pub(crate) published: Option<DateTime<FixedOffset>>,
  pub(crate) updated: Option<DateTime<FixedOffset>>,
}

impl Page {
  /// Only mods can change the post's stickied/locked status. So if either of these is changed from
  /// the current value, it is a mod action and needs to be verified as such.
  ///
  /// Both stickied and locked need to be false on a newly created post (verified in [[CreatePost]].
  pub(crate) async fn is_mod_action(&self, context: &LemmyContext) -> Result<bool, LemmyError> {
    let old_post = ObjectId::<ApubPost>::new(self.id.clone())
      .dereference_local(context)
      .await;

    let is_mod_action = if let Ok(old_post) = old_post {
      self.stickied != Some(old_post.stickied) || self.comments_enabled != Some(!old_post.locked)
    } else {
      false
    };
    Ok(is_mod_action)
  }

  pub(crate) async fn extract_community(
    &self,
    context: &LemmyContext,
    request_counter: &mut i32,
  ) -> Result<ApubCommunity, LemmyError> {
    let mut iter = self.to.iter().merge(self.cc.iter());
    loop {
      if let Some(cid) = iter.next() {
        let cid = ObjectId::new(cid.clone());
        if let Ok(c) = cid
          .dereference(context, context.client(), request_counter)
          .await
        {
          break Ok(c);
        }
      } else {
        return Err(LemmyError::from_message("No community found in cc"));
      }
    }
  }
}

// Used for community outbox, so that it can be compatible with Pleroma/Mastodon.
#[async_trait::async_trait(?Send)]
impl ActivityHandler for Page {
  type DataType = LemmyContext;
  async fn verify(
    &self,
    data: &Data<Self::DataType>,
    request_counter: &mut i32,
  ) -> Result<(), LemmyError> {
    ApubPost::verify(self, self.id.inner(), data, request_counter).await
  }
  async fn receive(
    self,
    data: &Data<Self::DataType>,
    request_counter: &mut i32,
  ) -> Result<(), LemmyError> {
    ApubPost::from_apub(self, data, request_counter).await?;
    Ok(())
  }
}
