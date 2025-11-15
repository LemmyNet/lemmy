use serde::Deserialize;
use std::{
  env,
  fs::File,
  io::{BufWriter, Write},
  path::Path,
};

/// Generates a list of communities which is fetched during Lemmy setup.
fn main() -> Result<(), Box<dyn std::error::Error>> {
  let community_ids = if env::var("PROFILE")? == "release" {
    // fetch list of communities from lemmy.ml
    // TODO: once lemmy.ml is upgraded to 1.0:
    //       - switch to api v4
    //       - change sort to `Subscribers`
    //       - directly use ListCommunitiesResponse as build-dependency (if it doesnt affect
    //         compilation time)
    // TODO: lemmy api returns 50 items at most, consider using pagination to get more
    let client = reqwest::blocking::Client::builder()
      .user_agent("Lemmy Build Script")
      .build()?;
    let mut communities: Vec<_> = client
      .get("https://lemmy.ml/api/v3/community/list?type_=All&sort=TopAll&limit=50")
      .send()?
      .json::<ListCommunitiesResponse>()?
      .communities
      .into_iter()
      // exclude nsfw
      .filter(|c| !c.community.nsfw)
      .map(|c| c.community.actor_id)
      .collect();

    // also prefetch these two communities as they are linked in the welcome post
    communities.insert(0, "https://lemmy.ml/c/announcements".to_string());
    communities.insert(0, "https://lemmy.ml/c/lemmy".to_string());
    communities
  } else {
    // in debug mode only use a single hardcoded community to avoid unnecessary requests
    vec!["https://lemmy.ml/c/lemmy".to_string()]
  };

  // write to file
  write_out_file("communities.json", serde_json::to_string(&community_ids)?)?;
  Ok(())
}

#[derive(Deserialize)]
pub struct ListCommunitiesResponse {
  pub communities: Vec<CommunityView>,
}

#[derive(Deserialize)]
pub struct CommunityView {
  pub community: Community,
  pub counts: CommunityAggregates,
}

#[derive(Deserialize)]
pub struct Community {
  pub nsfw: bool,
  pub actor_id: String,
}

#[derive(Deserialize)]
pub struct CommunityAggregates {
  pub users_active_month: i64,
}

/// https://github.com/harindaka/build_script_file_gen/blob/master/src/lib.rs
fn write_out_file(file_name: &str, content: String) -> Result<(), Box<dyn std::error::Error>> {
  let out_dir = env::var("OUT_DIR")?;
  let dest_path = Path::new(&out_dir).join(file_name);
  let mut f = BufWriter::new(File::create(&dest_path)?);

  write!(f, "{}", &content)?;
  Ok(())
}
