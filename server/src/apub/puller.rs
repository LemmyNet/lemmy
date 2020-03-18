extern crate reqwest;

use crate::api::community::{GetCommunityResponse, ListCommunitiesResponse};
use crate::api::post::GetPostsResponse;
use crate::apub::get_apub_protocol_string;
use crate::db::community_view::CommunityView;
use crate::db::post_view::PostView;
use crate::naive_now;
use crate::routes::nodeinfo::{NodeInfo, NodeInfoWellKnown};
use crate::settings::Settings;
use activitystreams::actor::apub::Group;
use activitystreams::collection::apub::{OrderedCollection, UnorderedCollection};
use activitystreams::object::apub::Page;
use activitystreams::object::ObjectBox;
use failure::Error;
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
  if node_info.software.name != "lemmy" {
    return Err(format_err!(
      "{} is not a Lemmy instance, federation is not supported",
      domain
    ));
  }

  // TODO: follow pagination (seems like page count is missing?)
  // TODO: see if there is any standard for discovering remote actors, so we dont have to rely on lemmy apis
  let communities_uri = format!(
    "http://{}/api/v1/communities/list?sort=Hot&local_only=true",
    domain
  );
  let communities1 = fetch_remote_object::<ListCommunitiesResponse>(&communities_uri)?;
  let mut communities2 = communities1.communities;
  for c in &mut communities2 {
    c.name = format_community_name(&c.name, domain);
  }
  Ok(communities2)
}

fn get_remote_community_uri(identifier: &str) -> String {
  let x: Vec<&str> = identifier.split('@').collect();
  let name = x[0].replace("!", "");
  let instance = x[1];
  format!("http://{}/federation/c/{}", instance, name)
}

fn fetch_remote_object<Response>(uri: &str) -> Result<Response, Error>
where
  Response: for<'de> Deserialize<'de>,
{
  if Settings::get().federation.tls_enabled && !uri.starts_with("https") {
    return Err(format_err!("Activitypub uri is insecure: {}", uri));
  }
  // TODO: should cache responses here when we are in production
  // TODO: this function should return a future
  // TODO: in production mode, fail if protocol is not https
  let x: Response = reqwest::get(uri)?.json()?;
  Ok(x)
}

pub fn get_remote_community_posts(identifier: &str) -> Result<GetPostsResponse, Error> {
  let community = fetch_remote_object::<Group>(&get_remote_community_uri(identifier))?;
  let outbox_uri = &community.ap_actor_props.get_outbox().to_string();
  let outbox = fetch_remote_object::<OrderedCollection>(outbox_uri)?;
  let items = outbox.collection_props.get_many_items_object_boxs();

  let posts: Vec<PostView> = items
    .unwrap()
    .map(|obox: &ObjectBox| {
      let page: Page = obox.clone().to_concrete::<Page>().unwrap();
      // TODO: need to populate this
      PostView {
        id: -1,
        name: page.object_props.get_name_xsd_string().unwrap().to_string(),
        url: None,
        body: None,
        creator_id: -1,
        community_id: -1,
        removed: false,
        locked: false,
        published: naive_now(),
        updated: None,
        deleted: false,
        nsfw: false,
        stickied: false,
        embed_title: None,
        embed_description: None,
        embed_html: None,
        thumbnail_url: None,
        banned: false,
        banned_from_community: false,
        creator_name: "".to_string(),
        creator_avatar: None,
        community_name: "".to_string(),
        community_removed: false,
        community_deleted: false,
        community_nsfw: false,
        number_of_comments: -1,
        score: -1,
        upvotes: -1,
        downvotes: -1,
        hot_rank: -1,
        newest_activity_time: naive_now(),
        user_id: None,
        my_vote: None,
        subscribed: None,
        read: None,
        saved: None,
      }
    })
    .collect();
  Ok(GetPostsResponse { posts })
}

pub fn get_remote_community(identifier: &str) -> Result<GetCommunityResponse, failure::Error> {
  let community = fetch_remote_object::<Group>(&get_remote_community_uri(identifier))?;
  let followers_uri = &community
    .ap_actor_props
    .get_followers()
    .unwrap()
    .to_string();
  let outbox_uri = &community.ap_actor_props.get_outbox().to_string();
  let outbox = fetch_remote_object::<OrderedCollection>(outbox_uri)?;
  let followers = fetch_remote_object::<UnorderedCollection>(followers_uri)?;
  // TODO: this is only for testing until we can call that function from GetPosts
  // (once string ids are supported)
  //dbg!(get_remote_community_posts(identifier)?);

  Ok(GetCommunityResponse {
    moderators: vec![],
    admins: vec![],
    community: CommunityView {
      // TODO: we need to merge id and name into a single thing (stuff like @user@instance.com)
      id: 1337, //community.object_props.get_id()
      name: identifier.to_string(),
      title: community
        .object_props
        .get_name_xsd_string()
        .unwrap()
        .to_string(),
      description: community
        .object_props
        .get_summary_xsd_string()
        .map(|s| s.to_string()),
      category_id: -1,
      creator_id: -1, //community.object_props.get_attributed_to_xsd_any_uri()
      removed: false,
      published: community
        .object_props
        .get_published()
        .unwrap()
        .as_ref()
        .naive_local()
        .to_owned(),
      updated: community
        .object_props
        .get_updated()
        .map(|u| u.as_ref().to_owned().naive_local()),
      deleted: false,
      nsfw: false,
      creator_name: "".to_string(),
      creator_avatar: None,
      category_name: "".to_string(),
      number_of_subscribers: *followers
        .collection_props
        .get_total_items()
        .unwrap()
        .as_ref() as i64, // TODO: need to use the same type
      number_of_posts: *outbox.collection_props.get_total_items().unwrap().as_ref() as i64,
      number_of_comments: -1,
      hot_rank: -1,
      user_id: None,
      subscribed: None,
    },
    online: 0,
  })
}

pub fn get_following_instances() -> Vec<&'static str> {
  Settings::get()
    .federation
    .followed_instances
    .split(',')
    .collect()
}

pub fn get_all_communities() -> Result<Vec<CommunityView>, Error> {
  let mut communities_list: Vec<CommunityView> = vec![];
  for instance in &get_following_instances() {
    match fetch_communities_from_instance(instance) {
      Ok(mut c) => communities_list.append(c.as_mut()),
      Err(e) => warn!("Failed to fetch instance list from remote instance: {}", e),
    };
  }
  Ok(communities_list)
}

/// If community is on local instance, don't include the @instance part
pub fn format_community_name(name: &str, instance: &str) -> String {
  if instance == Settings::get().hostname {
    format!("!{}", name)
  } else {
    format!("!{}@{}", name, instance)
  }
}
