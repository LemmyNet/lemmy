#![cfg(test)]
#![expect(clippy::expect_used)]
use lemmy_utils::settings::SETTINGS;
use std::{
  borrow::Cow,
  collections::{
    btree_set::{self, BTreeSet},
    HashSet,
  },
  process::{Command, Stdio},
};

// It's not possible to call `export_snapshot()` for each dump and run the dumps in parallel with
// the `--snapshot` flag. Don't waste your time!!!!

pub fn get_dump() -> String {
  let db_url = SETTINGS.get_database_url();
  let output = Command::new("pg_dump")
    .args([
      // Specify database URL
      "--dbname",
      &db_url,
      // Allow differences in row data and old fast tables
      "--schema-only",
      "--exclude-table=comment_aggregates_fast",
      "--exclude-table=community_aggregates_fast",
      "--exclude-table=post_aggregates_fast",
      "--exclude-table=user_fast",
      // Ignore some things to reduce the amount of queries done by pg_dump
      "--no-owner",
      "--no-privileges",
      "--no-comments",
      "--no-publications",
      "--no-security-labels",
      "--no-subscriptions",
      "--no-table-access-method",
      "--no-tablespaces",
      "--no-large-objects",
    ])
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

  let mut match_before_len = before
    .as_bytes()
    .into_iter()
    .zip(after.as_bytes())
    .position(|(a, b)| a != b)
    .unwrap_or(0);
  let mut match_after_len = before
    .as_bytes()
    .into_iter()
    .rev()
    .zip(after.as_bytes().into_iter().rev())
    .position(|(a, b)| a != b)
    .unwrap_or(0);
  match_before_len = before
    .get(0..match_before_len)
    .and_then(|s| s.rfind("\n\n"))
    .unwrap_or(0);
  match_after_len -= before
    .get((before.len() - match_after_len)..)
    .and_then(|s| s.find("\n\n"))
    .unwrap_or(0);
  let [before_chunks, after_chunks] = [&before, &after].map(|dump| {
    dump
      .get(match_before_len..(dump.len() - match_after_len))
      .unwrap_or(dump.as_str())
      .split("\n\n")
      .filter_map(normalize_chunk)
      .collect::<Vec<_>>()
  });
  let mut before_diff = BTreeSet::new();
  let mut after_diff = BTreeSet::new();
  let diff_results = diff::slice(&before_chunks, &after_chunks);
  dbg!(diff_results.len());
  for res in diff_results {
    match res {
      diff::Result::Both(_, _) => (),
      diff::Result::Left(chunk) => {
        before_diff.insert((&**chunk));
      }
      diff::Result::Right(chunk) => {
        after_diff.insert((&**chunk));
      }
    }
  }
  let diffs = [before_diff, after_diff];
  let [only_in_before, only_in_after] = diffs
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

      let diff_lines = if after_has_more {
        diff::lines(most_similar_chunk, chunk)
      } else {
        diff::lines(chunk, most_similar_chunk)
      };

      other_chunks.swap_remove(most_similar_chunk_index);

      Some(
        diff_lines
          .into_iter()
          .flat_map(|line| match line {
            diff::Result::Left(s) => ["- ", s, "\n"],
            diff::Result::Right(s) => ["+ ", s, "\n"],
            diff::Result::Both(s, _) => ["  ", s, "\n"],
          })
          .chain(["\n"])
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

fn normalize_chunk(mut chunk: &str) -> Option<Cow<'_, str>> {
  chunk = chunk.trim();
  while let Some(s) = remove_skipped_item_from_beginning(chunk) {
    chunk = s.trim_start();
  }
  if chunk.is_empty() ||
  // Skip old views and fast table triggers
  chunk.strip_prefix("CREATE ").is_some_and(|c| {
    c
      .starts_with("VIEW ")
      || c.starts_with("OR REPLACE VIEW ")
      || c.starts_with("MATERIALIZED VIEW ")
      || c.strip_prefix("FUNCTION public.")
          .and_then(after_skipped_trigger_name)
          .is_some_and(|a| a.starts_with('('))
      ||
        c.strip_prefix("TRIGGER ")
          .and_then(after_skipped_trigger_name)
          .is_some_and(|a| a.starts_with(' '))
  }) {
    return None;
  }
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
  }

  // Replace timestamps with a constant string, so differences in timestamps are ignored
  /*for index in 0.. {
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
  }*/

  Some(chunk)
}

fn sort_within_sections<T: Ord + ?Sized>(vec: &mut [&T], mut section: impl FnMut(&T) -> u8) {
  vec.sort_unstable_by_key(|&i| (section(i), i));
}

fn remove_skipped_item_from_beginning(s: &str) -> Option<&str> {
  // Skip commented line
  if let Some(after) = s.strip_prefix("--") {
    Some(after_first_occurence(after, "\n"))
  } else {
    None
  }
}

fn after_first_occurence<'a>(s: &'a str, pat: &str) -> &'a str {
  s.split_once(pat).unwrap_or_default().1
}

fn after_skipped_trigger_name(s: &str) -> Option<&str> {
  s.strip_prefix("refresh_comment_like")
    .or_else(|| s.strip_prefix("refresh_comment"))
    .or_else(|| s.strip_prefix("refresh_community_follower"))
    .or_else(|| s.strip_prefix("refresh_community_user_ban"))
    .or_else(|| s.strip_prefix("refresh_community"))
    .or_else(|| s.strip_prefix("refresh_post_like"))
    .or_else(|| s.strip_prefix("refresh_post"))
    .or_else(|| s.strip_prefix("refresh_private_message"))
    .or_else(|| s.strip_prefix("refresh_user"))
}
