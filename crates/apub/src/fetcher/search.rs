use activitypub_federation::{
  config::Data,
  fetch::{object_id::ObjectId, webfinger::webfinger_resolve_actor},
};
use lemmy_api_utils::context::LemmyContext;
use lemmy_apub_objects::objects::{SearchableObjects, UserOrCommunity};
use lemmy_utils::error::LemmyResult;
use url::Url;

/// Converts search query to object id. The query can either be an URL, which will be treated as
/// ObjectId directly, or a webfinger identifier (@user@example.com or !community@example.com)
/// which gets resolved to an URL.
pub(crate) async fn search_query_to_object_id(
  mut query: String,
  context: &Data<LemmyContext>,
) -> LemmyResult<SearchableObjects> {
  Ok(match Url::parse(&query) {
    Ok(url) => {
      // its already an url, just go with it
      ObjectId::from(url).dereference(context).await?
    }
    Err(_) => {
      // not an url, try to resolve via webfinger
      if query.starts_with('!') || query.starts_with('@') {
        query.remove(0);
      }
      SearchableObjects::Right(
        webfinger_resolve_actor::<LemmyContext, UserOrCommunity>(&query, context).await?,
      )
    }
  })
}

/// Converts a search query to an object id.  The query MUST bbe a URL which will bbe treated
/// as the ObjectId directly.  If the query is a webfinger identifier (@user@example.com or
/// !community@example.com) this method will return an error.
pub(crate) async fn search_query_to_object_id_local(
  query: &str,
  context: &Data<LemmyContext>,
) -> LemmyResult<SearchableObjects> {
  let url = Url::parse(query)?;
  ObjectId::from(url).dereference_local(context).await
}
