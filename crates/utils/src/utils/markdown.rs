use markdown_it::MarkdownIt;

pub mod spoiler_rule;

pub fn markdown_to_html(text: &str) -> String {
  let md = &mut MarkdownIt::new();
  markdown_it::plugins::cmark::add(md);
  markdown_it::plugins::extra::add(md);
  spoiler_rule::add(md);

  md.parse(text).render()
}

#[cfg(test)]
mod tests {
  use crate::utils::markdown::markdown_to_html;

  #[test]
  fn test_markdown() {
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
        "basic blockquotes",
        ">Nice quote",
        "<blockquote>\n<p>Nice quote</p>\n</blockquote>\n"),
      (
        "blockquotes with multiple paragraphs",
        "> Hello\n > \n > Goodbye",
        "<blockquote>\n<p>Hello</p>\n<p>Goodbye</p>\n</blockquote>\n"
      ),
      (
        "nested blockquotes",
        "> Hello\n > \n >> Goodbye",
        "<blockquote>\n<p>Hello</p>\n<blockquote>\n<p>Goodbye</p>\n</blockquote>\n</blockquote>\n"
      ),
      (
        "blockquotes with other elements",
        "> #### Hello\n > \n > - Hola\n > - 안영\n",
        "<blockquote>\n<h4>Hello</h4>\n<ul>\n<li>Hola</li>\n<li>안영</li>\n</ul>\n</blockquote>\n"
      ),
      (
        "ordered lists",
        "1. pen\n2. apple\n3. apple pen",
        "<ol>\n<li>pen</li>\n<li>apple</li>\n<li>apple pen</li>\n</ol>\n"
      ),
      (
        "unordered lists",
        "- pen\n- apple\n- apple pen",
        "<ul>\n<li>pen</li>\n<li>apple</li>\n<li>apple pen</li>\n</ul>\n"
      ),
      (
        "code",
        "this is my amazing `code snippet`",
        "<p>this is my amazing <code>code snippet</code></p>\n"
      ),
      (
        "code block",
        "this is my amazing ```code block```",
        "<p>this is my amazing <code>code block</code></p>\n"
      ),
      (
        "links",
        "[Lemmy](https://join-lemmy.org/ \"Join Lemmy!\")",
        "<p><a href=\"https://join-lemmy.org/\" title=\"Join Lemmy!\">Lemmy</a></p>\n"
      ),
      (
        "images",
        "![My linked image](https://image.com \"image alt text\")",
        "<p><img src=\"https://image.com\" alt=\"My linked image\" title=\"image alt text\"></p>\n"
      ),
      (
        "basic spoiler content, but no newline at the end",
        "::: spoiler click to see more\nhow spicy!\n:::",
        "<details><summary>click to see more</summary><p>how spicy!\n</p></details>\n"
      ),
      (
        "basic spoiler content with a newline at the end.",
        "::: spoiler click to see more\nhow spicy!\n:::\n",
        "<details><summary>click to see more</summary><p>how spicy!\n</p></details>\n"
      ),
      (
        "spoiler with extra markdown on the call to action. No special parsing will be done.",
        "::: spoiler _click to see more_\nhow spicy!\n:::\n",
        "<details><summary>_click to see more_</summary><p>how spicy!\n</p></details>\n"
      ),
      (
        "spoiler with extra markdown in the fenced spoiler block.",
        "::: spoiler click to see more\n**how spicy!**\n*i have many lines*\n:::\n",
        "<details><summary>click to see more</summary><p><strong>how spicy!</strong>\n<em>i have many lines</em>\n</p></details>\n"
      ),
      (
        "spoiler mixed with other content.",
        "hey you\npsst, wanna hear a secret?\n::: spoiler lean in and i'll tell you\n**you are breathtaking!**\n:::\nwhatcha think about that?",
        "<p>hey you\npsst, wanna hear a secret?</p>\n<details><summary>lean in and i'll tell you</summary><p><strong>you are breathtaking!</strong>\n</p></details>\n<p>whatcha think about that?</p>\n"
      ),
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
}
