use crate::{
  objects::{community::ApubCommunity, person::ApubPerson, post::ApubPost},
  protocol::{ImageObject, Source},
};
use activitystreams::{object::kind::PageType, unparsed::Unparsed};
use anyhow::anyhow;
use chrono::{DateTime, FixedOffset};
use lemmy_apub_lib::{
  data::Data,
  object_id::ObjectId,
  traits::ActivityHandler,
  values::MediaTypeHtml,
};
use lemmy_utils::LemmyError;
use lemmy_websocket::LemmyContext;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
use url::Url;

#[skip_serializing_none]
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Page {
  pub(crate) r#type: PageType,
  pub(crate) id: ObjectId<ApubPost>,
  pub(crate) attributed_to: ObjectId<ApubPerson>,
  pub(crate) to: Vec<Url>,
  #[serde(default)]
  pub(crate) cc: Vec<Url>,
  pub(crate) name: String,
  pub(crate) content: Option<String>,
  pub(crate) media_type: Option<MediaTypeHtml>,
  pub(crate) source: Option<Source>,
  pub(crate) url: Option<Url>,
  pub(crate) image: Option<ImageObject>,
  pub(crate) comments_enabled: Option<bool>,
  pub(crate) sensitive: Option<bool>,
  pub(crate) stickied: Option<bool>,
  pub(crate) published: Option<DateTime<FixedOffset>>,
  pub(crate) updated: Option<DateTime<FixedOffset>>,
  #[serde(flatten)]
  pub(crate) unparsed: Unparsed,
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
    let mut to_iter = self.to.iter();
    loop {
      if let Some(cid) = to_iter.next() {
        let cid = ObjectId::new(cid.clone());
        if let Ok(c) = cid.dereference(context, request_counter).await {
          break Ok(c);
        }
      } else {
        return Err(anyhow!("No community found in cc").into());
      }
    }
  }
}

// For Pleroma/Mastodon compat. Unimplemented because its only used for sending.
#[async_trait::async_trait(?Send)]
impl ActivityHandler for Page {
  type DataType = LemmyContext;
  async fn verify(&self, _: &Data<Self::DataType>, _: &mut i32) -> Result<(), LemmyError> {
    Err(anyhow!("Announce/Page can only be sent, not received").into())
  }
  async fn receive(self, _: &Data<Self::DataType>, _: &mut i32) -> Result<(), LemmyError> {
    unimplemented!()
  }
}
