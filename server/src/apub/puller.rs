extern crate reqwest;

use self::reqwest::Error;
use crate::api::community::{GetCommunityResponse, ListCommunitiesResponse};
use crate::api::post::GetPosts;
use crate::apub::group_wrapper::GroupHelper;
use crate::db::community_view::CommunityView;
use crate::settings::Settings;
use activitypub::actor::Group;
use crate::apub::parse_apub_endpoint;

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

pub fn get_remote_community_posts(name: String) -> Result<GetPosts, Error> {
  // TODO: this is for urls like /c/!main@example.com, activitypub exposes it through the outbox
  //       https://www.w3.org/TR/activitypub/#outbox
  dbg!(name);
  unimplemented!()
}

pub fn get_remote_community(identifier: String) -> Result<GetCommunityResponse, failure::Error> {
  let x: Vec<&str> = identifier.split('@').collect();
  let name = x[0].replace("!", "");
  let instance = x[1];
  let community_uri = format!("http://{}/federation/c/{}", instance, name);
  let community: Group = reqwest::get(&community_uri)?.json()?;

  // TODO: looks like a bunch of data is missing from the activitypub response
  // TODO: i dont think simple numeric ids are going to work, we probably need something like uuids
  Ok(GetCommunityResponse {
    moderators: vec![],
    admins: vec![],
    community: CommunityView {
      // TODO: we need to merge id and name into a single thing (stuff like @user@instance.com)
      id: parse_apub_endpoint(&Group::get_id(&community)?)?.1.parse::<i32>()?,
      name,
      title: Group::get_title(&community)?,
      description: Group::get_description(&community)?,
      category_id: -1,
      creator_id: parse_apub_endpoint(&Group::get_creator_id(&community)?)?.1.parse::<i32>()?,
      removed: false,
      published: Group::get_published(&community)?,
      updated: Group::get_updated(&community)?,
      deleted: false,
      nsfw: false,
      creator_name: "".to_string(),
      creator_avatar: None,
      category_name: "".to_string(),
      number_of_subscribers: -1,
      number_of_posts: -1,
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
