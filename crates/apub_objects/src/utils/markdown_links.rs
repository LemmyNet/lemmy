use crate::objects::{PostOrComment, SearchableObjects, UserOrCommunity};
use activitypub_federation::{config::Data, fetch::object_id::ObjectId};
use lemmy_api_common::context::LemmyContext;
use lemmy_db_schema::traits::ApubActor;
use lemmy_utils::utils::markdown::image_links::{markdown_find_links, markdown_handle_title};
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

    if let Some(local_url) = to_local_url(url, context).await {
      let mut local_url = local_url.to_string();
      // restore title
      if let Some(extra) = extra {
        local_url.push(' ');
        local_url.push_str(extra);
      }
      src.replace_range(start..end, local_url.as_str());
    }
  }

  src
}

pub(crate) async fn to_local_url(url: &str, context: &Data<LemmyContext>) -> Option<Url> {
  let local_domain = &context.settings().get_protocol_and_hostname();
  let object_id = ObjectId::<SearchableObjects>::parse(url).ok()?;
  let object_domain = object_id.inner().domain();
  if object_domain == Some(local_domain) {
    return None;
  }
  let dereferenced = object_id.dereference_local(context).await.ok()?;
  match dereferenced {
    SearchableObjects::Left(pc) => match pc {
      PostOrComment::Left(post) => post.local_url(context.settings()),
      PostOrComment::Right(comment) => comment.local_url(context.settings()),
    }
    .ok()
    .map(Into::into),
    SearchableObjects::Right(pc) => match pc {
      UserOrCommunity::Left(user) => user.actor_url(context.settings()),
      UserOrCommunity::Right(community) => community.actor_url(context.settings()),
    }
    .ok(),
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use lemmy_db_schema::{
    source::{
      community::{Community, CommunityInsertForm},
      instance::Instance,
      post::{Post, PostInsertForm},
    },
    traits::Crud,
  };
  use lemmy_db_views_local_user::LocalUserView;
  use lemmy_db_views_site::impls::create_test_instance;
  use lemmy_utils::error::LemmyResult;
  use pretty_assertions::assert_eq;
  use serial_test::serial;

  #[serial]
  #[tokio::test]
  async fn test_markdown_rewrite_remote_links() -> LemmyResult<()> {
    let context = LemmyContext::init_test_context().await;
    let instance = create_test_instance(&mut context.pool()).await?;
    let community_form = CommunityInsertForm {
      ap_id: Some(Url::parse("https://example.com/c/my_community")?.into()),
      ..CommunityInsertForm::new(
        instance.id,
        "my_community".to_string(),
        "My Community".to_string(),
        "pubkey".to_string(),
      )
    };
    let community = Community::create(&mut context.pool(), &community_form).await?;
    let user =
      LocalUserView::create_test_user(&mut context.pool(), "garda", "garda bio", false).await?;

    // insert a remote post which is already fetched
    let post_form = PostInsertForm {
      ap_id: Some(Url::parse("https://example.com/post/123")?.into()),
      ..PostInsertForm::new("My post".to_string(), user.person.id, community.id)
    };
    let post = Post::create(&mut context.pool(), &post_form).await?;
    let markdown_local_post_url = format!("[link](https://lemmy-alpha/post/{})", post.id);

    let tests: Vec<_> = vec![
      (
        "rewrite remote post link",
        format!("[link]({})", post.ap_id),
        markdown_local_post_url.as_ref(),
      ),
      (
        "rewrite community link",
        format!("[link]({})", community.ap_id),
        "[link](https://lemmy-alpha/c/my_community@example.com)",
      ),
      (
        "dont rewrite local post link",
        "[link](https://lemmy-alpha/post/2)".to_string(),
        "[link](https://lemmy-alpha/post/2)",
      ),
      (
        "dont rewrite local community link",
        "[link](https://lemmy-alpha/c/test)".to_string(),
        "[link](https://lemmy-alpha/c/test)",
      ),
      (
        "dont rewrite non-fediverse link",
        "[link](https://example.com/)".to_string(),
        "[link](https://example.com/)",
      ),
      (
        "dont rewrite invalid url",
        "[link](example-com)".to_string(),
        "[link](example-com)",
      ),
    ];

    let context = LemmyContext::init_test_context().await;
    for (msg, input, expected) in &tests {
      let result = markdown_rewrite_remote_links(input.to_string(), &context).await;

      assert_eq!(
        &result, expected,
        "Testing {}, with original input '{}'",
        msg, input
      );
    }

    Instance::delete(&mut context.pool(), instance.id).await?;

    Ok(())
  }
}
