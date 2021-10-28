use crate::{fetcher::object_id::ObjectId, objects::person::ApubPerson, protocol::Source};
use activitystreams::{
  chrono::{DateTime, FixedOffset},
  unparsed::Unparsed,
};
use anyhow::anyhow;
use lemmy_apub_lib::{values::MediaTypeHtml, verify::verify_domains_match};
use lemmy_utils::LemmyError;
use lemmy_websocket::LemmyContext;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
use url::Url;

#[skip_serializing_none]
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ChatMessage {
  pub(crate) r#type: ChatMessageType,
  pub(crate) id: Url,
  pub(crate) attributed_to: ObjectId<ApubPerson>,
  pub(crate) to: [ObjectId<ApubPerson>; 1],
  pub(crate) content: String,
  pub(crate) media_type: Option<MediaTypeHtml>,
  pub(crate) source: Option<Source>,
  pub(crate) published: Option<DateTime<FixedOffset>>,
  pub(crate) updated: Option<DateTime<FixedOffset>>,
  #[serde(flatten)]
  pub(crate) unparsed: Unparsed,
}

/// https://docs.pleroma.social/backend/development/ap_extensions/#chatmessages
#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum ChatMessageType {
  ChatMessage,
}

impl ChatMessage {
  pub(crate) fn id_unchecked(&self) -> &Url {
    &self.id
  }
  pub(crate) fn id(&self, expected_domain: &Url) -> Result<&Url, LemmyError> {
    verify_domains_match(&self.id, expected_domain)?;
    Ok(&self.id)
  }

  pub(crate) async fn verify(
    &self,
    context: &LemmyContext,
    request_counter: &mut i32,
  ) -> Result<(), LemmyError> {
    verify_domains_match(self.attributed_to.inner(), &self.id)?;
    let person = self
      .attributed_to
      .dereference(context, request_counter)
      .await?;
    if person.banned {
      return Err(anyhow!("Person is banned from site").into());
    }
    Ok(())
  }
}
