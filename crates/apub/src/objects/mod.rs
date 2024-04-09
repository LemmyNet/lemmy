use crate::protocol::Source;
use activitypub_federation::{
  config::Data,
  fetch::object_id::ObjectId,
  protocol::values::MediaTypeMarkdownOrHtml,
};
use anyhow::anyhow;
use html2md::parse_html;
use lemmy_api_common::context::LemmyContext;
use lemmy_utils::error::LemmyError;
use std::fmt::Debug;

pub mod comment;
pub mod community;
pub mod instance;
pub mod person;
pub mod post;
pub mod private_message;

pub(crate) fn read_from_string_or_source(
  content: &str,
  media_type: &Option<MediaTypeMarkdownOrHtml>,
  source: &Option<Source>,
) -> String {
  if let Some(s) = source {
    // markdown sent by lemmy in source field
    s.content.clone()
  } else if media_type == &Some(MediaTypeMarkdownOrHtml::Markdown) {
    // markdown sent by peertube in content field
    content.to_string()
  } else {
    // otherwise, convert content html to markdown
    parse_html(content)
  }
}

pub(crate) fn read_from_string_or_source_opt(
  content: &Option<String>,
  media_type: &Option<MediaTypeMarkdownOrHtml>,
  source: &Option<Source>,
) -> Option<String> {
  content
    .as_ref()
    .map(|content| read_from_string_or_source(content, media_type, source))
}

/// When for example a Post is made in a remote community, the community will send it back,
/// wrapped in Announce. If we simply receive this like any other federated object, overwrite the
/// existing, local Post. In particular, it will set the field local = false, so that the object
/// can't be fetched from the Activitypub HTTP endpoint anymore (which only serves local objects).
pub(crate) fn verify_is_remote_object<T>(
  id: &ObjectId<T>,
  context: &Data<LemmyContext>,
) -> Result<(), LemmyError>
where
  T: activitypub_federation::traits::Object<DataType = LemmyContext> + Debug + Send + 'static,
  for<'de2> <T as activitypub_federation::traits::Object>::Kind: serde::Deserialize<'de2>,
{
  if !id.is_local(context) {
    Err(anyhow!("cant accept local object from remote instance").into())
  } else {
    Ok(())
  }
}
