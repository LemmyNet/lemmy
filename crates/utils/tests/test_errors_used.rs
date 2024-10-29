use lemmy_utils::LemmyErrorType;
use std::{env::current_dir, process::Command};
use strum::IntoEnumIterator;

#[test]
#[allow(clippy::unwrap_used)]
fn test_errors_used() {
  let mut unused_error_found = false;
  let mut current_dir = current_dir().unwrap();
  current_dir.pop();
  current_dir.pop();
  for error in LemmyErrorType::iter() {
    let search = format!("LemmyErrorType::{error}");
    let mut grep_all = Command::new("grep");
    let grep_all = grep_all
      .current_dir(current_dir.clone())
      .arg("-R")
      .arg("--exclude=error.rs")
      .arg(&search)
      .arg("crates/")
      .arg("src/");
    let output = grep_all.output().unwrap();
    let grep_all_out = std::str::from_utf8(&output.stdout).unwrap();

    let mut grep_apub = Command::new("grep");
    let grep_apub = grep_apub
      .current_dir(current_dir.clone())
      .arg("-R")
      .arg("--exclude-dir=api")
      .arg(&search)
      .arg("crates/apub/");
    let output = grep_apub.output().unwrap();
    let grep_apub_out = std::str::from_utf8(&output.stdout).unwrap();

    if grep_all_out.is_empty() {
      println!("LemmyErrorType::{} is unused", error);
      unused_error_found = true;
    }
    if search != "LemmyErrorType::FederationError" && grep_all_out == grep_apub_out {
      println!("LemmyErrorType::{} is only used for federation", error);
      unused_error_found = true;
    }
  }
  assert!(!unused_error_found);
}
