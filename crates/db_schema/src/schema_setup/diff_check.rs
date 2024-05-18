use lemmy_utils::settings::SETTINGS;
use std::{
  collections::BTreeSet,
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
  if after == before {
    return;
  }
  let [before_chunks, after_chunks] =
    [&before, &after].map(|dump| chunks(dump).collect::<BTreeSet<_>>());

  // todo dont collect only_in_before?
  let [mut only_in_before, mut only_in_after] = [
    before_chunks.difference(&after_chunks),
    after_chunks.difference(&before_chunks),
  ]
  .map(|chunks| {
    chunks
      .map(|&chunk| {
        (
          chunk,
          // Used for ignoring formatting differences, especially indentation level, when
          // determining which item in `only_in_before` corresponds to which item in `only_in_after`
          chunk.replace([' ', '\t', '\r', '\n'], ""),
        )
      })
      .collect::<Vec<_>>()
  });
  if only_in_before.is_empty() && only_in_after.is_empty() {
    return;
  }
  // add empty strings to the shorter vec so the lengths match
  if only_in_before.len() < only_in_after.len() {
    only_in_before.resize_with(only_in_after.len(), Default::default);
  } else {
    only_in_after.resize_with(only_in_before.len(), Default::default);
  }

  let mut output = format!("These changes need to be applied in {name}:");
  for (before_chunk, before_chunk_filtered) in only_in_before {
    let (most_similar_chunk_index, (most_similar_chunk, _)) = only_in_after
      .iter()
      .enumerate()
      .max_by_key(|(_, (_, after_chunk_filtered))| {
        diff::chars(after_chunk_filtered, &before_chunk_filtered)
          .into_iter()
          .filter(|i| matches!(i, diff::Result::Both(_, _)))
          .count()
      })
      .expect("resize should have prevented this from failing");

    output.push_str('\n');
    for line in diff::lines(most_similar_chunk, before_chunk) {
      match line {
        diff::Result::Left(s) => write!(&mut output, "\n- {s}"),
        diff::Result::Right(s) => write!(&mut output, "\n+ {s}"),
        diff::Result::Both(s, _) => write!(&mut output, "\n  {s}"),
      }
      .expect("failed to build string");
    }
    only_in_after.swap_remove(most_similar_chunk_index);
  }
  // should have all been removed
  assert_eq!(only_in_after.len(), 0);
  panic!("{output}");
}

// todo inline?
fn chunks(dump: &str) -> impl Iterator<Item = &str> {
  let mut remaining = dump;
  std::iter::from_fn(move || {
    remaining = remaining.trim_start();
    while remaining.starts_with("--") {
      remaining = remaining.split_once('\n')?.1;
      remaining = remaining.trim_start();
    }
    let (a, b) = remaining.split_once("\n\n")?;
    remaining = b;
    // `a` can't be empty because of trim_start
    Some(a)
  })
}
