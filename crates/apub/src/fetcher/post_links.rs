use crate::fetcher::post_or_comment::PostOrComment;
use activitypub_federation::{config::Data, fetch::object_id::ObjectId};
use lemmy_api_common::{
  context::LemmyContext,
  utils::{generate_local_apub_endpoint, EndpointType},
};
use lemmy_utils::utils::markdown::image_links::{markdown_find_links, markdown_handle_title};
use url::Url;

pub async fn markdown_rewrite_remote_post_links_opt(
  src: Option<String>,
  context: &Data<LemmyContext>,
) -> Option<String> {
  match src {
    Some(t) => Some(markdown_rewrite_remote_post_links(t, context).await),
    None => None,
  }
}

// TODO: call this logic for comment.text etc
/// TODO: as it uses ObjectId::dereference, it can currently only be used in apub crate
pub async fn markdown_rewrite_remote_post_links(
  mut src: String,
  context: &Data<LemmyContext>,
) -> String {
  let links_offsets = markdown_find_links(&src);

  // Go through the collected links in reverse order
  for (start, end) in links_offsets.into_iter().rev() {
    let (url, extra) = markdown_handle_title(&src, start, end);

    // TODO: needs cleanup

    if let Some(local_url) = to_local_url(url, context).await {
      let mut local_url = local_url.to_string();
      // restore title
      if let Some(extra) = extra {
        local_url = format!("{local_url} {extra}");
      }
      src.replace_range(start..end, local_url.as_str());
    }
  }

  src
}

// TODO: also resolve user and community links
pub(crate) async fn to_local_url(url: &str, context: &Data<LemmyContext>) -> Option<Url> {
  let local_domain = &context.settings().get_protocol_and_hostname();
  let object_id = ObjectId::<PostOrComment>::parse(url).ok()?;
  if object_id.inner().domain() == Some(local_domain) {
    return None;
  }
  let dereferenced = object_id.dereference(context).await;
  dereferenced
    .map(|d| match d {
      PostOrComment::Post(post) => {
        generate_local_apub_endpoint(EndpointType::Post, &post.id.to_string(), local_domain).ok()
      }
      PostOrComment::Comment(comment) => {
        generate_local_apub_endpoint(EndpointType::Comment, &comment.id.to_string(), local_domain)
          .ok()
      }
    })
    .ok()
    .flatten()
    .map(std::convert::Into::into)
}

#[cfg(test)]
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
