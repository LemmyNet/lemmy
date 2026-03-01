use crate::utils::markdown::link_rule::Link;
use markdown_it::{
  MarkdownIt,
  Node,
  NodeValue,
  Renderer,
  parser::inline::{InlineRule, InlineState},
};

#[derive(Debug)]
pub struct Identifier {
  pub is_community: bool,
  pub name: String,
  pub domain: String,
}

impl NodeValue for Identifier {
  fn render(&self, node: &Node, fmt: &mut dyn Renderer) {
    let mut attrs = node.attrs.clone();
    let path = if self.is_community { 'c' } else { 'u' };
    attrs.push(("href", format!("/{path}/{}@{}", &self.name, &self.domain)));
    attrs.push(("rel", "nofollow".to_string()));
    attrs.push(("class", "u-url".to_string()));
    attrs.push(("class", "mention".to_string()));

    fmt.open("a", &attrs);
    let marker = if self.is_community { '!' } else { '@' };
    fmt.text(&format!("{marker}{}@{}", self.name, self.domain));
    fmt.close("a");
  }
}

struct CommunityIdentifierScanner;
struct PersonIdentifierScanner;

impl InlineRule for CommunityIdentifierScanner {
  const MARKER: char = '!';

  fn run(state: &mut InlineState) -> Option<(Node, usize)> {
    scan_for_identifier(true, Self::MARKER, state)
  }
}

impl InlineRule for PersonIdentifierScanner {
  const MARKER: char = '@';

  fn run(state: &mut InlineState) -> Option<(Node, usize)> {
    scan_for_identifier(false, Self::MARKER, state)
  }
}

fn scan_for_identifier(
  is_community: bool,
  marker: char,
  state: &mut InlineState,
) -> Option<(Node, usize)> {
  // Dont allow identifier inside link, otherwise it outputs nested `<a>` tags.
  if state.node.is::<Link>() {
    return None;
  }

  let Some(input) = &state.src.get(state.pos..state.pos_max) else {
    return None;
  };
  // wrong start character
  if !input.starts_with(marker) {
    return None;
  }

  let mut found_at = false;
  let mut name = String::new();
  let mut domain = String::new();
  for c in input.chars().skip(1) {
    // whitespace means we reached the end
    if c.is_whitespace() {
      break;
    }

    // we are inside a markdown link, ignore
    if c == ']' {
      return None;
    }

    // found the @ character between name and domain
    if c == '@' {
      found_at = true;
      continue;
    }
    if !found_at {
      name.push(c);
    } else {
      domain.push(c);
    }
  }

  // check if we found a valid, nonempty identifier
  (!name.is_empty() && !domain.is_empty()).then(|| {
    let len = name.len() + domain.len() + 2;
    let identifier = Identifier {
      is_community,
      name,
      domain,
    };
    (Node::new(identifier), len)
  })
}
pub fn add(md: &mut MarkdownIt) {
  md.inline.add_rule::<CommunityIdentifierScanner>();
  md.inline.add_rule::<PersonIdentifierScanner>();
}
