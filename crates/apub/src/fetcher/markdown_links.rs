use super::{search::SearchableObjects, user_or_community::UserOrCommunity};
use crate::fetcher::post_or_comment::PostOrComment;
use activitypub_federation::{config::Data, fetch::object_id::ObjectId};
use lemmy_api_common::{
  context::LemmyContext,
  utils::{generate_local_apub_endpoint, EndpointType},
};
use lemmy_db_schema::{newtypes::InstanceId, source::instance::Instance};
use lemmy_utils::{
  error::LemmyResult,
  utils::markdown::image_links::{markdown_find_links, markdown_handle_title},
};
use url::Url;

pub async fn markdown_rewrite_remote_links_opt(
  src: Option<String>,
  context: &Data<LemmyContext>,
) -> Option<String> {
  match src {
    Some(t) => Some(markdown_rewrite_remote_links(t, context).await),
    None => None,
  }
}

/// Goes through all remote markdown links and attempts to resolve them as Activitypub objects.
/// If successful, the link is rewritten to a local link, so it can be viewed without leaving the
/// local instance.
///
/// As it relies on ObjectId::dereference, it can only be used for incoming federated objects, not
/// for the API.
pub async fn markdown_rewrite_remote_links(
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

pub(crate) async fn to_local_url(url: &str, context: &Data<LemmyContext>) -> Option<Url> {
  let local_domain = &context.settings().get_protocol_and_hostname();
  let object_id = ObjectId::<SearchableObjects>::parse(url).ok()?;
  if object_id.inner().domain() == Some(local_domain) {
    return None;
  }
  let dereferenced = object_id.dereference(context).await.ok()?;
  match dereferenced {
    SearchableObjects::PostOrComment(pc) => match *pc {
      PostOrComment::Post(post) => {
        generate_local_apub_endpoint(EndpointType::Post, &post.id.to_string(), local_domain)
      }
      PostOrComment::Comment(comment) => {
        generate_local_apub_endpoint(EndpointType::Comment, &comment.id.to_string(), local_domain)
      }
    }
    .ok()
    .map(Into::into),
    SearchableObjects::PersonOrCommunity(pc) => match *pc {
      UserOrCommunity::User(user) => {
        format_actor_url(&user.name, "u", user.instance_id, context).await
      }
      UserOrCommunity::Community(community) => {
        format_actor_url(&community.name, "c", community.instance_id, context).await
      }
    }
    .ok(),
  }
}

async fn format_actor_url(
  name: &str,
  kind: &str,
  instance_id: InstanceId,
  context: &LemmyContext,
) -> LemmyResult<Url> {
  let local_protocol_and_hostname = context.settings().get_protocol_and_hostname();
  let local_hostname = &context.settings().hostname;
  let instance = Instance::read(&mut context.pool(), instance_id).await?;
  let url = if dbg!(&instance.domain) != dbg!(local_hostname) {
    format!(
      "{local_protocol_and_hostname}/{kind}/{name}@{}",
      instance.domain
    )
  } else {
    format!("{local_protocol_and_hostname}/{kind}/{name}")
  };
  Ok(Url::parse(&url)?)
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
        "rewrite community link",
        "[link](https://feddit.org/c/dach)",
        "[link](https://lemmy-alpha/c/dach@feddit.org)",
      ),
      (
        "dont rewrite local post link",
        "[link](https://lemmy-alpha/post/2)",
        "[link](https://lemmy-alpha/post/2)",
      ),
      (
        "dont rewrite local community link",
        "[link](https://lemmy-alpha/c/test)",
        "[link](https://lemmy-alpha/c/test)",
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
      let result = markdown_rewrite_remote_links(input.to_string(), &context).await;

      assert_eq!(
        result, expected,
        "Testing {}, with original input '{}'",
        msg, input
      );
    }
  }
}
