use crate::settings::SETTINGS;
use markdown_it::{generics::inline::full_link, MarkdownIt, Node, NodeValue, Renderer};
use url::Url;
use urlencoding::encode;

/// Renders markdown images. Copied directly from markdown-it source. It rewrites remote image URLs
/// to go through local proxy.
///
/// https://github.com/markdown-it-rust/markdown-it/blob/master/src/plugins/cmark/inline/image.rs
#[derive(Debug)]
pub struct Image {
  pub url: String,
  pub title: Option<String>,
}

impl NodeValue for Image {
  fn render(&self, node: &Node, fmt: &mut dyn Renderer) {
    let mut attrs = node.attrs.clone();

    // TODO: error handling

    let url = Url::parse(&self.url).unwrap();

    // Rewrite remote links to go through proxy
    let url = if url.domain().unwrap() != SETTINGS.hostname {
      let url = encode(&self.url);
      format!(
        "{}/api/v3/image_proxy?url={}",
        SETTINGS.get_protocol_and_hostname(),
        url
      )
    } else {
      self.url.clone()
    };
    attrs.push(("src", url));
    attrs.push(("alt", node.collect_text()));

    if let Some(title) = &self.title {
      attrs.push(("title", title.clone()));
    }

    fmt.self_close("img", &attrs);
  }
}

pub fn add(md: &mut MarkdownIt) {
  full_link::add_prefix::<'!', true>(md, |href, title| {
    Node::new(Image {
      url: href.unwrap_or_default(),
      title,
    })
  });
}
