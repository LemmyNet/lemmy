use crate::{error::LemmyResult, settings::SETTINGS, LemmyErrorType};
use markdown_it::{plugins::cmark::inline::image::Image, MarkdownIt};
use once_cell::sync::Lazy;
use regex::RegexSet;
use url::Url;
use urlencoding::encode;

mod link_rule;
mod spoiler_rule;

static MARKDOWN_PARSER: Lazy<MarkdownIt> = Lazy::new(|| {
  let mut parser = MarkdownIt::new();
  markdown_it::plugins::cmark::add(&mut parser);
  markdown_it::plugins::extra::add(&mut parser);
  spoiler_rule::add(&mut parser);
  link_rule::add(&mut parser);

  parser
});

/// Replace special HTML characters in API parameters to prevent XSS attacks.
///
/// Taken from https://github.com/OWASP/CheatSheetSeries/blob/master/cheatsheets/Cross_Site_Scripting_Prevention_Cheat_Sheet.md#output-encoding-for-html-contexts
///
/// `>` is left in place because it is interpreted as markdown quote.
pub fn sanitize_html(text: &str) -> String {
  text
    .replace('&', "&amp;")
    .replace('<', "&lt;")
    .replace('\"', "&quot;")
    .replace('\'', "&#x27;")
}

pub fn markdown_to_html(text: &str) -> String {
  MARKDOWN_PARSER.parse(text).xrender()
}

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

pub fn markdown_check_for_blocked_urls(text: &str, blocklist: &RegexSet) -> LemmyResult<()> {
  if blocklist.is_match(text) {
    Err(LemmyErrorType::BlockedUrl)?
  }
  Ok(())
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
#[allow(clippy::indexing_slicing)]
mod tests {

  use super::*;
  use pretty_assertions::assert_eq;

  #[test]
  fn test_basic_markdown() {
    let tests: Vec<_> = vec![
      (
        "headings",
        "# h1\n## h2\n### h3\n#### h4\n##### h5\n###### h6",
        "<h1>h1</h1>\n<h2>h2</h2>\n<h3>h3</h3>\n<h4>h4</h4>\n<h5>h5</h5>\n<h6>h6</h6>\n"
      ),
      (
        "line breaks",
        "First\rSecond",
        "<p>First\nSecond</p>\n"),
      (
        "emphasis",
        "__bold__ **bold** *italic* ***bold+italic***",
        "<p><strong>bold</strong> <strong>bold</strong> <em>italic</em> <em><strong>bold+italic</strong></em></p>\n"
      ),
      (
        "blockquotes",
        "> #### Hello\n > \n > - Hola\n > - 안영 \n>> Goodbye\n",
        "<blockquote>\n<h4>Hello</h4>\n<ul>\n<li>Hola</li>\n<li>안영</li>\n</ul>\n<blockquote>\n<p>Goodbye</p>\n</blockquote>\n</blockquote>\n"
      ),
      (
        "lists (ordered, unordered)",
        "1. pen\n2. apple\n3. apple pen\n- pen\n- pineapple\n- pineapple pen",
        "<ol>\n<li>pen</li>\n<li>apple</li>\n<li>apple pen</li>\n</ol>\n<ul>\n<li>pen</li>\n<li>pineapple</li>\n<li>pineapple pen</li>\n</ul>\n"
      ),
      (
        "code and code blocks",
        "this is my amazing `code snippet` and my amazing ```code block```",
        "<p>this is my amazing <code>code snippet</code> and my amazing <code>code block</code></p>\n"
      ),
      // Links with added nofollow attribute
      (
        "links",
        "[Lemmy](https://join-lemmy.org/ \"Join Lemmy!\")",
        "<p><a href=\"https://join-lemmy.org/\" rel=\"nofollow\" title=\"Join Lemmy!\">Lemmy</a></p>\n"
      ),
      // Remote images with proxy
      (
        "images",
        "![My linked image](https://example.com/image.png \"image alt text\")",
        "<p><img src=\"https://example.com/image.png\" alt=\"My linked image\" title=\"image alt text\" /></p>\n"
      ),
      // Local images without proxy
      (
        "images",
        "![My linked image](https://lemmy-alpha/image.png \"image alt text\")",
        "<p><img src=\"https://lemmy-alpha/image.png\" alt=\"My linked image\" title=\"image alt text\" /></p>\n"
      ),
      // Ensure spoiler plugin is added
      (
        "basic spoiler",
        "::: spoiler click to see more\nhow spicy!\n:::\n",
        "<details><summary>click to see more</summary><p>how spicy!\n</p></details>\n"
      ),
      (
        "escape html special chars",
        "<script>alert('xss');</script> hello &\"",
        "<p>&lt;script&gt;alert(‘xss’);&lt;/script&gt; hello &amp;&quot;</p>\n"
      )
    ];

    tests.iter().for_each(|&(msg, input, expected)| {
      let result = markdown_to_html(input);

      assert_eq!(
        result, expected,
        "Testing {}, with original input '{}'",
        msg, input
      );
    });
  }

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

  #[test]
  fn test_url_blocking() {
    let set = RegexSet::new(vec![r"(https://)?example\.com/?"]).unwrap();

    assert!(
      markdown_check_for_blocked_urls(&String::from("[](https://example.com)"), &set).is_err()
    );

    assert!(markdown_check_for_blocked_urls(
      &String::from("Go to https://example.com to get free Robux"),
      &set
    )
    .is_err());

    assert!(
      markdown_check_for_blocked_urls(&String::from("[](https://example.blog)"), &set).is_ok()
    );

    assert!(markdown_check_for_blocked_urls(&String::from("example.com"), &set).is_err());

    assert!(markdown_check_for_blocked_urls(
      "Odio exercitationem culpa sed sunt
      et. Sit et similique tempora deserunt doloremque. Cupiditate iusto
      repellat et quis qui. Cum veritatis facere quasi repellendus sunt
      eveniet nemo sint. Cumque sit unde est. https://example.com Alias
      repellendus at quos.",
      &set
    )
    .is_err());

    let set = RegexSet::new(vec![r"(https://)?example\.com/spam\.jpg"]).unwrap();
    assert!(markdown_check_for_blocked_urls(
      &String::from("![](https://example.com/spam.jpg)"),
      &set
    )
    .is_err());

    let set = RegexSet::new(vec![
      r"(https://)?quo\.example\.com/?",
      r"(https://)?foo\.example\.com/?",
      r"(https://)?bar\.example\.com/?",
    ])
    .unwrap();

    assert!(
      markdown_check_for_blocked_urls(&String::from("https://baz.example.com"), &set).is_ok()
    );

    assert!(
      markdown_check_for_blocked_urls(&String::from("https://bar.example.com"), &set).is_err()
    );

    let set = RegexSet::new(vec![r"(https://)?example\.com/banned_page"]).unwrap();

    assert!(
      markdown_check_for_blocked_urls(&String::from("https://example.com/page"), &set).is_ok()
    );

    let set = RegexSet::new(vec![r"(https://)?ex\.mple\.com/?"]).unwrap();

    assert!(markdown_check_for_blocked_urls("example.com", &set).is_ok());
  }

  #[test]
  fn test_sanitize_html() {
    let sanitized = sanitize_html("<script>alert('xss');</script> hello &\"'");
    let expected = "&lt;script>alert(&#x27;xss&#x27;);&lt;/script> hello &amp;&quot;&#x27;";
    assert_eq!(expected, sanitized)
  }
}
