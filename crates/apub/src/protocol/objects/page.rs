use crate::{
  activities::{verify_is_public, verify_person_in_community},
  fetcher::object_id::ObjectId,
  objects::{community::ApubCommunity, person::ApubPerson, post::ApubPost},
  protocol::{ImageObject, Source},
};
use activitystreams::{object::kind::PageType, unparsed::Unparsed};
use anyhow::anyhow;
use chrono::{DateTime, FixedOffset};
use lemmy_apub_lib::{values::MediaTypeHtml, verify::verify_domains_match};
use lemmy_utils::{utils::check_slurs, LemmyError};
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

  pub(crate) async fn verify(
    &self,
    context: &LemmyContext,
    request_counter: &mut i32,
  ) -> Result<(), LemmyError> {
    let community = self.extract_community(context, request_counter).await?;

    check_slurs(&self.name, &context.settings().slur_regex())?;
    verify_domains_match(self.attributed_to.inner(), self.id.inner())?;
    verify_person_in_community(&self.attributed_to, &community, context, request_counter).await?;
    verify_is_public(&self.to.clone())?;
    Ok(())
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
