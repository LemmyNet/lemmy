use super::clean_urls_in_text;
use markdown_it::{
  MarkdownIt,
  plugins::cmark::{
    block::{code, fence},
    inline::backticks,
  },
};
use std::sync::LazyLock;

pub fn clean_urls_skip_code_links(src: &str) -> String {
  static PARSER: LazyLock<MarkdownIt> = LazyLock::new(|| {
    let mut p = MarkdownIt::new();
    fence::add(&mut p);
    code::add(&mut p);
    backticks::add(&mut p);
    p
  });
  let ast = PARSER.parse(src);

  let mut code_offsets: Vec<(usize, usize)> = Vec::new();
  //we need to exclude code fences (``` ``` and ~~~ ~~~) as well as bacticks inline code (` `) and code blocks (4 spaces)
  ast.walk(|node, _| {
    if (node.cast::<fence::CodeFence>().is_some()
      || node.cast::<code::CodeBlock>().is_some()
      || node.cast::<backticks::CodeInline>().is_some())
      && let Some(srcmap) = node.srcmap
    {
      code_offsets.push(srcmap.get_byte_offsets());
    }
  });
  let mut output_string = String::with_capacity(src.len());
  let mut index = 0;
  for (code_start, code_end) in code_offsets {
    if let Some(slice_before_code) = src.get(index..code_start) {
      output_string.push_str(&clean_urls_in_text(slice_before_code));
    }
    if let Some(slice_code) = src.get(code_start..code_end) {
      output_string.push_str(slice_code);
    }
    index = code_end;
  }

  // if no fences or bacticks => same as using pure clean_urls_in_text()
  if let Some(slice_after_code) = src.get(index..) {
    output_string.push_str(&clean_urls_in_text(slice_after_code));
  };
  output_string
}

#[cfg(test)]
mod tests {
  use super::clean_urls_skip_code_links;
  use pretty_assertions::assert_eq;

  const SOURCE: &str = r#"
https://example.com/path/123?utm_content=buffercf3b2&utm_medium=social&user+name=random+user&id=123

[link](https://example.com/path/123?utm_content=buffercf3b2&utm_medium=social&user+name=random+user&id=123)

```javascript
const url = `https://example.com?foo=${bar}`
```

`const url = "https://example.com?foo=${bar}"`

    const url = `https://example.com?foo=${bar}`
"#;

  const SOURCE_LINKS_CLEANED: &str = r#"
https://example.com/path/123?user+name=random+user&id=123

[link](https://example.com/path/123?user+name=random+user&id=123)

```javascript
const url = `https://example.com?foo=${bar}`
```

`const url = "https://example.com?foo=${bar}"`

    const url = `https://example.com?foo=${bar}`
"#;

  #[test]
  fn test_clean_urls_skip_code_links() {
    let cleaned = clean_urls_skip_code_links(SOURCE);
    assert_eq!(SOURCE_LINKS_CLEANED.to_string(), cleaned);
  }
}
