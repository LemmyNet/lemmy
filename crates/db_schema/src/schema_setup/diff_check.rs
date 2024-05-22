use lemmy_utils::settings::SETTINGS;
use std::{
  borrow::Cow,
  collections::btree_set::{self, BTreeSet},
  process::{Command, Stdio},
};

// It's not possible to call `export_snapshot()` for each dump and run the dumps in parallel with the
// `--snapshot` flag. Don't waste your time!!!

pub fn get_dump() -> String {
  let output = Command::new("pg_dump")
    .args([
      "--schema-only",
      "--no-owner",
      "--no-privileges",
      "--no-comments",
      "--no-publications",
      "--no-security-labels",
      "--no-subscriptions",
      "--no-table-access-method",
      "--no-tablespaces",
    ])
    .env("DATABASE_URL", SETTINGS.get_database_url())
    .stderr(Stdio::inherit())
    .output()
    .expect("failed to start pg_dump process");

  // TODO: use exit_ok method when it's stable
  assert!(output.status.success());

  String::from_utf8(output.stdout).expect("pg_dump output is not valid UTF-8 text")
}

const PATTERN_LEN: usize = 19;

pub fn check_dump_diff(before: String, after: String, label: &str) {
  // Performance optimization
  if after == before {
    return;
  }

  let normalized_chunk_vecs = [&before, &after]
    // Remove identical items
    .map(|dump| chunks(dump).collect::<BTreeSet<_>>())
    .differences()
    // Remove items without unwanted types of differences (if migrations are correct, then this removes everything)
    .map(|chunks| chunks.map(|&i| normalize_chunk(i)).collect::<BTreeSet<_>>());

  let [only_in_before, only_in_after] = normalized_chunk_vecs
    .differences()
    .map(|chunks| chunks.map(|i| &**i).collect::<Vec<_>>());

  if only_in_before.is_empty() && only_in_after.is_empty() {
    return;
  }

  // Build the panic message

  let after_has_more = only_in_before.len() < only_in_after.len();
  let [chunks, mut other_chunks] = if after_has_more {
    [only_in_before, only_in_after]
  } else {
    [only_in_after, only_in_before]
  };

  let diffs = chunks
    .into_iter()
    .chain(std::iter::repeat(""))
    .map_while(|chunk| {
      let (most_similar_chunk_index, most_similar_chunk) = other_chunks
        .iter()
        .enumerate()
        .max_by_key(|(_, other_chunk)| {
          if sql_command_name(chunk) != sql_command_name(other_chunk) {
            0
          } else {
            similarity(chunk, other_chunk)
          }
        })?;

      let lines = if after_has_more {
        diff::lines(most_similar_chunk, chunk)
      } else {
        diff::lines(chunk, most_similar_chunk)
      };

      other_chunks.swap_remove(most_similar_chunk_index);

      Some(
        lines
          .into_iter()
          .map(|line| {
            Cow::Owned(match line {
              diff::Result::Left(s) => format!("- {s}\n"),
              diff::Result::Right(s) => format!("+ {s}\n"),
              diff::Result::Both(s, _) => format!("  {s}\n"),
            })
          })
          .chain([Cow::Borrowed("\n")])
          .collect::<String>(),
      )
    });

  panic!(
    "{}",
    std::iter::once(format!("{label}\n\n"))
      .chain(diffs)
      .collect::<String>()
  );
}

trait Differences<T> {
  fn differences(&self) -> [btree_set::Difference<'_, T>; 2];
}

impl<T: Ord> Differences<T> for [BTreeSet<T>; 2] {
  /// Items only in `a`, and items only in `b`
  fn differences(&self) -> [btree_set::Difference<'_, T>; 2] {
    let [a, b] = self;
    [a.difference(b), b.difference(a)]
  }
}

fn sql_command_name(chunk: &str) -> &str {
  chunk
    .split_once(|c: char| c.is_lowercase())
    .unwrap_or_default()
    .0
}

fn similarity(chunk: &str, other_chunk: &str) -> usize {
  diff::chars(chunk, other_chunk)
    .into_iter()
    .filter(|i| {
      match i {
        diff::Result::Both(c, _) => {
          // Prevent whitespace from affecting similarity level
          !c.is_whitespace()
            && (
              // Increase accuracy for some trigger function diffs
              c.is_lowercase()
                  // Preserve differences in names that contain a number
                  || c.is_numeric()
            )
        }
        _ => false,
      }
    })
    .count()
}

fn normalize_chunk(chunk: &str) -> Cow<'_, str> {
  let mut chunk = Cow::Borrowed(chunk);

  let stripped_lines = chunk
    .lines()
    .map(|line| line.strip_suffix(',').unwrap_or(line));

  // Sort column names, so differences in column order are ignored
  if chunk.starts_with("CREATE TABLE ") {
    let mut lines = stripped_lines.collect::<Vec<_>>();

    sort_within_sections(&mut lines, |line| {
      match line.chars().next() {
        // CREATE
        Some('C') => 0,
        // Indented column name
        Some(' ') => 1,
        // End
        Some(')') => 2,
        _ => panic!("unrecognized part of `CREATE TABLE` statement: {line}"),
      }
    });

    chunk = Cow::Owned(lines.join("\n"));
  } else if chunk.starts_with("CREATE VIEW ") || chunk.starts_with("CREATE OR REPLACE VIEW ") {
    let is_simple_select_statement = chunk.lines().enumerate().all(|(i, line)| {
      match (i, line.trim_start().chars().next()) {
        // CREATE
        (0, Some('C')) => true,
        // SELECT
        (1, Some('S')) => true,
        // FROM
        (_, Some('F')) if line.ends_with(';') => true,
        // Column name
        (_, Some(c)) if c.is_lowercase() => true,
        _ => false,
      }
    });

    if is_simple_select_statement {
      let mut lines = stripped_lines.collect::<Vec<_>>();

      sort_within_sections(&mut lines, |line| {
        match line.trim_start().chars().next() {
          // CREATE
          Some('C') => 0,
          // SELECT
          Some('S') => 1,
          // FROM
          Some('F') => 3,
          // Column name
          _ => 2,
        }
      });

      chunk = Cow::Owned(lines.join("\n"));
    }
  }

  // Replace timestamps with a constant string, so differences in timestamps are ignored
  for index in 0.. {
    // Performance optimization
    let Some(byte) = chunk.as_bytes().get(index) else {
      break;
    };
    if !byte.is_ascii_digit() {
      continue;
    }

    // Check for this pattern: 0000-00-00 00:00:00
    let Some((
      &[a0, a1, a2, a3, b0, a4, a5, b1, a6, a7, b2, a8, a9, b3, a10, a11, b4, a12, a13],
      remaining,
    )) = chunk
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
      // (there's up to 6 digits after the decimal point)
      chunk.to_mut().replace_range(
        index..(index + PATTERN_LEN + len_after),
        "AAAAAAAAAAAAAAAAAAAAAAAAAA",
      );
    }
  }

  chunk
}

fn sort_within_sections<T: Ord + ?Sized>(vec: &mut [&T], mut section: impl FnMut(&T) -> u8) {
  vec.sort_unstable_by_key(|&i| (section(i), i));
}

fn chunks(dump: &str) -> impl Iterator<Item = &str> {
  let mut remaining = dump;
  std::iter::from_fn(move || {
    remaining = remaining.trim_start();
    while let Some(s) = remove_skipped_item_from_beginning(remaining) {
      remaining = s.trim_start();
    }

    // `trim_start` guarantees that `result` is not empty
    let (result, after_result) = remaining.split_once("\n\n")?;
    remaining = after_result;
    Some(result)
  })
}

fn remove_skipped_item_from_beginning(s: &str) -> Option<&str> {
  // Skip commented line
  if let Some(after) = s.strip_prefix("--") {
    Some(after_first_occurence(after, "\n"))
  }
  // Skip view definition that's replaced later (the first definition selects all nulls)
  else if let Some(after) = s.strip_prefix("CREATE VIEW ") {
    let (name, after_name) = after.split_once(' ').unwrap_or_default();
    Some(after_first_occurence(after_name, "\n\n"))
      .filter(|after_view| after_view.contains(&format!("\nCREATE OR REPLACE VIEW {name} ")))
  } else {
    None
  }
}

fn after_first_occurence<'a>(s: &'a str, pat: &str) -> &'a str {
  s.split_once(pat).unwrap_or_default().1
}
