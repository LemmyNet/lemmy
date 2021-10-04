//! The enums here serve to limit a json string value to a single, hardcoded value which can be
//! verified at compilation time. When using it as the type of a struct field, the struct can only
//! be constructed or deserialized if the field has the exact same value.
//!
//! If we used String as the field type, any value would be accepted, and we would have to check
//! manually at runtime that it contains the expected value.
//!
//! The enums in [`activitystreams::activity::kind`] work in the same way, and can be used to
//! distinguish different activity types.
//!
//! In the example below, `MyObject` can only be constructed or
//! deserialized if `media_type` is `text/markdown`, but not if it is `text/html`.
//!
//! ```
//! use lemmy_apub_lib::values::MediaTypeMarkdown;
//! use serde_json::from_str;
//! use serde::{Deserialize, Serialize};
//!
//! #[derive(Deserialize, Serialize)]
//! struct MyObject {
//!   content: String,
//!   media_type: MediaTypeMarkdown,
//! }
//!
//! let markdown_json = r#"{"content": "**test**", "media_type": "text/markdown"}"#;
//! let from_markdown = from_str::<MyObject>(markdown_json);
//! assert!(from_markdown.is_ok());
//!
//! let markdown_html = r#"{"content": "<b>test</b>", "media_type": "text/html"}"#;
//! let from_html = from_str::<MyObject>(markdown_html);
//! assert!(from_html.is_err());
//! ```

use serde::{Deserialize, Serialize};

/// The identifier used to address activities to the public.
///
/// <https://www.w3.org/TR/activitypub/#public-addressing>
#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum PublicUrl {
  #[serde(rename = "https://www.w3.org/ns/activitystreams#Public")]
  Public,
}

/// Media type for markdown text.
///
/// <https://www.iana.org/assignments/media-types/media-types.xhtml>
#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum MediaTypeMarkdown {
  #[serde(rename = "text/markdown")]
  Markdown,
}

/// Media type for HTML text/
///
/// <https://www.iana.org/assignments/media-types/media-types.xhtml>
#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum MediaTypeHtml {
  #[serde(rename = "text/html")]
  Html,
}
