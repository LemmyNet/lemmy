// Custom Markdown plugin to manage spoilers.
//
// Matches the capability described in Lemmy UI:
// https://github.com/LemmyNet/lemmy-ui/blob/main/src/shared/utils.ts#L159
// that is based off of:
// https://github.com/markdown-it/markdown-it-container/tree/master#example
//
// FORMAT:
// Input Markdown: ::: spoiler THE_LEAD_IN\nTHE_HIDDEN_SPOILER\n:::\n
// Output HTML: <details><summary>THE_LEAD_IN</summary><p>THE_HIDDEN_SPOILER</p></details>

use markdown_it::{
  parser::{
    block::{BlockRule, BlockState},
    inline::InlineRoot,
  },
  MarkdownIt,
  Node,
  NodeValue,
  Renderer,
};
use once_cell::sync::Lazy;
use regex::Regex;

#[derive(Debug)]
struct SpoilerBlock {
  call_to_action: String,
}

const SPOILER_PREFIX: &str = "::: spoiler ";
const SPOILER_SUFFIX: &str = ":::";
const SPOILER_SUFFIX_NEWLINE: &str = ":::\n";

static SPOILER_REGEX: Lazy<Regex> =
  Lazy::new(|| Regex::new(r"^::: spoiler .*$").expect("compile spoiler markdown regex."));

impl NodeValue for SpoilerBlock {
  fn render(&self, node: &Node, fmt: &mut dyn Renderer) {
    fmt.cr();
    fmt.open("details", &node.attrs);
    fmt.open("summary", &[]);
    fmt.text(&self.call_to_action); // Not allowing special styling here to keep it simple.
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
  fn run(state: &mut BlockState) -> Option<(Node, usize)> {
    let first_line: &str = state.get_line(state.line).trim();

    // Check if the first line contains the spoiler syntax...
    if !SPOILER_REGEX.is_match(first_line) {
      return None;
    }

    let begin_fence_line_idx: usize = state.line + 1;
    let mut end_fence_line_idx: usize = begin_fence_line_idx;
    let mut has_end_marker: bool = false;

    // Search for the end of the spoiler; there could potentially be multiple lines between
    // the beginning and end of the block.
    //
    // Block ends with a line with ':::' or ':::\n'; it must be isolated from other markdown.
    while end_fence_line_idx < state.line_max && !has_end_marker {
      let next_line: &str = state.get_line(end_fence_line_idx).trim();

      if next_line.eq(SPOILER_SUFFIX) || next_line.eq(SPOILER_SUFFIX_NEWLINE) {
        has_end_marker = true;
        break;
      }

      end_fence_line_idx += 1;
    }

    if has_end_marker {
      let (spoiler_content, mapping) = state.get_lines(
        begin_fence_line_idx,
        end_fence_line_idx,
        state.blk_indent,
        true,
      );

      let mut node = Node::new(SpoilerBlock {
        call_to_action: String::from(first_line.replace(SPOILER_PREFIX, "").trim()),
      });

      node
        .children
        .push(Node::new(InlineRoot::new(spoiler_content, mapping)));

      // NOTE: Not using begin_fence_line_idx here because of incorrect results when state.line == 0
      //       (subtract an idx) vs the expected correct result (add an idx).
      Some((node, end_fence_line_idx - state.line + 1))
    } else {
      None
    }
  }
}

pub fn add(markdown_parser: &mut MarkdownIt) {
  markdown_parser.block.add_rule::<SpoilerBlockScanner>();
}
