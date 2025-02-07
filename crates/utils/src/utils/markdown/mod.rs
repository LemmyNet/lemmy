use crate::error::{LemmyErrorType, LemmyResult};
use markdown_it::MarkdownIt;
use regex::RegexSet;
use std::sync::LazyLock;

pub mod image_links;
mod link_rule;

static MARKDOWN_PARSER: LazyLock<MarkdownIt> = LazyLock::new(|| {
  let mut parser = MarkdownIt::new();
  markdown_it::plugins::cmark::add(&mut parser);
  markdown_it::plugins::extra::add(&mut parser);
  markdown_it_block_spoiler::add(&mut parser);
  markdown_it_sub::add(&mut parser);
  markdown_it_sup::add(&mut parser);
  markdown_it_ruby::add(&mut parser);
  markdown_it_footnote::add(&mut parser);
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

pub fn markdown_check_for_blocked_urls(text: &str, blocklist: &RegexSet) -> LemmyResult<()> {
  if blocklist.is_match(text) {
    Err(LemmyErrorType::BlockedUrl)?
  }
  Ok(())
}

#[cfg(test)]
mod tests {

  use super::*;
  use crate::utils::validation::check_urls_are_valid;
  use pretty_assertions::assert_eq;
  use regex::escape;

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
        "<details><summary>click to see more</summary>how spicy!\n</details>\n"
      ),
      (
        "escape html special chars",
        "<script>alert('xss');</script> hello &\"",
        "<p>&lt;script&gt;alert(‘xss’);&lt;/script&gt; hello &amp;&quot;</p>\n"
      ),("subscript","log~2~(a)","<p>log<sub>2</sub>(a)</p>\n"),
      (
        "superscript",
        "Markdown^TM^",
        "<p>Markdown<sup>TM</sup></p>\n"
      ),
      (
        "ruby text",
        "{漢|Kan}{字|ji}",
        "<p><ruby>漢<rp>(</rp><rt>Kan</rt><rp>)</rp></ruby><ruby>字<rp>(</rp><rt>ji</rt><rp>)</rp></ruby></p>\n"
      ),
      (
        "footnotes",
        "Bold claim.[^1]\n\n[^1]: example.com",
        "<p>Bold claim.<sup class=\"footnote-ref\"><a href=\"#fn1\" id=\"fnref1\">[1]</a></sup></p>\n\
	 <hr class=\"footnotes-sep\" />\n\
	 <section class=\"footnotes\">\n\
	 <ol class=\"footnotes-list\">\n\
	 <li id=\"fn1\" class=\"footnote-item\">\n\
	 <p>example.com <a href=\"#fnref1\" class=\"footnote-backref\">↩︎</a></p>\n\
	 </li>\n</ol>\n</section>\n"
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

  // This replicates the logic when saving url blocklist patterns and querying them.
  // Refer to lemmy_api_crud::site::update::update_site and
  // lemmy_api_common::utils::get_url_blocklist().
  fn create_url_blocklist_test_regex_set(patterns: Vec<&str>) -> LemmyResult<RegexSet> {
    let url_blocklist = patterns.iter().map(|&s| s.to_string()).collect();
    let valid_urls = check_urls_are_valid(&url_blocklist)?;
    let regexes = valid_urls.iter().map(|p| format!(r"\b{}\b", escape(p)));
    let set = RegexSet::new(regexes)?;
    Ok(set)
  }

  #[test]
  fn test_url_blocking() -> LemmyResult<()> {
    let set = create_url_blocklist_test_regex_set(vec!["example.com/"])?;

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

    let set = create_url_blocklist_test_regex_set(vec!["example.com/spam.jpg"])?;
    assert!(markdown_check_for_blocked_urls("![](https://example.com/spam.jpg)", &set).is_err());
    assert!(markdown_check_for_blocked_urls("![](https://example.com/spam.jpg1)", &set).is_ok());
    // TODO: the following should not be matched, scunthorpe problem.
    assert!(
      markdown_check_for_blocked_urls("![](https://example.com/spam.jpg.html)", &set).is_err()
    );

    let set = create_url_blocklist_test_regex_set(vec![
      r"quo.example.com/",
      r"foo.example.com/",
      r"bar.example.com/",
    ])?;

    assert!(markdown_check_for_blocked_urls("https://baz.example.com", &set).is_ok());

    assert!(markdown_check_for_blocked_urls("https://bar.example.com", &set).is_err());

    let set = create_url_blocklist_test_regex_set(vec!["example.com/banned_page"])?;

    assert!(markdown_check_for_blocked_urls("https://example.com/page", &set).is_ok());

    let set = create_url_blocklist_test_regex_set(vec!["ex.mple.com/"])?;

    assert!(markdown_check_for_blocked_urls("example.com", &set).is_ok());

    let set = create_url_blocklist_test_regex_set(vec!["rt.com/"])?;

    assert!(markdown_check_for_blocked_urls("deviantart.com", &set).is_ok());
    assert!(markdown_check_for_blocked_urls("art.com.example.com", &set).is_ok());
    assert!(markdown_check_for_blocked_urls("https://rt.com/abc", &set).is_err());
    assert!(markdown_check_for_blocked_urls("go to rt.com.", &set).is_err());
    assert!(markdown_check_for_blocked_urls("check out rt.computer", &set).is_ok());
    // TODO: the following should not be matched, scunthorpe problem.
    assert!(markdown_check_for_blocked_urls("rt.com.example.com", &set).is_err());

    Ok(())
  }

  #[test]
  fn test_sanitize_html() {
    let sanitized = sanitize_html("<script>alert('xss');</script> hello &\"'");
    let expected = "&lt;script>alert(&#x27;xss&#x27;);&lt;/script> hello &amp;&quot;&#x27;";
    assert_eq!(expected, sanitized);

    let sanitized =
      sanitize_html("Polling the group: what do y'all know about the Orion browser from Kagi?");
    let expected = "Polling the group: what do y&#x27;all know about the Orion browser from Kagi?";
    assert_eq!(expected, sanitized);
  }
}
