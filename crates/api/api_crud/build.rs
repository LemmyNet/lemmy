use serde::Deserialize;
use std::{
  env,
  fs::File,
  io::{BufWriter, Write},
  path::Path,
};

/// Generates a list of communities which is fetched during Lemmy setup.
fn main() -> Result<(), Box<dyn std::error::Error>> {
  let community_ids = if env::var("OUT_DIR").unwrap() == "release" {
    // fetch list of communities from lemmyverse.net
    let mut communities: Vec<CommunityInfo> =
      reqwest::blocking::get("https://data.lemmyverse.net/data/community.full.json")?.json()?;

    // invert sort key to get largest values first
    communities.sort_by_key(|c| -c.counts.users_active_month);

    communities
      .into_iter()
      .map(|c| c.url)
      .take(100)
      .collect::<Vec<_>>()
  } else {
    // in debug mode only use a single hardcoded community to avoid unnecessary requests
    vec!["https://lemmy.ml/c/lemmy".to_string()]
  };

  // write to file
  write_out_file("communities.json", serde_json::to_string(&community_ids)?);
  Ok(())
}

#[derive(Deserialize)]
struct CommunityInfo {
  url: String,
  counts: CommunityInfoCounts,
}

#[derive(Deserialize)]
struct CommunityInfoCounts {
  users_active_month: i32,
}

/// https://github.com/harindaka/build_script_file_gen/blob/master/src/lib.rs
fn write_out_file(file_name: &str, content: String) {
  let out_dir = env::var("OUT_DIR").unwrap();
  let dest_path = Path::new(&out_dir).join(&file_name);
  let mut f = BufWriter::new(File::create(&dest_path).unwrap());

  write!(f, "{}", &content).unwrap();
}
