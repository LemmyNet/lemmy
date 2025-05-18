#![cfg(test)]
#![expect(clippy::expect_used)]
use itertools::Itertools;
use lemmy_utils::settings::SETTINGS;
use std::{
  borrow::Cow,
  collections::{
    btree_set::{self, BTreeSet},
    HashSet,
  },
  num::NonZero,
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

  let len_of_match_at_beginning = count_bytes_until_chunks_dont_match(
    before.as_bytes().into_iter(),
    after.as_bytes().into_iter(),
  );
  let len_of_match_at_end = count_bytes_until_chunks_dont_match(
    before.as_bytes().into_iter().rev(),
    after.as_bytes().into_iter().rev(),
  );
  let [before_chunks, after_chunks] = [&before, &after].map(|dump| {
    dump
      .get(len_of_match_at_beginning..(dump.len() - len_of_match_at_end))
      .expect("invalid count_bytes_until_chunks_dont_match result")
      .split("\n\n")
      .filter_map(remove_ignored_details_from_chunk)
      .collect::<Vec<_>>()
  });
  let diff_results = diff::slice(&before_chunks, &after_chunks);
  dbg!(diff_results.len());
  let mut before_diff = BTreeSet::new();
  let mut after_diff = BTreeSet::new();
  for res in diff_results {
    let not_duplicate = match res {
      diff::Result::Both(_, _) => true,
      diff::Result::Left(chunk) => before_diff.insert((&**chunk)),
      diff::Result::Right(chunk) => after_diff.insert((&**chunk)),
    };
    assert!(
      not_duplicate,
      "a dump contains the same chunk multiple times"
    );
  }
  let [only_in_before, only_in_after] = [
    before_diff.difference(&after_diff),
    after_diff.difference(&before_diff),
  ]
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
      // Compare with the most similar chunk in the other dump, to output better diffs
      let (most_similar_chunk_index, most_similar_chunk) = other_chunks
        .iter()
        .enumerate()
        .max_by_key(|(_, other_chunk)| {
          let command_name_match = sql_command_name(chunk) == sql_command_name(other_chunk);
          (command_name_match, similarity(chunk, other_chunk))
        })?;

      let [chunk_before, chunk_after] = if after_has_more {
        [most_similar_chunk, chunk]
      } else {
        [chunk, most_similar_chunk]
      };

      let diff_lines = diff::lines(chunk_before, chunk_after);

      other_chunks.swap_remove(most_similar_chunk_index);

      Some(
        diff_lines
          .into_iter()
          .flat_map(|line| match line {
            diff::Result::Left(s) => ["- ", s, "\n"],
            diff::Result::Right(s) => ["+ ", s, "\n"],
            diff::Result::Both(s, _) => ["  ", s, "\n"],
          })
          .chain(["\n"]) // Blank line after each chunk diff
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

fn count_bytes_until_chunks_dont_match<
  'a,
  T: Iterator<Item = &'a u8> + DoubleEndedIterator + ExactSizeIterator + Clone,
>(
  iter0: T,
  iter1: T,
) -> usize {
  // iter0: FOO\n\nBAR\n\nBUNNY
  // iter1: FOO\n\nBAR\n\nBURROW
  //        ^^^^^^^^^^^^^^^^ matching_len
  //                      ^^ last_chunk_matching_len
  let matching_len = iter0
    .clone()
    .zip(iter1.clone())
    .take_while(|(a, b)| a == b)
    .count();
  let last_chunk_matching_len = iter0
    .take(matching_len)
    .rev()
    .copied()
    .tuple_windows::<(_, _)>()
    .take_while(|&a| a != (b'\n', b'\n'))
    .count();
  matching_len - last_chunk_matching_len
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

fn remove_ignored_details_from_chunk(mut chunk: &str) -> Option<Cow<'_, str>> {
  while let Some(s) = trim_start_of_chunk(chunk) {
    chunk = s;
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

  // Sort column names, so differences in column order are ignored
  if chunk.starts_with("CREATE TABLE ") {
    let mut lines = chunk
      .lines()
      .map(|line| line.strip_suffix(',').unwrap_or(line))
      .collect::<Vec<_>>();

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

fn trim_start_of_chunk(s: &str) -> Option<&str> {
  if let Some(after) = s.strip_prefix("--") {
    // Skip commented line
    Some(after_first_occurence(after, "\n"))
  } else if let Some(after) = s.strip_prefix(char::is_whitespace) {
    // Skip whitespace
    Some(after.trim_start())
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

// cfg(test) would be redundant here
mod tests {
  #[test]
  fn test_count_bytes_until_chunks_dont_match() {
    let a = b"FOO\n\nFOO\n\nFOO\nFOO";
    let b = b"FOO\n\nFOO\n\nFOO\nBAR";
    let c = b"FOO\n\nFOO\n\n";
    assert_eq!(
      super::count_bytes_until_chunks_dont_match(a.into_iter(), b.into_iter()),
      c.len()
    );
    assert_eq!(
      super::count_bytes_until_chunks_dont_match(a.into_iter(), a.into_iter()),
      a.len()
    );
    assert_eq!(
      super::count_bytes_until_chunks_dont_match(b"z".into_iter(), b"z".into_iter()),
      1
    );
    assert_eq!(
      super::count_bytes_until_chunks_dont_match(b"z".into_iter(), b"y".into_iter()),
      0
    );
  }
}
