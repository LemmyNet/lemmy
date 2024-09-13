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
    let mut command = Command::new("grep");
    let command = command
      .current_dir(current_dir.clone())
      .arg("-R")
      .arg("--exclude=error.rs")
      .arg(error.to_string())
      .arg("crates/")
      .arg("src/");
    let output = command.output().unwrap();
    let stdout = std::str::from_utf8(&output.stdout).unwrap();
    if stdout.len() == 0 {
      println!("LemmyErrorType::{} is unused", error);
      unused_error_found = true;
    }
  }
  assert!(unused_error_found == false);
}
