// Custom Markdown plugin to manage spoilers.
//
// Matches the capability described in Lemmy UI:
// https://github.com/LemmyNet/lemmy-ui/blob/main/src/shared/utils.ts#L159
// that is based off of:
// https://github.com/markdown-it/markdown-it-container/tree/master#example
//
// FORMAT:
// Input Markdown: ::: spoiler VISIBLE_TEXT\nHIDDEN_SPOILER\n:::\n
// Output HTML: <details><summary>VISIBLE_TEXT</summary><p>nHIDDEN_SPOILER</p></details>
//
// Anatomy of a spoiler:
//     keyword
//        ^
// ::: spoiler VISIBLE_HINT
//  ^                ^
// begin fence   visible text
//
// HIDDEN_SPOILER
//      ^
//  hidden text
//
// :::
//  ^
// end fence

use markdown_it::{
  parser::{
    block::{BlockRule, BlockState},
    inline::InlineRoot,
  },
  MarkdownIt, Node, NodeValue, Renderer,
};
use once_cell::sync::Lazy;
use regex::Regex;

#[derive(Debug)]
struct SpoilerBlock {
  visible_text: String,
}

const SPOILER_PREFIX: &str = "::: spoiler ";
const SPOILER_SUFFIX: &str = ":::";
const SPOILER_SUFFIX_NEWLINE: &str = ":::\n";

static SPOILER_REGEX: Lazy<Regex> =
  Lazy::new(|| Regex::new(r"^::: spoiler .*$").expect("compile spoiler markdown regex."));

impl NodeValue for SpoilerBlock {
  // Formats any node marked as a 'SpoilerBlock' into HTML.
  // See the SpoilerBlockScanner#run implementation to see how these nodes get added to the tree.
  fn render(&self, node: &Node, fmt: &mut dyn Renderer) {
    fmt.cr();
    fmt.open("details", &node.attrs);
    fmt.open("summary", &[]);
    // Not allowing special styling to the visible text to keep it simple.
    // If allowed, would need to parse the child nodes to assign to visible vs hidden text sections.
    fmt.text(&self.visible_text);
    fmt.close("summary");
    fmt.open("p", &[]);
    fmt.contents(&node.children);
    fmt.close("p");
    fmt.close("details");
    fmt.cr();
  }
}

struct SpoilerBlockScanner;

impl BlockRule for SpoilerBlockScanner {
  // Invoked on every line in the provided Markdown text to check if the BlockRule applies.
  //
  // NOTE: This does NOT support nested spoilers at this time.
  fn run(state: &mut BlockState) -> Option<(Node, usize)> {
    let first_line: &str = state.get_line(state.line).trim();

    // 1. Check if the first line contains the spoiler syntax...
    if !SPOILER_REGEX.is_match(first_line) {
      return None;
    }

    let begin_spoiler_line_idx: usize = state.line + 1;
    let mut end_fence_line_idx: usize = begin_spoiler_line_idx;
    let mut has_end_fence: bool = false;

    // 2. Search for the end of the spoiler and find the index of the last line of the spoiler.
    // There could potentially be multiple lines between the beginning and end of the block.
    //
    // Block ends with a line with ':::' or ':::\n'; it must be isolated from other markdown.
    while end_fence_line_idx < state.line_max && !has_end_fence {
      let next_line: &str = state.get_line(end_fence_line_idx).trim();

      if next_line.eq(SPOILER_SUFFIX) || next_line.eq(SPOILER_SUFFIX_NEWLINE) {
        has_end_fence = true;
        break;
      }

      end_fence_line_idx += 1;
    }

    // 3. If available, construct and return the spoiler node to add to the tree.
    if has_end_fence {
      let (spoiler_content, mapping) = state.get_lines(
        begin_spoiler_line_idx,
        end_fence_line_idx,
        state.blk_indent,
        true,
      );

      let mut node = Node::new(SpoilerBlock {
        visible_text: String::from(first_line.replace(SPOILER_PREFIX, "").trim()),
      });

      // Add the spoiler content as children; marking as a child tells the tree to process the
      // node again, which means other Markdown syntax (ex: emphasis, links) can be rendered.
      node
        .children
        .push(Node::new(InlineRoot::new(spoiler_content, mapping)));

      // NOTE: Not using begin_spoiler_line_idx here because of incorrect results when
      //       state.line == 0 (subtracts an idx) vs the expected correct result (adds an idx).
      Some((node, end_fence_line_idx - state.line + 1))
    } else {
      None
    }
  }
}

pub fn add(markdown_parser: &mut MarkdownIt) {
  markdown_parser.block.add_rule::<SpoilerBlockScanner>();
}

#[cfg(test)]
mod tests {
  #![allow(clippy::unwrap_used)]
  #![allow(clippy::indexing_slicing)]

  use crate::utils::markdown::spoiler_rule::add;
  use markdown_it::MarkdownIt;

  #[test]
  fn test_spoiler_markdown() {
    let tests: Vec<_> = vec![
      (
        "invalid spoiler",
        "::: spoiler click to see more\nbut I never finished",
        "<p>::: spoiler click to see more\nbut I never finished</p>\n",
      ),
      (
        "another invalid spoiler",
        "::: spoiler\nnever added the lead in\n:::",
        "<p>::: spoiler\nnever added the lead in\n:::</p>\n",
      ),
      (
        "basic spoiler, but no newline at the end",
        "::: spoiler click to see more\nhow spicy!\n:::",
        "<details><summary>click to see more</summary><p>how spicy!\n</p></details>\n"
      ),
      (
        "basic spoiler with a newline at the end",
        "::: spoiler click to see more\nhow spicy!\n:::\n",
        "<details><summary>click to see more</summary><p>how spicy!\n</p></details>\n"
      ),
      (
        "spoiler with extra markdown on the call to action (no extra parsing)",
        "::: spoiler _click to see more_\nhow spicy!\n:::\n",
        "<details><summary>_click to see more_</summary><p>how spicy!\n</p></details>\n"
      ),
      (
        "spoiler with extra markdown in the fenced spoiler block",
        "::: spoiler click to see more\n**how spicy!**\n*i have many lines*\n:::\n",
        "<details><summary>click to see more</summary><p><strong>how spicy!</strong>\n<em>i have many lines</em>\n</p></details>\n"
      ),
      (
        "spoiler mixed with other content",
        "hey you\npsst, wanna hear a secret?\n::: spoiler lean in and i'll tell you\n**you are breathtaking!**\n:::\nwhatcha think about that?",
        "<p>hey you\npsst, wanna hear a secret?</p>\n<details><summary>lean in and i'll tell you</summary><p><strong>you are breathtaking!</strong>\n</p></details>\n<p>whatcha think about that?</p>\n"
      ),
      (
        "spoiler mixed with indented content",
        "- did you know that\n::: spoiler the call was\n***coming from inside the house!***\n:::\n - crazy, right?",
        "<ul>\n<li>did you know that</li>\n</ul>\n<details><summary>the call was</summary><p><em><strong>coming from inside the house!</strong></em>\n</p></details>\n<ul>\n<li>crazy, right?</li>\n</ul>\n"
      )
    ];

    tests.iter().for_each(|&(msg, input, expected)| {
      let md = &mut MarkdownIt::new();
      markdown_it::plugins::cmark::add(md);
      add(md);

      assert_eq!(
        md.parse(input).xrender(),
        expected,
        "Testing {}, with original input '{}'",
        msg,
        input
      );
    });
  }
}
