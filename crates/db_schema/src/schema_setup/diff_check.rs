use diesel::{PgConnection, RunQueryDsl};
use lemmy_utils::settings::SETTINGS;
use std::{
  borrow::Cow,
  collections::BTreeSet,
  fmt::Write,
  process::{Command, Stdio},
};

diesel::sql_function! {
  fn pg_export_snapshot() -> diesel::sql_types::Text;
}

pub fn get_dump(conn: &mut PgConnection) -> String {
  /*// Required for pg_dump to see uncommitted changes from a different database connection

  // The pg_dump command runs independently from `conn`, which means it can't see changes from
  // an uncommitted transaction. NASA made each migration run in a separate transaction. When
  // it was discovered that 
  let snapshot = diesel::select(pg_export_snapshot())
    .get_result::<String>(conn)
    .expect("pg_export_snapshot failed");
  let snapshot_arg = format!("--snapshot={snapshot}");*/

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

const PATTERN_LEN: usize = 19;

// TODO add unit test for output
pub fn check_dump_diff(conn: &mut PgConnection, mut before: String, name: &str) {
  let mut after = get_dump(conn);
  // Ignore timestamp differences by removing timestamps
  for dump in [&mut before, &mut after] {
    for index in 0.. {
      // Check for this pattern: 0000-00-00 00:00:00
      let Some((
        &[a0, a1, a2, a3, b0, a4, a5, b1, a6, a7, b2, a8, a9, b3, a10, a11, b4, a12, a13],
        remaining,
      )) = dump
        .get(index..)
        .and_then(|s| s.as_bytes().split_first_chunk::<PATTERN_LEN>())
      else {
        break;
      };
      if [a0, a1, a2, a3, a4, a5, a6, a7, a8, a9, a10, a11, a12, a13]
        .into_iter()
        .all(|byte| byte.is_ascii_digit())
        && [b0, b1, b2, b3, b4] == *b"-- ::"
      {
        // Replace the part of the string that has the checked pattern and an optional fractional part
        let len_after = if let Some((b'.', s)) = remaining.split_first() {
          1 + s.iter().position(|c| !c.is_ascii_digit()).unwrap_or(0)
        } else {
          0
        };
        // Length of replacement string is likely to match previous string
        // (usually there's up to 6 digits after the decimal point)
        dump.replace_range(
          index..(index + PATTERN_LEN + len_after),
          "AAAAAAAAAAAAAAAAAAAAAAAAAA",
        );
      }
    }
  }

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
      .map(|chunk| {
        (
          &**chunk,
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
      .max_by_key(|(_, (after_chunk, after_chunk_filtered))| {
        diff::chars(after_chunk_filtered, &before_chunk_filtered)
          .into_iter()
          .filter(|i| matches!(i, diff::Result::Both(c, _)
          // This increases accuracy for some trigger function diffs
          if c.is_lowercase()))
          .count()
      })
      .expect("resize should have prevented this from failing");

    output.push('\n');
    for line in diff::lines(most_similar_chunk, &before_chunk) {
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
fn chunks<'a>(dump: &'a str) -> impl Iterator<Item = Cow<'a, str>> {
  let mut remaining = dump;
  std::iter::from_fn(move || {
    remaining = remaining.trim_start();
    while let Some(s) = remove_skipped_item_from_beginning(remaining) {
      remaining = s.trim_start();
    }
    // `a` can't be empty because of trim_start
    let (result, after_result) = remaining.split_once("\n\n")?;
    remaining = after_result;
    Some(if result.starts_with("CREATE TABLE ") {
      // Allow column order to change
      let mut lines = result
        .lines()
        .map(|line| line.strip_suffix(',').unwrap_or(line))
        .collect::<Vec<_>>();
      lines.sort_unstable_by_key(|line| -> (u8, &str) {
        let placement = match line.chars().next() {
          Some('C') => 0,
          Some(' ') => 1,
          Some(')') => 2,
          _ => panic!("unrecognized part of `CREATE TABLE` statement: {line}"),
        };
        (placement, line)
      });
      Cow::Owned(lines.join("\n"))
    } else {
      Cow::Borrowed(result)
    })
  })
}

fn remove_skipped_item_from_beginning(s: &str) -> Option<&str> {
  // Skip commented line
  if let Some(after) = s.strip_prefix("--") {
    Some(after.split_once('\n').unwrap_or_default().1)
  }
  // Skip view definition that's replaced later (the first definition selects all nulls)
  else if let Some(after) = s.strip_prefix("CREATE VIEW ") {
    let (name, after_name) = after.split_once(' ').unwrap_or_default();
    Some(after_name.split_once("\n\n").unwrap_or_default().1)
      .filter(|after_view| after_view.contains(&format!("\nCREATE OR REPLACE VIEW {name} ")))
  } else {
    None
  }
}
