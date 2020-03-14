extern crate reqwest;

use crate::api::community::{GetCommunityResponse, ListCommunitiesResponse};
use crate::api::post::GetPostsResponse;
use crate::db::community_view::CommunityView;
use crate::settings::Settings;
use activitystreams::actor::apub::Group;
use activitystreams::collection::apub::{OrderedCollection, UnorderedCollection};
use failure::Error;

// TODO: right now all of the data is requested on demand, for production we will need to store
//       things in the local database to not ruin the performance

fn fetch_communities_from_instance(domain: &str) -> Result<Vec<CommunityView>, Error> {
  // TODO: check nodeinfo to make sure we are dealing with a lemmy instance
  //       -> means we need proper nodeinfo json classes instead of inline generation
  // TODO: follow pagination (seems like page count is missing?)
  // TODO: see if there is any standard for discovering remote actors, so we dont have to rely on lemmy apis
  let communities_uri = format!("http://{}/api/v1/communities/list?sort=Hot", domain);
  let communities1: ListCommunitiesResponse = reqwest::get(&communities_uri)?.json()?;
  let mut communities2 = communities1.communities;
  for c in &mut communities2 {
    c.name = format_community_name(&c.name, domain);
  }
  Ok(communities2)
}

// TODO: this should be cached or stored in the database
fn get_remote_community_uri(identifier: &str) -> String {
  let x: Vec<&str> = identifier.split('@').collect();
  let name = x[0].replace("!", "");
  let instance = x[1];
  format!("http://{}/federation/c/{}", instance, name)
}

pub fn get_remote_community_posts(identifier: &str) -> Result<GetPostsResponse, Error> {
  let community: Group = reqwest::get(&get_remote_community_uri(identifier))?.json()?;
  let outbox_uri = &community.ap_actor_props.get_outbox().to_string();
  let outbox: OrderedCollection = reqwest::get(outbox_uri)?.json()?;
  let items = outbox.collection_props.get_many_items_object_boxs();
  dbg!(items);
  unimplemented!()
}

pub fn get_remote_community(identifier: &str) -> Result<GetCommunityResponse, failure::Error> {
  let community: Group = reqwest::get(&get_remote_community_uri(identifier))?.json()?;
  let followers_uri = &community
    .ap_actor_props
    .get_followers()
    .unwrap()
    .to_string();
  let outbox_uri = &community.ap_actor_props.get_outbox().to_string();
  let outbox: OrderedCollection = reqwest::get(outbox_uri)?.json()?;
  // TODO: this need to be done in get_remote_community_posts() (meaning we need to store the outbox uri?)
  let followers: UnorderedCollection = reqwest::get(followers_uri)?.json()?;

  // TODO: looks like a bunch of data is missing from the activitypub response
  // TODO: i dont think simple numeric ids are going to work, we probably need something like uuids
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

pub fn get_following_instances() -> Result<Vec<String>, Error> {
  let instance_list = match Settings::get().federated_instance.clone() {
    Some(f) => vec![f, Settings::get().hostname.clone()],
    None => vec![Settings::get().hostname.clone()],
  };
  Ok(instance_list)
}

pub fn get_all_communities() -> Result<Vec<CommunityView>, Error> {
  let mut communities_list: Vec<CommunityView> = vec![];
  for instance in &get_following_instances()? {
    communities_list.append(fetch_communities_from_instance(instance)?.as_mut());
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
