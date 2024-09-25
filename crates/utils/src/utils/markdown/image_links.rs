use super::MARKDOWN_PARSER;
use crate::settings::SETTINGS;
use markdown_it::plugins::cmark::inline::image::Image;
use url::Url;
use urlencoding::encode;

/// Rewrites all links to remote domains in markdown, so they go through `/api/v3/image_proxy`.
pub fn markdown_rewrite_image_links(mut src: String) -> (String, Vec<Url>) {
  let ast = MARKDOWN_PARSER.parse(&src);
  let mut links_offsets = vec![];

  // Walk the syntax tree to find positions of image links
  ast.walk(|node, _depth| {
    if let Some(image) = node.cast::<Image>() {
      // srcmap is always present for image
      // https://github.com/markdown-it-rust/markdown-it/issues/36#issuecomment-1777844387
      let node_offsets = node.srcmap.expect("srcmap is none").get_byte_offsets();
      // necessary for custom emojis which look like `![name](url "title")`
      let start_offset = node_offsets.1
        - image.url.len()
        - 1
        - image
          .title
          .as_ref()
          .map(|t| t.len() + 3)
          .unwrap_or_default();
      let end_offset = node_offsets.1 - 1;

      links_offsets.push((start_offset, end_offset));
    }
  });

  let mut links = vec![];
  // Go through the collected links in reverse order
  while let Some((start, end)) = links_offsets.pop() {
    let content = src.get(start..end).unwrap_or_default();
    // necessary for custom emojis which look like `![name](url "title")`
    let (url, extra) = if content.contains(' ') {
      let split = content.split_once(' ').expect("split is valid");
      (split.0, Some(split.1))
    } else {
      (content, None)
    };
    match Url::parse(url) {
      Ok(parsed) => {
        links.push(parsed.clone());
        // If link points to remote domain, replace with proxied link
        if parsed.domain() != Some(&SETTINGS.hostname) {
          let mut proxied = format!(
            "{}/api/v3/image_proxy?url={}",
            SETTINGS.get_protocol_and_hostname(),
            encode(url),
          );
          // restore custom emoji format
          if let Some(extra) = extra {
            proxied = format!("{proxied} {extra}");
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

#[cfg(test)]
#[expect(clippy::unwrap_used)]
mod tests {

  use super::*;
  use pretty_assertions::assert_eq;

  #[test]
  fn test_markdown_proxy_images() {
    let tests: Vec<_> =
      vec![
        (
          "remote image proxied",
          "![link](http://example.com/image.jpg)",
          "![link](https://lemmy-alpha/api/v3/image_proxy?url=http%3A%2F%2Fexample.com%2Fimage.jpg)",
        ),
        (
          "local image unproxied",
          "![link](http://lemmy-alpha/image.jpg)",
          "![link](http://lemmy-alpha/image.jpg)",
        ),
        (
          "multiple image links",
          "![link](http://example.com/image1.jpg) ![link](http://example.com/image2.jpg)",
          "![link](https://lemmy-alpha/api/v3/image_proxy?url=http%3A%2F%2Fexample.com%2Fimage1.jpg) ![link](https://lemmy-alpha/api/v3/image_proxy?url=http%3A%2F%2Fexample.com%2Fimage2.jpg)",
        ),
        (
          "empty link handled",
          "![image]()",
          "![image]()"
        ),
        (
          "empty label handled",
          "![](http://example.com/image.jpg)",
          "![](https://lemmy-alpha/api/v3/image_proxy?url=http%3A%2F%2Fexample.com%2Fimage.jpg)"
        ),
        (
          "invalid image link removed",
          "![image](http-not-a-link)",
          "![image]()"
        ),
        (
          "label with nested markdown handled",
          "![a *b* c](http://example.com/image.jpg)",
          "![a *b* c](https://lemmy-alpha/api/v3/image_proxy?url=http%3A%2F%2Fexample.com%2Fimage.jpg)"
        ),
        (
          "custom emoji support",
          r#"![party-blob](https://www.hexbear.net/pictrs/image/83405746-0620-4728-9358-5f51b040ffee.gif "emoji party-blob")"#,
          r#"![party-blob](https://lemmy-alpha/api/v3/image_proxy?url=https%3A%2F%2Fwww.hexbear.net%2Fpictrs%2Fimage%2F83405746-0620-4728-9358-5f51b040ffee.gif "emoji party-blob")"#
        )
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
