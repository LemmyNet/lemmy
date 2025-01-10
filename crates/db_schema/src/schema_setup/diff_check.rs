#![cfg(test)]
#![expect(clippy::expect_used)]
use lemmy_utils::settings::SETTINGS;
use std::process::{Command, Stdio};

// It's not possible to call `export_snapshot()` for each dump and run the dumps in parallel with
// the `--snapshot` flag. Don't waste your time!!!!

pub fn get_dump() -> String {
  let db_url = SETTINGS.get_database_url();
  let output = Command::new("pg_dump")
    .args([
      // Specify database URL
      "--dbname",
      &db_url,
      // Disable some things
      "--no-owner",
      "--no-privileges",
      "--no-table-access-method",
      "--schema-only",
      "--no-sync",
    ])
    .stderr(Stdio::inherit())
    .output()
    .expect("failed to start pg_dump process");

  // TODO: use exit_ok method when it's stable
  assert!(output.status.success());

  String::from_utf8(output.stdout).expect("pg_dump output is not valid UTF-8 text")
}

pub fn check_dump_diff(before: String, after: String, label: &str) {
  if before != after {
    let diff_bytes =
      diffutilslib::unified_diff(before.as_bytes(), after.as_bytes(), &Default::default());
    let diff = String::from_utf8_lossy(&diff_bytes);

    panic!("{label}\n\n{diff}");
  }
}
