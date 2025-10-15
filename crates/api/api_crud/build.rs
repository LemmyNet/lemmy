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
    // fetch list of communities from lemmyverse.net
    let mut communities: Vec<_> =
      reqwest::blocking::get("https://data.lemmyverse.net/data/community.full.json")?
        .json::<Vec<CommunityInfo>>()?
        .into_iter()
        // exclude nsfw and suspicious
        .filter(|c| !c.nsfw && !c.is_suspicious)
        .collect();

    // invert sort key to get largest values first
    communities.sort_by_key(|c| -c.counts.users_active_month);

    // take urls of top 100 communities
    let mut c = communities
      .into_iter()
      .map(|c| c.url)
      .take(100)
      .collect::<Vec<_>>();

    // also prefetch these two communities as they are linked in the welcome post
    c.insert(0, "https://lemmy.ml/c/announcements".to_string());
    c.insert(0, "https://lemmy.ml/c/lemmy".to_string());
    c
  } else {
    // in debug mode only use a single hardcoded community to avoid unnecessary requests
    vec!["https://lemmy.ml/c/lemmy".to_string()]
  };

  // write to file
  write_out_file("communities.json", serde_json::to_string(&community_ids)?)?;
  Ok(())
}

#[derive(Deserialize)]
struct CommunityInfo {
  url: String,
  counts: CommunityInfoCounts,
  nsfw: bool,
  #[serde(rename = "isSuspicious")]
  is_suspicious: bool,
}

#[derive(Deserialize)]
struct CommunityInfoCounts {
  users_active_month: i32,
}

/// https://github.com/harindaka/build_script_file_gen/blob/master/src/lib.rs
fn write_out_file(file_name: &str, content: String) -> Result<(), Box<dyn std::error::Error>> {
  let out_dir = env::var("OUT_DIR")?;
  let dest_path = Path::new(&out_dir).join(file_name);
  let mut f = BufWriter::new(File::create(&dest_path)?);

  write!(f, "{}", &content).unwrap();
  Ok(())
}
