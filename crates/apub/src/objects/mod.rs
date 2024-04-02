use crate::protocol::Source;
use activitypub_federation::protocol::values::MediaTypeMarkdownOrHtml;
use anyhow::anyhow;
use chrono::{DateTime, Utc};
use html2md::parse_html;
use lemmy_utils::{error::LemmyError, settings::structs::Settings};
use url::Url;

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
pub(crate) fn verify_is_remote_object(id: &Url, settings: &Settings) -> Result<(), LemmyError> {
  let local_domain = settings.get_hostname_without_port()?;
  if id.domain() == Some(&local_domain) {
    Err(anyhow!("cant accept local object from remote instance").into())
  } else {
    Ok(())
  }
}

/// When receiving a federated object, check that the timestamp is newer than the latest version stored
/// locally. Necessary to reject edits which are received in wrong order.
pub(crate) fn verify_object_timestamp(
  old_timestamp: Option<DateTime<Utc>>,
  new_timestamp: Option<DateTime<Utc>>,
) -> Result<(), LemmyError> {
  if let (Some(old_timestamp), Some(new_timestamp)) = (old_timestamp, new_timestamp) {
    if new_timestamp < old_timestamp {
      return Err(anyhow!("Ignoring old object edit").into());
    }
  }
  Ok(())
}

#[cfg(test)]
pub(crate) mod tests {
  use super::*;
  use chrono::TimeZone;

  #[test]
  fn test_verify_object_timestamp() {
    let old = Some(Utc.with_ymd_and_hms(2024, 1, 1, 0, 0, 0).unwrap());
    let new = Some(Utc.with_ymd_and_hms(2024, 2, 1, 0, 0, 0).unwrap());

    assert!(verify_object_timestamp(old, new).is_ok());
    assert!(verify_object_timestamp(None, new).is_ok());
    assert!(verify_object_timestamp(old, None).is_ok());
    assert!(verify_object_timestamp(new, old).is_err());
  }
}
