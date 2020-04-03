use crate::api::community::GetCommunityResponse;
use crate::api::post::GetPostsResponse;
use crate::apub::*;
use crate::db::community_view::CommunityView;
use crate::db::post_view::PostView;
use crate::routes::nodeinfo::{NodeInfo, NodeInfoWellKnown};
use crate::settings::Settings;
use activitystreams::collection::{OrderedCollection, UnorderedCollection};
use activitystreams::object::Page;
use activitystreams::BaseBox;
use failure::Error;
use isahc::prelude::*;
use log::warn;
use serde::Deserialize;

fn fetch_node_info(domain: &str) -> Result<NodeInfo, Error> {
  let well_known_uri = format!(
    "{}://{}/.well-known/nodeinfo",
    get_apub_protocol_string(),
    domain
  );
  let well_known = fetch_remote_object::<NodeInfoWellKnown>(&well_known_uri)?;
  Ok(fetch_remote_object::<NodeInfo>(&well_known.links.href)?)
}

fn fetch_communities_from_instance(domain: &str) -> Result<Vec<CommunityView>, Error> {
  let node_info = fetch_node_info(domain)?;

  if let Some(community_list_url) = node_info.metadata.community_list_url {
    let collection = fetch_remote_object::<UnorderedCollection>(&community_list_url)?;
    let object_boxes = collection
      .collection_props
      .get_many_items_base_boxes()
      .unwrap();
    let communities: Result<Vec<CommunityView>, Error> = object_boxes
      .map(|c| -> Result<CommunityView, Error> {
        let group = c.to_owned().to_concrete::<GroupExt>()?;
        CommunityView::from_group(&group, domain)
      })
      .collect();
    Ok(communities?)
  } else {
    Err(format_err!(
      "{} is not a Lemmy instance, federation is not supported",
      domain
    ))
  }
}

pub fn fetch_remote_object<Response>(uri: &str) -> Result<Response, Error>
where
  Response: for<'de> Deserialize<'de>,
{
  if Settings::get().federation.tls_enabled && !uri.starts_with("https://") {
    return Err(format_err!("Activitypub uri is insecure: {}", uri));
  }
  // TODO: should cache responses here when we are in production
  // TODO: this function should return a future
  let text = isahc::get(uri)?.text()?;
  let res: Response = serde_json::from_str(&text)?;
  Ok(res)
}

pub fn fetch_remote_community_posts(identifier: &str) -> Result<GetPostsResponse, Error> {
  let community = fetch_remote_object::<GroupExt>(&get_remote_community_uri(identifier))?;
  let outbox_uri = &community.extension.get_outbox().to_string();
  let outbox = fetch_remote_object::<OrderedCollection>(outbox_uri)?;
  let items = outbox.collection_props.get_many_items_base_boxes();

  let posts: Result<Vec<PostView>, Error> = items
    .unwrap()
    .map(|obox: &BaseBox| {
      let page = obox.clone().to_concrete::<Page>().unwrap();
      PostView::from_page(&page)
    })
    .collect();
  Ok(GetPostsResponse { posts: posts? })
}

pub fn fetch_remote_community(identifier: &str) -> Result<GetCommunityResponse, failure::Error> {
  let group = fetch_remote_object::<GroupExt>(&get_remote_community_uri(identifier))?;
  // TODO: this is only for testing until we can call that function from GetPosts
  // (once string ids are supported)
  //dbg!(get_remote_community_posts(identifier)?);

  let (_, domain) = split_identifier(identifier);
  Ok(GetCommunityResponse {
    moderators: vec![],
    admins: vec![],
    community: CommunityView::from_group(&group, &domain)?,
    online: 0,
  })
}

pub fn fetch_all_communities() -> Result<Vec<CommunityView>, Error> {
  let mut communities_list: Vec<CommunityView> = vec![];
  for instance in &get_following_instances() {
    match fetch_communities_from_instance(instance) {
      Ok(mut c) => communities_list.append(c.as_mut()),
      Err(e) => warn!("Failed to fetch instance list from remote instance: {}", e),
    };
  }
  Ok(communities_list)
}
