use lemmy_utils::LemmyErrorType;
use std::{env::current_dir, process::Command};
use strum::IntoEnumIterator;

#[test]
fn test_errors_used() {
  let mut unused_error_found = false;
  let mut current_dir = current_dir().unwrap();
  current_dir.pop();
  current_dir.pop();
  for error in LemmyErrorType::iter() {
    let mut grep_all = Command::new("grep");
    let grep_all = grep_all
      .current_dir(current_dir.clone())
      .arg("-R")
      .arg("--exclude=error.rs")
      .arg(error.to_string())
      .arg("crates/")
      .arg("src/");
    let output = grep_all.output().unwrap();
    let grep_all_out = std::str::from_utf8(&output.stdout).unwrap();

    let mut grep_apub = Command::new("grep");
    let grep_apub = grep_apub
      .current_dir(current_dir.clone())
      .arg("-R")
      .arg(error.to_string())
      .arg("crates/apub/");
    let output = grep_apub.output().unwrap();
    let grep_apub_out = std::str::from_utf8(&output.stdout).unwrap();

    if grep_all_out.len() == 0 {
      println!("LemmyErrorType::{} is unused", error);
      unused_error_found = true;
    }
    if grep_all_out == grep_apub_out {
      println!("LemmyErrorType::{} is only used for federation", error);
      unused_error_found = true;
    }
  }
  assert!(unused_error_found == false);
}
