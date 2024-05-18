use lemmy_utils::settings::SETTINGS;
use std::{
  fmt::Write,
  process::{Command, Stdio},
};

pub fn get_dump() -> String {
  let output = Command::new("pg_dump")
    .args(["--schema-only"])
    .env("DATABASE_URL", SETTINGS.get_database_url())
    .stderr(Stdio::inherit())
    .output()
    .expect("failed to start pg_dump process");

  // TODO: use exit_ok method when it's stable
  assert!(output.status.success());

  String::from_utf8(output.stdout).expect("pg_dump output is not valid UTF-8 text")
}

// TODO add unit test for output
pub fn check_dump_diff(before: String, name: &str) {
  let after = get_dump();
  if after != before {
    let mut output = format!("These changes need to be applied in {name}:");
    let line_diffs = diff::lines(&after, &before);
    for chunk in line_diffs.split(|line| matches!(line, diff::Result::Left("") | diff::Result::Right("") | diff::Result::Both("", _))) {
      if chunk
        .iter()
        .all(|line| matches!(line, diff::Result::Both(_, _)))
      {
        continue;
      }
      output.push_str("\n================");
      for line in chunk {
        match line {
          diff::Result::Left(s) => write!(&mut output, "\n- {s}"),
          diff::Result::Right(s) => write!(&mut output, "\n+ {s}"),
          diff::Result::Both(s, _) => write!(&mut output, "\n  {s}"),
        }
        .expect("failed to build string");
      }
    }
    panic!("{output}");
  }
}
