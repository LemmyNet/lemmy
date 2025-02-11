use super::link_rule::Link;
use crate::{settings::SETTINGS, utils::markdown::link_rule};
use markdown_it::{
  parser::linkfmt::LinkFormatter,
  plugins::cmark::{
    block::fence,
    inline::{image, image::Image},
  },
  MarkdownIt,
  NodeValue,
};
use std::sync::LazyLock;
use url::Url;
use urlencoding::encode;

/// Rewrites all links to remote domains in markdown, so they go through `/api/v4/image_proxy`.
pub fn markdown_rewrite_image_links(mut src: String) -> (String, Vec<Url>) {
  let links_offsets = find_urls::<Image>(&src);

  let mut links = vec![];
  // Go through the collected links in reverse order
  for (start, end) in links_offsets.into_iter().rev() {
    let (url, extra) = markdown_handle_title(&src, start, end);
    match Url::parse(url) {
      Ok(parsed) => {
        links.push(parsed.clone());
        // If link points to remote domain, replace with proxied link
        if parsed.domain() != Some(&SETTINGS.hostname) {
          let mut proxied = format!(
            "{}/api/v4/image/proxy?url={}",
            SETTINGS.get_protocol_and_hostname(),
            encode(url),
          );
          // restore custom emoji format
          if let Some(extra) = extra {
            proxied.push(' ');
            proxied.push_str(extra);
          }
          src.replace_range(start..end, &proxied);
        }
      }
      Err(_) => {
        // If its not a valid url, replace with empty text
        src.replace_range(start..end, "");
      }
    }
  }

  (src, links)
}

pub fn markdown_handle_title(src: &str, start: usize, end: usize) -> (&str, Option<&str>) {
  let content = src.get(start..end).unwrap_or_default();
  // necessary for custom emojis which look like `![name](url "title")`
  match content.split_once(' ') {
    Some((a, b)) => (a, Some(b)),
    _ => (content, None),
  }
}

pub fn markdown_find_links(src: &str) -> Vec<(usize, usize)> {
  find_urls::<Link>(src)
}

// Walk the syntax tree to find positions of image or link urls
fn find_urls<T: NodeValue + UrlAndTitle>(src: &str) -> Vec<(usize, usize)> {
  // Use separate markdown parser here, with most features disabled for faster parsing,
  // and a dummy link formatter which doesnt normalize links.
  static PARSER: LazyLock<MarkdownIt> = LazyLock::new(|| {
    let mut p = MarkdownIt::new();
    p.link_formatter = Box::new(NoopLinkFormatter {});
    image::add(&mut p);
    fence::add(&mut p);
    link_rule::add(&mut p);
    p
  });

  let ast = PARSER.parse(src);
  let mut links_offsets = vec![];
  ast.walk(|node, _depth| {
    if let Some(image) = node.cast::<T>() {
      if let Some(srcmap) = node.srcmap {
        let (_, node_offset) = srcmap.get_byte_offsets();
        let start_offset = node_offset - image.url_len() - 1 - image.title_len();
        let end_offset = node_offset - 1;

        links_offsets.push((start_offset, end_offset));
      }
    }
  });
  links_offsets
}

pub trait UrlAndTitle {
  fn url_len(&self) -> usize;
  fn title_len(&self) -> usize;
}

impl UrlAndTitle for Image {
  fn url_len(&self) -> usize {
    self.url.len()
  }

  fn title_len(&self) -> usize {
    self.title.as_ref().map(|t| t.len() + 3).unwrap_or_default()
  }
}
impl UrlAndTitle for Link {
  fn url_len(&self) -> usize {
    self.url.len()
  }
  fn title_len(&self) -> usize {
    self.title.as_ref().map(|t| t.len() + 3).unwrap_or_default()
  }
}

/// markdown-it normalizes links by default, which breaks the link rewriting. So we use a dummy
/// formatter here which does nothing. Note this isnt actually used to render the markdown.
#[derive(Debug)]
struct NoopLinkFormatter;

impl LinkFormatter for NoopLinkFormatter {
  fn validate_link(&self, _url: &str) -> Option<()> {
    Some(())
  }

  fn normalize_link(&self, url: &str) -> String {
    url.to_owned()
  }

  fn normalize_link_text(&self, url: &str) -> String {
    url.to_owned()
  }
}

#[cfg(test)]
mod tests {

  use super::*;
  use pretty_assertions::assert_eq;

  #[test]
  fn test_find_links() {
    let links = markdown_find_links("[test](https://example.com)");
    assert_eq!(vec![(7, 26)], links);

    let links = find_urls::<Image>("![test](https://example.com)");
    assert_eq!(vec![(8, 27)], links);

    let links = find_urls::<Image>("![ითხოვს](https://example.com/ითხოვს)");
    assert_eq!(vec![(22, 60)], links);

    let links = find_urls::<Image>("![test](https://example.com/%C3%A4%C3%B6%C3%BC.jpg)");
    assert_eq!(vec![(8, 50)], links);
  }

  #[test]
  fn test_markdown_proxy_images() {
    let tests: Vec<_> =
      vec![
        (
          "remote image proxied",
          "![link](http://example.com/image.jpg)",
          "![link](https://lemmy-alpha/api/v4/image/proxy?url=http%3A%2F%2Fexample.com%2Fimage.jpg)",
        ),
        (
          "local image unproxied",
          "![link](http://lemmy-alpha/image.jpg)",
          "![link](http://lemmy-alpha/image.jpg)",
        ),
        (
          "multiple image links",
          "![link](http://example.com/image1.jpg) ![link](http://example.com/image2.jpg)",
          "![link](https://lemmy-alpha/api/v4/image/proxy?url=http%3A%2F%2Fexample.com%2Fimage1.jpg) ![link](https://lemmy-alpha/api/v4/image/proxy?url=http%3A%2F%2Fexample.com%2Fimage2.jpg)",
        ),
        (
          "empty link handled",
          "![image]()",
          "![image]()"
        ),
        (
          "empty label handled",
          "![](http://example.com/image.jpg)",
          "![](https://lemmy-alpha/api/v4/image/proxy?url=http%3A%2F%2Fexample.com%2Fimage.jpg)"
        ),
        (
          "invalid image link removed",
          "![image](http-not-a-link)",
          "![image]()"
        ),
        (
          "label with nested markdown handled",
          "![a *b* c](http://example.com/image.jpg)",
          "![a *b* c](https://lemmy-alpha/api/v4/image/proxy?url=http%3A%2F%2Fexample.com%2Fimage.jpg)"
        ),
        (
          "custom emoji support",
          r#"![party-blob](https://www.hexbear.net/pictrs/image/83405746-0620-4728-9358-5f51b040ffee.gif "emoji party-blob")"#,
          r#"![party-blob](https://lemmy-alpha/api/v4/image/proxy?url=https%3A%2F%2Fwww.hexbear.net%2Fpictrs%2Fimage%2F83405746-0620-4728-9358-5f51b040ffee.gif "emoji party-blob")"#
        ),
        (
          "image with special chars",
          "ითხოვს ![ითხოვს](http://example.com/ითხოვს%C3%A4.jpg)",
          "ითხოვს ![ითხოვს](https://lemmy-alpha/api/v4/image/proxy?url=http%3A%2F%2Fexample.com%2F%E1%83%98%E1%83%97%E1%83%AE%E1%83%9D%E1%83%95%E1%83%A1%25C3%25A4.jpg)",
        ),
      ];

    tests.iter().for_each(|&(msg, input, expected)| {
      let result = markdown_rewrite_image_links(input.to_string());

      assert_eq!(
        result.0, expected,
        "Testing {}, with original input '{}'",
        msg, input
      );
    });
  }
}
