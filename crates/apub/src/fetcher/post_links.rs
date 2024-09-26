use crate::fetcher::post_or_comment::PostOrComment;
use activitypub_federation::{config::Data, fetch::object_id::ObjectId};
use lemmy_api_common::{
  context::LemmyContext,
  utils::{generate_local_apub_endpoint, EndpointType},
};
use lemmy_utils::{
  error::LemmyResult,
  utils::markdown::image_links::{markdown_find_links, markdown_handle_title},
};

pub async fn markdown_rewrite_remote_post_links_opt(
  src: Option<String>,
  context: &Data<LemmyContext>,
) -> Option<String> {
  match src {
    Some(t) => Some(markdown_rewrite_remote_post_links(t, context).await),
    None => None,
  }
}

/// TODO: as it uses ObjectId::dereference, it can currently only be used in apub crate
pub async fn markdown_rewrite_remote_post_links(
  mut src: String,
  context: &Data<LemmyContext>,
) -> String {
  let links_offsets = markdown_find_links(&src);
  let domain = &context.settings().get_protocol_and_hostname();

  // Go through the collected links in reverse order
  for (start, end) in links_offsets.into_iter().rev() {
    let (url, extra) = markdown_handle_title(&src, start, end);

    // TODO: call this logic for post.url, comment.text etc
    // TODO: needs cleanup
    // TODO: also resolve user and community links
    if let Ok(parsed) = ObjectId::<PostOrComment>::parse(url) {
      if parsed.inner().domain() != Some(&context.settings().hostname) {
        let dereferenced = parsed.dereference(context).await;

        if let Some(mut local_url) = to_local_url(dereferenced, &domain) {
          // restore title
          if let Some(extra) = extra {
            local_url = format!("{local_url} {extra}");
          }
          src.replace_range(start..end, local_url.as_str());
        }
      }
    }
  }

  src
}

fn to_local_url(dereferenced: LemmyResult<PostOrComment>, domain: &str) -> Option<String> {
  dereferenced
    .map(|d| match d {
      PostOrComment::Post(post) => {
        generate_local_apub_endpoint(EndpointType::Post, &post.id.to_string(), domain).ok()
      }
      PostOrComment::Comment(comment) => {
        generate_local_apub_endpoint(EndpointType::Comment, &comment.id.to_string(), domain).ok()
      }
    })
    .ok()
    .flatten()
    .map(|e| e.to_string())
}

#[cfg(test)]
#[expect(clippy::unwrap_used)]
mod tests {

  use super::*;
  use pretty_assertions::assert_eq;

  #[tokio::test]
  async fn test_markdown_rewrite_remote_post_links() {
    let tests: Vec<_> = vec![
      (
        "rewrite remote link",
        "[link](https://feddit.org/post/3172593)",
        "[link](https://lemmy-alpha/post/1)",
      ),
      (
        "dont rewrite local link",
        "[link](https://lemmy-alpha/post/2)",
        "[link](https://lemmy-alpha/post/2)",
      ),
      (
        "dont rewrite non-fediverse link",
        "[link](https://example.com/)",
        "[link](https://example.com/)",
      ),
      (
        "dont rewrite invalid url",
        "[link](example-com)",
        "[link](example-com)",
      ),
    ];

    let context = LemmyContext::init_test_context().await;
    for &(msg, input, expected) in &tests {
      let result = markdown_rewrite_remote_post_links(input.to_string(), &context).await;

      assert_eq!(
        result, expected,
        "Testing {}, with original input '{}'",
        msg, input
      );
    }
  }
}
