use markdown_it::generics::inline::full_link;
use markdown_it::{MarkdownIt, Node, NodeValue, Renderer};

/// Renders markdown links. Copied directly from markdown-it source, unlike original code it also
/// sets `rel=nofollow` attribute.
///
/// TODO: We can set nofollow only if post was not made by mod/admin, but then we have to construct
///       new parser for every invocation which might have performance implications.
/// https://github.com/markdown-it-rust/markdown-it/blob/master/src/plugins/cmark/inline/link.rs
#[derive(Debug)]
pub struct Link {
    pub url: String,
    pub title: Option<String>,
}

impl NodeValue for Link {
    fn render(&self, node: &Node, fmt: &mut dyn Renderer) {
        let mut attrs = node.attrs.clone();
        attrs.push(("href", self.url.clone()));
        attrs.push(("rel", "nofollow".to_string()));

        if let Some(title) = &self.title {
            attrs.push(("title", title.clone()));
        }

        fmt.open("a", &attrs);
        fmt.contents(&node.children);
        fmt.close("a");
    }
}

pub fn add(md: &mut MarkdownIt) {
    full_link::add::<false>(md, |href, title| Node::new(Link {
        url: href.unwrap_or_default(),
        title,
    }));
}